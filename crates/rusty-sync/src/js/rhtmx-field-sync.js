/**
 * rhtmx-field-sync.js - WebSocket-based Field-Level Sync
 * Client-side field-level synchronization for RHTMX with WebSocket/SSE support
 *
 * Features:
 * - WebSocket primary, SSE fallback
 * - Automatic reconnection with exponential backoff
 * - Offline queue with automatic sync
 * - Heartbeat/ping-pong
 * - Connection state management
 * - Optimistic field updates
 * - CRDT-like field-level conflict resolution
 * - Multi-tab sync via BroadcastChannel
 *
 * Usage:
 * <script src="/api/sync/field-client.js"
 *         data-sync-entities="users,posts"
 *         data-field-strategy="last-write-wins"
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

    class RHTMXFieldSync {
        constructor(config) {
            this.entities = config.entities || [];
            this.fieldStrategy = config.fieldStrategy || 'last-write-wins';
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
            this.reconnectDelay = 1000;
            this.maxReconnectDelay = 30000;
            this.heartbeatInterval = null;
            this.heartbeatTimeout = null;

            // Sync state
            this.db = null;
            this.syncInProgress = false;
            this.pendingChanges = new Map();
            this.isOnline = navigator.onLine;

            // Multi-tab sync
            this.broadcastChannel = null;
            this.tabId = this.generateTabId();
            this.processingBroadcast = false;

            this.log('Initializing RHTMX Field Sync', {
                entities: this.entities,
                useWebSocket: this.useWebSocket,
                compression: this.compressionEnabled ? `enabled (${this.compressionThreshold}B threshold)` : 'disabled',
                tabId: this.tabId
            });
        }

        log(...args) {
            if (this.debug) {
                console.log('[RHTMX Field Sync]', ...args);
            }
        }

        error(...args) {
            console.error('[RHTMX Field Sync]', ...args);
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
         * Initialize IndexedDB for field-level storage
         */
        async initIndexedDB() {
            return new Promise((resolve, reject) => {
                const request = indexedDB.open('rhtmx-field-cache', 2);

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

                    // Create field metadata store
                    if (!db.objectStoreNames.contains('_field_meta')) {
                        const metaStore = db.createObjectStore('_field_meta', { keyPath: 'key' });
                        metaStore.createIndex('entity_field', ['entity', 'entity_id', 'field']);
                    }

                    // Create pending field changes store
                    if (!db.objectStoreNames.contains('_pending_fields')) {
                        const pendingStore = db.createObjectStore('_pending_fields', { autoIncrement: true });
                        pendingStore.createIndex('timestamp', 'timestamp');
                    }

                    // Create entity version store
                    if (!db.objectStoreNames.contains('_versions')) {
                        db.createObjectStore('_versions', { keyPath: 'entity' });
                    }

                    // Create optimistic field updates store
                    if (!db.objectStoreNames.contains('_optimistic_fields')) {
                        db.createObjectStore('_optimistic_fields', {
                            keyPath: ['entity', 'entity_id', 'field']
                        });
                    }

                    this.log('IndexedDB schema created');
                };
            });
        }

        /**
         * Perform initial field sync for all entities
         */
        async initialSync() {
            this.log('Starting initial field sync');

            for (const entity of this.entities) {
                await this.syncEntity(entity);
            }

            this.log('Initial field sync complete');
        }

        /**
         * Sync field changes for a single entity via HTTP
         */
        async syncEntity(entity) {
            try {
                const lastVersion = await this.getLastVersion(entity);
                this.log(`Syncing ${entity} fields since version ${lastVersion}`);

                const response = await fetch(`/api/field-sync/${entity}?since=${lastVersion}`);
                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
                }

                const data = await response.json();
                this.log(`Received ${data.changes.length} field changes for ${entity}`);

                await this.applyFieldChanges(entity, data.changes);
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
                this.log('WebSocket not available, feature limited');
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
            this.log('Connecting to Field Sync WebSocket');

            const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
            const wsUrl = `${protocol}//${location.host}/api/field-sync/ws`;

            try {
                this.ws = new WebSocket(wsUrl);

                this.ws.onopen = () => {
                    this.log('Field Sync WebSocket connected');
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
                    this.syncPendingChanges();
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
                    this.log('Field Sync WebSocket closed');
                    this.stopHeartbeat();
                    this.handleDisconnect();
                };

            } catch (error) {
                this.error('Failed to create WebSocket:', error);
                this.updateConnectionState(ConnectionState.DISCONNECTED);
            }
        }

        /**
         * Handle WebSocket messages
         */
        async handleWebSocketMessage(message) {
            this.log('Received Field Sync message:', message.type);

            switch (message.type) {
                case 'field_change':
                    await this.applyFieldChanges(message.change.entity, [message.change]);
                    this.triggerRefresh(message.change.entity);

                    // Broadcast to other tabs
                    this.broadcastChange('field_change', {
                        change: message.change
                    });
                    break;

                case 'push_ack':
                    this.log('Field push acknowledged:', message);
                    // Clear optimistic updates
                    if (message.applied > 0) {
                        await this.clearPendingForEntity(message.entity, message.entity_id);
                    }
                    break;

                case 'conflict':
                    this.handleConflict(message);
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

            this.heartbeatInterval = setInterval(() => {
                if (this.sendMessageSync({ type: 'ping' })) {
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
                this.log('Max reconnect attempts reached');
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
         * Update connection state and emit event
         */
        updateConnectionState(newState) {
            const oldState = this.connectionState;
            this.connectionState = newState;

            if (oldState !== newState) {
                this.log(`Connection state: ${oldState} -> ${newState}`);
                window.dispatchEvent(new CustomEvent('rhtmx:field:connection:state', {
                    detail: { state: newState, oldState }
                }));
            }
        }

        /**
         * Apply field changes to IndexedDB
         */
        async applyFieldChanges(entity, changes) {
            if (changes.length === 0) return;

            return new Promise((resolve, reject) => {
                const tx = this.db.transaction([entity, '_field_meta'], 'readwrite');
                const entityStore = tx.objectStore(entity);
                const metaStore = tx.objectStore('_field_meta');

                // Group changes by entity_id
                const changesByEntity = new Map();
                for (const change of changes) {
                    if (!changesByEntity.has(change.entity_id)) {
                        changesByEntity.set(change.entity_id, []);
                    }
                    changesByEntity.get(change.entity_id).push(change);
                }

                // Process each entity instance
                for (const [entityId, entityChanges] of changesByEntity) {
                    const getRequest = entityStore.get(entityId);

                    getRequest.onsuccess = () => {
                        let entityData = getRequest.result || { id: entityId };

                        // Apply each field change
                        for (const change of entityChanges) {
                            if (change.action === 'update') {
                                entityData[change.field] = change.value;
                            } else if (change.action === 'delete') {
                                delete entityData[change.field];
                            }

                            // Store field metadata
                            const metaKey = `${entity}:${entityId}:${change.field}`;
                            metaStore.put({
                                key: metaKey,
                                entity: entity,
                                entity_id: entityId,
                                field: change.field,
                                version: change.version,
                                timestamp: change.timestamp
                            });
                        }

                        entityStore.put(entityData);
                        this.log(`Applied ${entityChanges.length} field changes to ${entity}:${entityId}`);
                    };
                }

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Record a local field change (with optimistic update)
         */
        async recordFieldChange(entity, entityId, field, value) {
            const timestamp = new Date().toISOString();

            // Apply optimistically
            await this.applyOptimisticFieldChange(entity, entityId, field, value, timestamp);
            this.triggerRefresh(entity);

            // Broadcast optimistic update to other tabs
            this.broadcastChange('optimistic_field', {
                entity,
                entityId,
                field,
                value,
                timestamp
            });

            if (this.connectionState === ConnectionState.CONNECTED) {
                // Send via WebSocket
                this.sendMessage({
                    type: 'push_fields',
                    entity,
                    entity_id: entityId,
                    fields: [{
                        field,
                        value,
                        action: 'update',
                        timestamp
                    }]
                });
            } else {
                // Queue for later
                await this.queueFieldChange(entity, entityId, field, value, timestamp);
            }
        }

        /**
         * Apply optimistic field change
         */
        async applyOptimisticFieldChange(entity, entityId, field, value, timestamp) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(['_optimistic_fields', entity, '_field_meta'], 'readwrite');

                // Store in optimistic
                const optStore = tx.objectStore('_optimistic_fields');
                optStore.put({
                    entity,
                    entity_id: entityId,
                    field,
                    value,
                    timestamp: Date.now()
                });

                // Apply to entity
                const entityStore = tx.objectStore(entity);
                const getRequest = entityStore.get(entityId);

                getRequest.onsuccess = () => {
                    const entityData = getRequest.result || { id: entityId };
                    entityData[field] = value;
                    entityStore.put(entityData);

                    // Update field metadata
                    const metaStore = tx.objectStore('_field_meta');
                    const metaKey = `${entity}:${entityId}:${field}`;
                    metaStore.put({
                        key: metaKey,
                        entity: entity,
                        entity_id: entityId,
                        field: field,
                        timestamp: timestamp,
                        local: true
                    });
                };

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Queue field change for server sync
         */
        async queueFieldChange(entity, entityId, field, value, timestamp) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(['_pending_fields'], 'readwrite');
                const store = tx.objectStore('_pending_fields');

                store.add({
                    entity: entity,
                    entity_id: entityId,
                    field: field,
                    value: value,
                    action: 'update',
                    timestamp: timestamp,
                    queued_at: Date.now()
                });

                tx.oncomplete = () => {
                    this.log(`Queued field change: ${entity}:${entityId}.${field}`);
                    resolve();
                };
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Sync pending field changes
         */
        async syncPendingChanges() {
            if (this.syncInProgress) return;
            this.syncInProgress = true;

            try {
                const pending = await this.getPendingFieldChanges();
                this.log(`Syncing ${pending.length} pending field changes`);

                if (pending.length === 0) {
                    this.syncInProgress = false;
                    return;
                }

                // Group by entity and entity_id
                const grouped = new Map();
                for (const change of pending) {
                    const key = `${change.entity}:${change.entity_id}`;
                    if (!grouped.has(key)) {
                        grouped.set(key, {
                            entity: change.entity,
                            entity_id: change.entity_id,
                            fields: []
                        });
                    }
                    grouped.get(key).fields.push({
                        field: change.field,
                        value: change.value,
                        action: change.action,
                        timestamp: change.timestamp
                    });
                }

                // Send each group
                for (const [key, group] of grouped) {
                    if (this.connectionState === ConnectionState.CONNECTED) {
                        this.sendMessage({
                            type: 'push_fields',
                            entity: group.entity,
                            entity_id: group.entity_id,
                            fields: group.fields
                        });
                    } else {
                        // Use HTTP fallback
                        await this.pushFieldsViaHTTP(group);
                    }
                }

                // Clear pending
                await this.clearPendingFieldChanges();

            } catch (error) {
                this.error('Failed to sync pending field changes:', error);
            } finally {
                this.syncInProgress = false;
            }
        }

        /**
         * Push fields via HTTP (fallback)
         */
        async pushFieldsViaHTTP(group) {
            try {
                const response = await fetch(`/api/field-sync/${group.entity}`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        changes: group.fields.map(f => ({
                            entity_id: group.entity_id,
                            field: f.field,
                            value: f.value,
                            action: f.action,
                            timestamp: f.timestamp
                        }))
                    })
                });

                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}`);
                }

                const result = await response.json();
                this.log('Fields pushed via HTTP:', result);

                // Handle conflicts
                if (result.conflicts && result.conflicts.length > 0) {
                    for (const conflict of result.conflicts) {
                        this.handleConflict(conflict);
                    }
                }
            } catch (error) {
                this.error('Failed to push fields via HTTP:', error);
                throw error;
            }
        }

        /**
         * Handle field conflict
         */
        handleConflict(conflict) {
            this.log('Field conflict detected:', conflict);

            // Emit custom event
            window.dispatchEvent(new CustomEvent('rhtmx:field:conflict', {
                detail: conflict
            }));

            // Apply resolution based on strategy
            if (this.fieldStrategy === 'server-wins' && conflict.server_value !== undefined) {
                this.applyFieldChanges(conflict.entity, [{
                    entity_id: conflict.entity_id,
                    field: conflict.field,
                    value: conflict.server_value,
                    action: 'update',
                    timestamp: conflict.server_timestamp
                }]);
            }
        }

        /**
         * Get pending field changes
         */
        async getPendingFieldChanges() {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(['_pending_fields'], 'readonly');
                const store = tx.objectStore('_pending_fields');
                const request = store.getAll();

                request.onsuccess = () => resolve(request.result || []);
                request.onerror = () => reject(request.error);
            });
        }

        /**
         * Clear pending field changes
         */
        async clearPendingFieldChanges() {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(['_pending_fields'], 'readwrite');
                const store = tx.objectStore('_pending_fields');
                store.clear();

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Clear pending for specific entity
         */
        async clearPendingForEntity(entity, entityId) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(['_optimistic_fields'], 'readwrite');
                const store = tx.objectStore('_optimistic_fields');

                // Clear all optimistic updates for this entity instance
                const range = IDBKeyRange.bound(
                    [entity, entityId, ''],
                    [entity, entityId, '\uffff']
                );

                const request = store.openCursor(range);
                request.onsuccess = (event) => {
                    const cursor = event.target.result;
                    if (cursor) {
                        cursor.delete();
                        cursor.continue();
                    }
                };

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Get last synced version
         */
        async getLastVersion(entity) {
            return new Promise((resolve) => {
                const tx = this.db.transaction(['_versions'], 'readonly');
                const store = tx.objectStore('_versions');
                const request = store.get(entity);

                request.onsuccess = () => {
                    const result = request.result;
                    resolve(result ? result.version : 0);
                };
                request.onerror = () => resolve(0);
            });
        }

        /**
         * Set last synced version
         */
        async setLastVersion(entity, version) {
            return new Promise((resolve, reject) => {
                const tx = this.db.transaction(['_versions'], 'readwrite');
                const store = tx.objectStore('_versions');
                store.put({ entity: entity, version: version });

                tx.oncomplete = () => resolve();
                tx.onerror = () => reject(tx.error);
            });
        }

        /**
         * Trigger HTMX refresh
         */
        triggerRefresh(entity) {
            const event = new CustomEvent(`rhtmx:${entity}:field-changed`, {
                detail: { entity: entity },
                bubbles: true
            });
            document.body.dispatchEvent(event);
            this.log(`Triggered field refresh for ${entity}`);
        }

        /**
         * Setup online/offline handlers
         */
        setupOfflineHandlers() {
            window.addEventListener('online', async () => {
                this.log('Back online');
                this.isOnline = true;

                if (this.useWebSocket) {
                    this.connectWebSocket();
                } else {
                    await this.syncPendingChanges();
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
                this.broadcastChannel = new BroadcastChannel('rhtmx-field-sync');
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
                    case 'field_change':
                        // Apply field change from another tab
                        await this.applyFieldChange(message.change);
                        this.triggerFieldRefresh(message.change.entity, message.change.entity_id, message.change.field);
                        break;

                    case 'optimistic_field':
                        // Apply optimistic field update from another tab
                        await this.applyOptimisticFieldChange(
                            message.entity,
                            message.entityId,
                            message.field,
                            message.value,
                            message.timestamp
                        );
                        this.triggerFieldRefresh(message.entity, message.entityId, message.field);
                        break;

                    case 'connection_state':
                        // Sync connection state info (optional)
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
            const fieldStrategy = scriptTag.getAttribute('data-field-strategy') || 'last-write-wins';
            const useWebSocket = scriptTag.getAttribute('data-use-websocket') !== 'false';
            const debug = scriptTag.getAttribute('data-debug') === 'true';

            // Compression configuration
            const compressionEnabled = scriptTag.getAttribute('data-compression-enabled') !== 'false'; // Default true
            const compressionThreshold = parseInt(scriptTag.getAttribute('data-compression-threshold') || '1024', 10);

            if (!entities) {
                console.error('[RHTMX Field Sync] No entities specified in data-sync-entities');
                return;
            }

            const config = {
                entities: entities.split(',').map(e => e.trim()),
                fieldStrategy,
                useWebSocket,
                compressionEnabled,
                compressionThreshold,
                debug
            };

            const sync = new RHTMXFieldSync(config);

            try {
                await sync.initIndexedDB();
                await sync.initialSync();
                sync.connectRealtime();
                sync.setupOfflineHandlers();
                sync.setupBroadcastChannel();

                // Cleanup on page unload
                window.addEventListener('beforeunload', () => sync.cleanup());

                // Make available globally
                window.RHTMXFieldSync = sync;

                console.log('[RHTMX Field Sync] Initialization complete');
                window.dispatchEvent(new CustomEvent('rhtmx:field:sync:ready'));

            } catch (error) {
                console.error('[RHTMX Field Sync] Initialization failed:', error);
            }
        }
    }

    // Auto-initialize
    if (document.currentScript && document.currentScript.hasAttribute('data-sync-entities')) {
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', () => RHTMXFieldSync.init());
        } else {
            RHTMXFieldSync.init();
        }
    }

    // Export for manual initialization
    window.RHTMXFieldSync = RHTMXFieldSync;
})();
