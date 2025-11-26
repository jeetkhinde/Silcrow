/**
 * rhtmx-sync.js - WebSocket-based Entity Sync
 * Client-side IndexedDB synchronization for RHTMX with WebSocket/SSE support
 *
 * Features:
 * - WebSocket primary, SSE fallback
 * - Automatic reconnection with exponential backoff
 * - Offline queue with automatic sync
 * - Heartbeat/ping-pong
 * - Connection state management
 * - Optimistic UI updates
 *
 * Usage:
 * <script src="/api/sync/client.js"
 *         data-sync-entities="users,posts"
 *         data-conflict-strategy="last-write-wins"
 *         data-use-websocket="true"
 *         data-debug="false">
 * </script>
 */

(function() {
    'use strict';

    // Connection states
    const ConnectionState = {
        DISCONNECTED: 'disconnected',
        CONNECTING: 'connecting',
        CONNECTED: 'connected',
        RECONNECTING: 'reconnecting',
        FALLBACK_SSE: 'fallback_sse'
    };

    class RHTMXSync {
        constructor(config) {
            this.entities = config.entities || [];
            this.conflictStrategy = config.conflictStrategy || 'last-write-wins';
            this.useWebSocket = config.useWebSocket !== false; // Default true
            this.debug = config.debug || false;

            // Connection management
            this.connectionState = ConnectionState.DISCONNECTED;
            this.ws = null;
            this.eventSource = null;
            this.reconnectAttempts = 0;
            this.maxReconnectAttempts = 10;
            this.reconnectDelay = 1000; // Start at 1 second
            this.maxReconnectDelay = 30000; // Max 30 seconds
            this.heartbeatInterval = null;
            this.heartbeatTimeout = null;

            // Sync state
            this.db = null;
            this.syncInProgress = false;
            this.offlineQueue = [];
            this.isOnline = navigator.onLine;

            this.log('Initializing RHTMX Sync', {
                entities: this.entities,
                useWebSocket: this.useWebSocket
            });
        }

        log(...args) {
            if (this.debug) {
                console.log('[RHTMX Sync]', ...args);
            }
        }

        error(...args) {
            console.error('[RHTMX Sync]', ...args);
        }

        /**
         * Initialize IndexedDB
         */
        async initIndexedDB() {
            return new Promise((resolve, reject) => {
                const request = indexedDB.open('rhtmx-cache', 2); // Version 2 for schema updates

                request.onerror = () => {
                    this.error('Failed to open IndexedDB', request.error);
                    reject(request.error);
                };

                request.onsuccess = () => {
                    this.db = request.result;
                    this.log('IndexedDB initialized');
                    resolve();
                };

                request.onupgradeneeded = (event) => {
                    const db = event.target.result;

                    // Create object stores for each entity
                    for (const entity of this.entities) {
                        if (!db.objectStoreNames.contains(entity)) {
                            db.createObjectStore(entity, { keyPath: 'id' });
                            this.log(`Created object store: ${entity}`);
                        }
                    }

                    // Create metadata store
                    if (!db.objectStoreNames.contains('_meta')) {
                        db.createObjectStore('_meta', { keyPath: 'key' });
                    }

                    // Create pending mutations store with timestamp index
                    if (!db.objectStoreNames.contains('_pending')) {
                        const pendingStore = db.createObjectStore('_pending', { autoIncrement: true });
                        pendingStore.createIndex('timestamp', 'timestamp');
                    }

                    // Create optimistic updates store
                    if (!db.objectStoreNames.contains('_optimistic')) {
                        db.createObjectStore('_optimistic', { keyPath: ['entity', 'entity_id'] });
                    }

                    this.log('IndexedDB schema created');
                };
            });
        }

        /**
         * Perform initial sync for all entities
         */
        async initialSync() {
            this.log('Starting initial sync');

            for (const entity of this.entities) {
                await this.syncEntity(entity);
            }

            this.log('Initial sync complete');
        }

        /**
         * Sync a single entity via HTTP
         */
        async syncEntity(entity) {
            try {
                const lastVersion = await this.getLastVersion(entity);
                this.log(`Syncing ${entity} since version ${lastVersion}`);

                const response = await fetch(`/api/sync/${entity}?since=${lastVersion}`);
                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();
                this.log(`Received ${data.changes.length} changes for ${entity}`);

                await this.applyChanges(entity, data.changes);
                await this.setLastVersion(entity, data.version);

                this.triggerRefresh(entity);
                this.log(`Synced ${entity} to version ${data.version}`);
            } catch (error) {
                this.error(`Failed to sync ${entity}:`, error);
            }
        }

        /**
         * Connect to real-time sync (WebSocket or SSE)
         */
        connectRealtime() {
            if (this.useWebSocket && 'WebSocket' in window) {
                this.connectWebSocket();
            } else {
                this.connectSSE();
            }
        }

        /**
         * Connect via WebSocket
         */
        connectWebSocket() {
            if (this.connectionState === ConnectionState.CONNECTING ||
                this.connectionState === ConnectionState.CONNECTED) {
                return;
            }

            this.updateConnectionState(ConnectionState.CONNECTING);
            this.log('Connecting to WebSocket');

            const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUrl = `${protocol}//${location.host}/api/sync/ws`;

            try {
                this.ws = new WebSocket(wsUrl);

                this.ws.onopen = () => {
                    this.log('WebSocket connected');
                    this.updateConnectionState(ConnectionState.CONNECTED);
                    this.reconnectAttempts = 0;
                    this.reconnectDelay = 1000;

                    // Subscribe to entities
                    this.sendMessage({
                        type: 'subscribe',
                        entities: this.entities
                    });

                    // Start heartbeat
                    this.startHeartbeat();

                    // Sync pending offline mutations
                    this.syncPendingMutations();
                };

                this.ws.onmessage = (event) => {
                    try {
                        const message = JSON.parse(event.data);
                        this.handleWebSocketMessage(message);
                    } catch (error) {
                        this.error('Failed to parse WebSocket message:', error);
                    }
                };

                this.ws.onerror = (error) => {
                    this.error('WebSocket error:', error);
                };

                this.ws.onclose = () => {
                    this.log('WebSocket closed');
                    this.stopHeartbeat();
                    this.handleDisconnect();
                };

            } catch (error) {
                this.error('Failed to create WebSocket:', error);
                this.fallbackToSSE();
            }
        }

        /**
         * Handle WebSocket messages
         */
        async handleWebSocketMessage(message) {
            this.log('Received WebSocket message:', message.type);

            switch (message.type) {
                case 'change':
                    await this.applyChanges(message.change.entity, [message.change]);
                    this.triggerRefresh(message.change.entity);
                    break;

                case 'push_ack':
                    this.log('Push acknowledged:', message);
                    // Remove from optimistic updates
                    await this.clearOptimistic(message.entity, message.entity_id);
                    break;

                case 'error':
                    this.error('Server error:', message.message);
                    break;

                case 'pong':
                    this.resetHeartbeatTimeout();
                    break;
            }
        }

        /**
         * Send message via WebSocket
         */
        sendMessage(message) {
            if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                this.ws.send(JSON.stringify(message));
                return true;
            }
            return false;
        }

        /**
         * Start heartbeat ping/pong
         */
        startHeartbeat() {
            this.stopHeartbeat();

            // Send ping every 30 seconds
            this.heartbeatInterval = setInterval(() => {
                if (this.sendMessage({ type: 'ping' })) {
                    // Expect pong within 5 seconds
                    this.heartbeatTimeout = setTimeout(() => {
                        this.error('Heartbeat timeout, reconnecting');
                        this.ws.close();
                    }, 5000);
                }
            }, 30000);
        }

        /**
         * Stop heartbeat
         */
        stopHeartbeat() {
            if (this.heartbeatInterval) {
                clearInterval(this.heartbeatInterval);
                this.heartbeatInterval = null;
            }
            if (this.heartbeatTimeout) {
                clearTimeout(this.heartbeatTimeout);
                this.heartbeatTimeout = null;
            }
        }

        /**
         * Reset heartbeat timeout
         */
        resetHeartbeatTimeout() {
            if (this.heartbeatTimeout) {
                clearTimeout(this.heartbeatTimeout);
                this.heartbeatTimeout = null;
            }
        }

        /**
         * Handle disconnection
         */
        handleDisconnect() {
            this.updateConnectionState(ConnectionState.DISCONNECTED);

            if (this.reconnectAttempts < this.maxReconnectAttempts) {
                this.reconnect();
            } else {
                this.log('Max reconnect attempts reached, falling back to SSE');
                this.fallbackToSSE();
            }
        }

        /**
         * Reconnect with exponential backoff
         */
        reconnect() {
            this.reconnectAttempts++;
            this.updateConnectionState(ConnectionState.RECONNECTING);

            const delay = Math.min(
                this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1),
                this.maxReconnectDelay
            );

            this.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);

            setTimeout(() => {
                if (this.useWebSocket) {
                    this.connectWebSocket();
                }
            }, delay);
        }

        /**
         * Fallback to SSE
         */
        fallbackToSSE() {
            this.log('Falling back to SSE');
            this.useWebSocket = false;
            this.connectSSE();
        }

        /**
         * Connect via SSE
         */
        connectSSE() {
            this.updateConnectionState(ConnectionState.FALLBACK_SSE);
            this.log('Connecting to SSE');

            this.eventSource = new EventSource('/api/sync/events');

            this.eventSource.addEventListener('sync', async (event) => {
                try {
                    const change = JSON.parse(event.data);
                    this.log('Received SSE update', change);

                    await this.applyChanges(change.entity, [change]);
                    this.triggerRefresh(change.entity);
                } catch (error) {
                    this.error('Failed to process SSE event:', error);
                }
            });

            this.eventSource.onerror = (error) => {
                this.error('SSE error:', error);
                // SSE will auto-reconnect
            };
        }

        /**
         * Update connection state and emit event
         */
        updateConnectionState(newState) {
            const oldState = this.connectionState;
            this.connectionState = newState;

            if (oldState !== newState) {
                this.log(`Connection state: ${oldState} -> ${newState}`);
                window.dispatchEvent(new CustomEvent('rhtmx:connection:state', {
                    detail: { state: newState, oldState }
                }));
            }
        }

        /**
         * Apply changes to IndexedDB
         */
        async applyChanges(entity, changes) {
            if (changes.length === 0) return;

            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(entity, 'readwrite');
                const store = tx.objectStore(entity);

                for (const change of changes) {
                    try {
                        if (change.action === 'delete') {
                            store.delete(change.entity_id);
                        } else if (change.data) {
                            store.put(change.data);
                        }
                    } catch (error) {
                        this.error(`Failed to apply change to ${entity}:`, error);
                    }
                }

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Push change (with optimistic update)
         */
        async pushChange(entity, entityId, action, data) {
            // Apply optimistically
            await this.applyOptimistic(entity, entityId, data);
            this.triggerRefresh(entity);

            if (this.connectionState === ConnectionState.CONNECTED) {
                // Send via WebSocket
                this.sendMessage({
                    type: 'push',
                    entity,
                    entity_id: entityId,
                    action,
                    data
                });
            } else {
                // Queue for later
                await this.queueMutation(entity, entityId, action, data);
            }
        }

        /**
         * Apply optimistic update
         */
        async applyOptimistic(entity, entityId, data) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(['_optimistic', entity], 'readwrite');

                // Store in optimistic
                const optStore = tx.objectStore('_optimistic');
                optStore.put({ entity, entity_id: entityId, data, timestamp: Date.now() });

                // Apply to entity store
                const entityStore = tx.objectStore(entity);
                if (data) {
                    entityStore.put(data);
                } else {
                    entityStore.delete(entityId);
                }

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Clear optimistic update
         */
        async clearOptimistic(entity, entityId) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction('_optimistic', 'readwrite');
                const store = tx.objectStore('_optimistic');
                store.delete([entity, entityId]);

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Queue mutation for offline sync
         */
        async queueMutation(entity, entityId, action, data) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction('_pending', 'readwrite');
                const store = tx.objectStore('_pending');

                store.add({
                    entity,
                    entity_id: entityId,
                    action,
                    data,
                    timestamp: Date.now()
                });

                tx.oncomplete = () => {
                    this.log(`Queued mutation: ${entity}:${entityId}`);
                    resolve();
                };
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Sync pending mutations
         */
        async syncPendingMutations() {
            if (this.syncInProgress) return;
            this.syncInProgress = true;

            try {
                const pending = await this.getPendingMutations();
                this.log(`Syncing ${pending.length} pending mutations`);

                for (const mutation of pending) {
                    if (this.connectionState === ConnectionState.CONNECTED) {
                        this.sendMessage({
                            type: 'push',
                            entity: mutation.entity,
                            entity_id: mutation.entity_id,
                            action: mutation.action,
                            data: mutation.data
                        });
                    } else {
                        // Use HTTP fallback
                        await this.pushViaHTTP(mutation);
                    }
                }

                // Clear pending
                await this.clearPendingMutations();

            } catch (error) {
                this.error('Failed to sync pending mutations:', error);
            } finally {
                this.syncInProgress = false;
            }
        }

        /**
         * Push via HTTP (fallback)
         */
        async pushViaHTTP(mutation) {
            try {
                const response = await fetch(`/api/sync/${mutation.entity}`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        changes: [{
                            id: mutation.entity_id,
                            action: mutation.action,
                            data: mutation.data
                        }]
                    })
                });

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}`);
                }

                this.log('Mutation pushed via HTTP:', mutation);
            } catch (error) {
                this.error('Failed to push via HTTP:', error);
                throw error;
            }
        }

        /**
         * Get pending mutations
         */
        async getPendingMutations() {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction('_pending', 'readonly');
                const store = tx.objectStore('_pending');
                const request = store.getAll();

                request.onsuccess = () => resolve(request.result || []);
                request.onerror = () => reject(request.error);
            });
        }

        /**
         * Clear pending mutations
         */
        async clearPendingMutations() {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction('_pending', 'readwrite');
                const store = tx.objectStore('_pending');
                store.clear();

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Get last synced version
         */
        async getLastVersion(entity) {
            return new Promise((resolve) => {
                const tx = this.db.transaction('_meta', 'readonly');
                const store = tx.objectStore('_meta');
                const request = store.get(`${entity}_version`);

                request.onsuccess = () => {
                    const result = request.result;
                    resolve(result ? result.value : 0);
                };
                request.onerror = () => resolve(0);
            });
        }

        /**
         * Set last synced version
         */
        async setLastVersion(entity, version) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction('_meta', 'readwrite');
                const store = tx.objectStore('_meta');
                store.put({ key: `${entity}_version`, value: version });

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Trigger HTMX refresh
         */
        triggerRefresh(entity) {
            const event = new CustomEvent(`rhtmx:${entity}:changed`, {
                detail: { entity },
                bubbles: true
            });
            document.body.dispatchEvent(event);
            this.log(`Triggered refresh for ${entity}`);
        }

        /**
         * Setup online/offline handlers
         */
        setupOfflineHandlers() {
            window.addEventListener('online', async () => {
                this.log('Back online');
                this.isOnline = true;

                // Reconnect WebSocket or sync pending
                if (this.useWebSocket) {
                    this.connectWebSocket();
                } else {
                    await this.syncPendingMutations();
                    for (const entity of this.entities) {
                        await this.syncEntity(entity);
                    }
                }
            });

            window.addEventListener('offline', () => {
                this.log('Went offline');
                this.isOnline = false;
            });
        }

        /**
         * Cleanup on page unload
         */
        cleanup() {
            this.stopHeartbeat();
            if (this.ws) {
                this.ws.close();
            }
            if (this.eventSource) {
                this.eventSource.close();
            }
        }

        /**
         * Initialize everything
         */
        static async init() {
            const scriptTag = document.currentScript;
            const entities = scriptTag.getAttribute('data-sync-entities');
            const conflictStrategy = scriptTag.getAttribute('data-conflict-strategy') || 'last-write-wins';
            const useWebSocket = scriptTag.getAttribute('data-use-websocket') !== 'false';
            const debug = scriptTag.getAttribute('data-debug') === 'true';

            if (!entities) {
                console.error('[RHTMX Sync] No entities specified in data-sync-entities');
                return;
            }

            const config = {
                entities: entities.split(',').map(e => e.trim()),
                conflictStrategy,
                useWebSocket,
                debug
            };

            const sync = new RHTMXSync(config);

            try {
                await sync.initIndexedDB();
                await sync.initialSync();
                sync.connectRealtime();
                sync.setupOfflineHandlers();

                // Cleanup on page unload
                window.addEventListener('beforeunload', () => sync.cleanup());

                // Make available globally
                window.rhtmxSync = sync;

                console.log('[RHTMX Sync] Initialization complete');
                window.dispatchEvent(new CustomEvent('rhtmx:sync:ready'));

            } catch (error) {
                console.error('[RHTMX Sync] Initialization failed:', error);
            }
        }
    }

    // Auto-initialize
    if (document.currentScript && document.currentScript.hasAttribute('data-sync-entities')) {
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => RHTMXSync.init());
        } else {
            RHTMXSync.init();
        }
    }

    // Export for manual initialization
    window.RHTMXSync = RHTMXSync;
})();
