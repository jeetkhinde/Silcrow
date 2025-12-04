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
 * - Multi-tab sync via BroadcastChannel
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

            // Compression configuration
            this.compressionEnabled = config.compressionEnabled !== false; // Default true
            this.compressionThreshold = config.compressionThreshold || 1024; // 1KB default
            this.compressionSupported = typeof CompressionStream !== 'undefined';

            if (this.compressionEnabled && !this.compressionSupported) {
                this.log('Warning: Compression requested but CompressionStream API not supported');
                this.compressionEnabled = false;
            }

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

            // Multi-tab sync
            this.broadcastChannel = null;
            this.tabId = this.generateTabId();
            this.processingBroadcast = false;

            this.log('Initializing RHTMX Sync', {
                entities: this.entities,
                useWebSocket: this.useWebSocket,
                compression: this.compressionEnabled ? `enabled (${this.compressionThreshold}B threshold)` : 'disabled',
                tabId: this.tabId
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
         * Compress data using gzip
         */
        async compressData(text) {
            if (!this.compressionEnabled || text.length < this.compressionThreshold) {
                return null; // Don't compress
            }

            try {
                const encoder = new TextEncoder();
                const data = encoder.encode(text);

                // Use gzip compression
                const stream = new Blob([data]).stream();
                const compressedStream = stream.pipeThrough(new CompressionStream('gzip'));
                const compressedBlob = await new Response(compressedStream).blob();
                const compressed = await compressedBlob.arrayBuffer();

                // Only use compression if it actually reduces size
                if (compressed.byteLength < data.byteLength) {
                    return compressed;
                }
                return null; // Uncompressed is smaller
            } catch (error) {
                this.error('Compression failed:', error);
                return null; // Fall back to uncompressed
            }
        }

        /**
         * Decompress gzip data
         */
        async decompressData(arrayBuffer) {
            try {
                const stream = new Blob([arrayBuffer]).stream();
                const decompressedStream = stream.pipeThrough(new DecompressionStream('gzip'));
                const decompressed = await new Response(decompressedStream).text();
                return decompressed;
            } catch (error) {
                this.error('Decompression failed:', error);
                throw error;
            }
        }

        /**
         * Generate unique tab ID
         */
        generateTabId() {
            return `tab_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
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

                this.ws.onmessage = async (event) => {
                    try {
                        let messageText;

                        // Handle both text and binary (compressed) messages
                        if (typeof event.data === 'string') {
                            messageText = event.data;
                        } else if (event.data instanceof Blob) {
                            // Binary message - decompress
                            const arrayBuffer = await event.data.arrayBuffer();
                            messageText = await this.decompressData(arrayBuffer);
                        } else if (event.data instanceof ArrayBuffer) {
                            // Binary message - decompress
                            messageText = await this.decompressData(event.data);
                        } else {
                            this.error('Unknown message type:', typeof event.data);
                            return;
                        }

                        const message = JSON.parse(messageText);
                        this.handleWebSocketMessage(message);
                    } catch (error) {
                        this.error('Failed to process WebSocket message:', error);
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

                    // Broadcast to other tabs
                    this.broadcastChange('change', {
                        entity: message.change.entity,
                        change: message.change
                    });
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
         * Send message via WebSocket (with optional compression)
         */
        async sendMessage(message) {
            if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
                return false;
            }

            try {
                const json = JSON.stringify(message);
                const compressed = await this.compressData(json);

                if (compressed) {
                    // Send as binary (compressed)
                    this.ws.send(compressed);
                    this.log(`Sent compressed message (${compressed.byteLength}B from ${json.length}B)`);
                } else {
                    // Send as text (uncompressed)
                    this.ws.send(json);
                }

                return true;
            } catch (error) {
                this.error('Failed to send message:', error);
                return false;
            }
        }

        /**
         * Send message via WebSocket synchronously (no compression)
         * Used for heartbeat/ping messages
         */
        sendMessageSync(message) {
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
                if (this.sendMessageSync({ type: 'ping' })) {
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

                    // Broadcast to other tabs
                    this.broadcastChange('change', {
                        entity: change.entity,
                        change: change
                    });
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

            // Broadcast optimistic update to other tabs
            this.broadcastChange('optimistic', {
                entity,
                entityId,
                data
            });

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
         * Setup BroadcastChannel for multi-tab sync
         */
        setupBroadcastChannel() {
            if (!('BroadcastChannel' in window)) {
                this.log('BroadcastChannel not supported');
                return;
            }

            try {
                this.broadcastChannel = new BroadcastChannel('rhtmx-sync');
                this.log('BroadcastChannel initialized');

                this.broadcastChannel.onmessage = (event) => {
                    this.handleBroadcastMessage(event.data);
                };

                this.broadcastChannel.onerror = (error) => {
                    this.error('BroadcastChannel error:', error);
                };

            } catch (error) {
                this.error('Failed to setup BroadcastChannel:', error);
            }
        }

        /**
         * Handle message from another tab
         */
        async handleBroadcastMessage(message) {
            // Ignore messages from this tab
            if (message.tabId === this.tabId) {
                return;
            }

            // Prevent infinite loops
            if (this.processingBroadcast) {
                return;
            }

            this.log('Received broadcast from tab:', message.tabId, message.type);

            try {
                this.processingBroadcast = true;

                switch (message.type) {
                    case 'change':
                        // Apply change from another tab
                        await this.applyChanges(message.entity, [message.change]);
                        this.triggerRefresh(message.entity);
                        break;

                    case 'optimistic':
                        // Apply optimistic update from another tab
                        await this.applyOptimistic(message.entity, message.entityId, message.data);
                        this.triggerRefresh(message.entity);
                        break;

                    case 'connection_state':
                        // Sync connection state info (optional, for UI consistency)
                        this.log(`Tab ${message.tabId} connection state: ${message.state}`);
                        break;
                }

            } catch (error) {
                this.error('Failed to process broadcast message:', error);
            } finally {
                this.processingBroadcast = false;
            }
        }

        /**
         * Broadcast change to other tabs
         */
        broadcastChange(type, data) {
            if (!this.broadcastChannel) {
                return;
            }

            try {
                this.broadcastChannel.postMessage({
                    tabId: this.tabId,
                    type,
                    timestamp: Date.now(),
                    ...data
                });
            } catch (error) {
                this.error('Failed to broadcast:', error);
            }
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
            if (this.broadcastChannel) {
                this.broadcastChannel.close();
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

            // Compression configuration
            const compressionEnabled = scriptTag.getAttribute('data-compression-enabled') !== 'false'; // Default true
            const compressionThreshold = parseInt(scriptTag.getAttribute('data-compression-threshold') || '1024', 10);

            if (!entities) {
                console.error('[RHTMX Sync] No entities specified in data-sync-entities');
                return;
            }

            const config = {
                entities: entities.split(',').map(e => e.trim()),
                conflictStrategy,
                useWebSocket,
                compressionEnabled,
                compressionThreshold,
                debug
            };

            const sync = new RHTMXSync(config);

            try {
                await sync.initIndexedDB();
                await sync.initialSync();
                sync.connectRealtime();
                sync.setupOfflineHandlers();
                sync.setupBroadcastChannel();

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
