# WebSocket Message Compression

The RHTMX sync system now supports transparent compression of WebSocket messages for both entity-level and field-level synchronization.

## Overview

Compression automatically reduces bandwidth usage for large payloads by using gzip compression on WebSocket messages. This is especially beneficial for:

- Large entity data syncs
- Bulk field updates
- Applications with limited bandwidth
- Mobile and slow network connections

## How It Works

### Protocol

- **Text messages** (WebSocket text frames): Uncompressed JSON messages
- **Binary messages** (WebSocket binary frames): Gzip-compressed JSON messages

The system automatically decides whether to compress based on:
1. Compression enabled/disabled
2. Message size threshold
3. Whether compression actually reduces size

### Thresholds

Messages are only compressed if:
- Compression is enabled
- Message size ≥ threshold (default: 1024 bytes / 1KB)
- Compressed size < original size (automatic check)

## Server-Side Configuration

### Basic Setup

```rust
use rhtmx_sync::{SyncEngine, SyncConfig, CompressionConfig};

let config = SyncConfig::new(db_pool, vec!["users".to_string(), "posts".to_string()])
    .with_compression(CompressionConfig::default()); // Enable with default settings

let engine = SyncEngine::new(config).await?;
```

### Custom Configuration

```rust
use rhtmx_sync::CompressionConfig;

// Custom threshold and compression level
let compression = CompressionConfig::new(
    true,    // enabled
    2048,    // threshold in bytes (2KB)
    6        // compression level (0-9, 6 is balanced)
);

let config = SyncConfig::new(db_pool, entities)
    .with_compression(compression);
```

### Disable Compression

```rust
let config = SyncConfig::new(db_pool, entities)
    .without_compression();

// Or explicitly
let config = SyncConfig::new(db_pool, entities)
    .with_compression(CompressionConfig::disabled());
```

### Compression Levels

- **0**: No compression (fastest)
- **1-3**: Fast compression, lower ratio
- **4-6**: Balanced (6 is default)
- **7-9**: Best compression, slower

## Client-Side Configuration

### Entity-Level Sync

```html
<script src="/api/sync/client.js"
        data-sync-entities="users,posts"
        data-compression-enabled="true"
        data-compression-threshold="1024"
        data-debug="false">
</script>
```

### Field-Level Sync

```html
<script src="/api/sync/field-client.js"
        data-sync-entities="users,posts"
        data-compression-enabled="true"
        data-compression-threshold="1024"
        data-debug="false">
</script>
```

### Configuration Options

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `data-compression-enabled` | boolean | `true` | Enable/disable compression |
| `data-compression-threshold` | number | `1024` | Minimum message size (bytes) to compress |

### Disable Compression on Client

```html
<script src="/api/sync/client.js"
        data-sync-entities="users,posts"
        data-compression-enabled="false">
</script>
```

## Browser Compatibility

Compression uses the native browser APIs:
- `CompressionStream` (gzip)
- `DecompressionStream` (gzip)

**Supported Browsers:**
- Chrome 80+ (Feb 2020)
- Edge 80+ (Feb 2020)
- Safari 16.4+ (Mar 2023)
- Firefox 113+ (May 2023)

**Fallback:** If compression APIs are not available, the system automatically falls back to uncompressed messages with a warning in the console.

## Performance Considerations

### Benefits
- **Reduced bandwidth**: 50-90% reduction for large, repetitive data
- **Faster sync**: Less data to transfer over slow connections
- **Cost savings**: Lower data transfer costs on metered connections

### Trade-offs
- **CPU overhead**: Compression/decompression uses CPU time
- **Small messages**: Gzip overhead can make tiny messages larger
- **Battery impact**: Extra CPU usage on mobile devices

### Recommendations

1. **Enable by default** for most applications
2. **Higher threshold** (2KB-4KB) for CPU-constrained devices
3. **Disable** for:
   - All small messages (< 500 bytes consistently)
   - Real-time gaming or high-frequency updates
   - Already compressed data (images, video metadata)

## Example: Compression in Action

### Before Compression
```
Message size: 5,234 bytes (5.1 KB)
Network transfer: 5,234 bytes
```

### After Compression
```
Original size: 5,234 bytes (5.1 KB)
Compressed size: 892 bytes (0.9 KB)
Compression ratio: 83% reduction
Network transfer: 892 bytes
```

## Debugging

Enable debug logging to see compression statistics:

```html
<script src="/api/sync/client.js"
        data-sync-entities="users,posts"
        data-debug="true">
</script>
```

**Console output:**
```
[RHTMX Sync] Sent compressed message (892B from 5234B)
[RHTMX Sync] Received binary message, decompressing...
```

## Backward Compatibility

- **Fully backward compatible**: Clients without compression support receive uncompressed messages
- **Mixed environments**: Compressed and uncompressed clients can coexist
- **No protocol negotiation needed**: Works transparently

## Implementation Details

### Server (Rust)
- Library: `flate2` (gzip compression)
- Algorithm: DEFLATE (RFC 1951) with gzip wrapper (RFC 1952)
- Compression level: Configurable (0-9)

### Client (JavaScript)
- API: Native `CompressionStream` / `DecompressionStream`
- Algorithm: gzip (compatible with flate2)
- Format: Binary WebSocket frames

### Message Flow

**Client → Server:**
```
1. Serialize message to JSON
2. Check if size ≥ threshold
3. Compress with gzip (if applicable)
4. Send as binary frame (if compressed) or text frame (if not)
```

**Server → Client:**
```
1. Receive binary or text frame
2. Decompress if binary (gzip)
3. Parse JSON
4. Handle message
```

## Testing

Run compression tests:
```bash
cargo test --package rhtmx-sync compression
```

All tests:
```bash
cargo test --package rhtmx-sync
```

## Future Enhancements

Potential improvements:
- [ ] Brotli compression (better ratio, broader browser support)
- [ ] Per-entity compression settings
- [ ] Adaptive threshold based on network conditions
- [ ] Compression statistics/metrics endpoint
