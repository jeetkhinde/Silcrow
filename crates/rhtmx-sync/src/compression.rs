// File: rusty-sync/src/compression.rs
// Purpose: WebSocket message compression utilities

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::{Read, Write};

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Enable compression
    pub enabled: bool,
    /// Minimum message size in bytes to trigger compression
    pub threshold: usize,
    /// Compression level (0-9, where 6 is default)
    pub level: u32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 1024, // 1KB
            level: 6,        // Default compression
        }
    }
}

impl CompressionConfig {
    pub fn new(enabled: bool, threshold: usize, level: u32) -> Self {
        Self {
            enabled,
            threshold,
            level: level.min(9), // Cap at max level 9
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            threshold: usize::MAX,
            level: 0,
        }
    }
}

/// Compress data using gzip
pub fn compress(data: &[u8], level: u32) -> anyhow::Result<Vec<u8>> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
    encoder.write_all(data)?;
    let compressed = encoder.finish()?;
    Ok(compressed)
}

/// Decompress gzip data
pub fn decompress(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// Decide whether to compress based on config and data size
pub fn should_compress(config: &CompressionConfig, data_size: usize) -> bool {
    config.enabled && data_size >= config.threshold
}

/// Compress JSON message if it meets threshold
pub fn compress_message(
    json: &str,
    config: &CompressionConfig,
) -> anyhow::Result<CompressedMessage> {
    let bytes = json.as_bytes();

    if should_compress(config, bytes.len()) {
        let compressed = compress(bytes, config.level)?;

        // Only use compression if it actually reduces size
        if compressed.len() < bytes.len() {
            return Ok(CompressedMessage::Compressed(compressed));
        }
    }

    Ok(CompressedMessage::Uncompressed(json.to_string()))
}

/// Represents a message that may be compressed or uncompressed
#[derive(Debug)]
pub enum CompressedMessage {
    Compressed(Vec<u8>),
    Uncompressed(String),
}

impl CompressedMessage {
    pub fn is_compressed(&self) -> bool {
        matches!(self, CompressedMessage::Compressed(_))
    }

    pub fn size(&self) -> usize {
        match self {
            CompressedMessage::Compressed(data) => data.len(),
            CompressedMessage::Uncompressed(text) => text.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        // Use a larger, more repetitive message that compresses well
        let data = b"Hello, World! This is a test message that should compress well. ".repeat(10);
        let compressed = compress(&data, 6).unwrap();

        // Compressed should be smaller for repetitive data
        assert!(compressed.len() < data.len(),
            "Compressed size {} should be less than original size {}",
            compressed.len(), data.len());

        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(data, decompressed.as_slice());
    }

    #[test]
    fn test_should_compress() {
        let config = CompressionConfig::default();
        assert!(!should_compress(&config, 512)); // Below threshold
        assert!(should_compress(&config, 2048)); // Above threshold
    }

    #[test]
    fn test_disabled_compression() {
        let config = CompressionConfig::disabled();
        assert!(!should_compress(&config, 10000));
    }

    #[test]
    fn test_compress_message_small() {
        let config = CompressionConfig::default();
        let small_json = r#"{"type":"ping"}"#;

        let result = compress_message(small_json, &config).unwrap();
        assert!(!result.is_compressed()); // Too small, stays uncompressed
    }

    #[test]
    fn test_compress_message_large() {
        let config = CompressionConfig::default();
        // Create a large JSON message (> 1KB)
        let large_json = format!(
            r#"{{"type":"change","data":{}}}"#,
            "x".repeat(2000)
        );

        let result = compress_message(&large_json, &config).unwrap();
        assert!(result.is_compressed()); // Large enough to compress
        assert!(result.size() < large_json.len()); // Should be smaller
    }

    #[test]
    fn test_compression_config_level_capped() {
        let config = CompressionConfig::new(true, 1024, 15); // Try level 15 (invalid)
        assert_eq!(config.level, 9); // Should be capped at 9
    }
}
