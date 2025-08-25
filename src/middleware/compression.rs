use crate::config::site::CompressionConfig;
use brotli::enc::BrotliEncoderParams;
use bytes::Bytes;
use flate2::write::{DeflateEncoder, GzEncoder};
use flate2::Compression;
use std::io::Write;

#[derive(Debug, Clone)]
pub enum CompressionMethod {
    None,
    Gzip,
    Deflate,
    Brotli,
}

impl CompressionMethod {
    pub fn from_accept_encoding(accept_encoding: &str) -> Self {
        // Parse Accept-Encoding header and choose best compression method
        let encoding = accept_encoding.to_lowercase();

        // Priority order: brotli > gzip > deflate
        if encoding.contains("br") {
            CompressionMethod::Brotli
        } else if encoding.contains("gzip") {
            CompressionMethod::Gzip
        } else if encoding.contains("deflate") {
            CompressionMethod::Deflate
        } else {
            CompressionMethod::None
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CompressionMethod::None => "",
            CompressionMethod::Gzip => "gzip",
            CompressionMethod::Deflate => "deflate",
            CompressionMethod::Brotli => "br",
        }
    }
}

pub struct CompressionMiddleware {
    config: CompressionConfig,
}

impl CompressionMiddleware {
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// Check if content should be compressed based on content type and size
    pub fn should_compress(&self, content_type: &str, content_length: usize) -> bool {
        if !self.config.enabled {
            return false;
        }

        // Check minimum size
        if content_length < self.config.min_size {
            return false;
        }

        // Check if content type is in the list of compressible types
        self.config
            .types
            .iter()
            .any(|t| content_type.to_lowercase().starts_with(&t.to_lowercase()))
    }

    /// Compress content using the specified method
    pub fn compress(
        &self,
        content: &[u8],
        method: CompressionMethod,
    ) -> Result<Bytes, Box<dyn std::error::Error>> {
        match method {
            CompressionMethod::None => Ok(Bytes::copy_from_slice(content)),
            CompressionMethod::Gzip => self.compress_gzip(content),
            CompressionMethod::Deflate => self.compress_deflate(content),
            CompressionMethod::Brotli => self.compress_brotli(content),
        }
    }

    fn compress_gzip(&self, content: &[u8]) -> Result<Bytes, Box<dyn std::error::Error>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.config.level));
        encoder.write_all(content)?;
        let compressed = encoder.finish()?;
        Ok(Bytes::from(compressed))
    }

    fn compress_deflate(&self, content: &[u8]) -> Result<Bytes, Box<dyn std::error::Error>> {
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::new(self.config.level));
        encoder.write_all(content)?;
        let compressed = encoder.finish()?;
        Ok(Bytes::from(compressed))
    }

    fn compress_brotli(&self, content: &[u8]) -> Result<Bytes, Box<dyn std::error::Error>> {
        let params = BrotliEncoderParams {
            quality: self.config.level as i32,
            ..Default::default()
        };

        let mut compressed = Vec::new();
        let mut brotli_encoder = brotli::CompressorWriter::with_params(
            &mut compressed,
            4096, // buffer size
            &params,
        );

        brotli_encoder.write_all(content)?;
        brotli_encoder.flush()?;
        drop(brotli_encoder); // Ensure compression is finalized

        Ok(Bytes::from(compressed))
    }

    /// Get the best compression method based on Accept-Encoding header
    pub fn get_best_compression(&self, accept_encoding: Option<&str>) -> CompressionMethod {
        match accept_encoding {
            Some(encoding) => CompressionMethod::from_accept_encoding(encoding),
            None => CompressionMethod::None,
        }
    }

    /// Check if a content type is compressible
    pub fn is_compressible_type(&self, content_type: &str) -> bool {
        self.config
            .types
            .iter()
            .any(|t| content_type.to_lowercase().starts_with(&t.to_lowercase()))
    }
}

/// Streaming compression wrapper for large files
pub struct StreamingCompressor {
    method: CompressionMethod,
    level: u32,
}

impl StreamingCompressor {
    pub fn new(method: CompressionMethod, level: u32) -> Self {
        Self { method, level }
    }

    /// Create a compressor writer for streaming compression
    pub fn create_writer<W: Write + 'static>(&self, writer: W) -> Box<dyn Write> {
        match self.method {
            CompressionMethod::None => Box::new(writer),
            CompressionMethod::Gzip => {
                Box::new(GzEncoder::new(writer, Compression::new(self.level)))
            }
            CompressionMethod::Deflate => {
                Box::new(DeflateEncoder::new(writer, Compression::new(self.level)))
            }
            CompressionMethod::Brotli => {
                let params = BrotliEncoderParams {
                    quality: self.level as i32,
                    ..Default::default()
                };
                Box::new(brotli::CompressorWriter::with_params(
                    writer, 4096, // buffer size
                    &params,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> CompressionConfig {
        CompressionConfig {
            enabled: true,
            types: vec![
                "text/html".to_string(),
                "text/css".to_string(),
                "application/javascript".to_string(),
                "application/json".to_string(),
            ],
            level: 6,
            min_size: 1024,
        }
    }

    #[test]
    fn test_compression_method_from_accept_encoding() {
        // Test brotli preference
        assert!(matches!(
            CompressionMethod::from_accept_encoding("gzip, deflate, br"),
            CompressionMethod::Brotli
        ));

        // Test gzip fallback
        assert!(matches!(
            CompressionMethod::from_accept_encoding("gzip, deflate"),
            CompressionMethod::Gzip
        ));

        // Test deflate fallback
        assert!(matches!(
            CompressionMethod::from_accept_encoding("deflate"),
            CompressionMethod::Deflate
        ));

        // Test no compression
        assert!(matches!(
            CompressionMethod::from_accept_encoding("identity"),
            CompressionMethod::None
        ));
    }

    #[test]
    fn test_should_compress() {
        let middleware = CompressionMiddleware::new(create_test_config());

        // Should compress HTML content above min size
        assert!(middleware.should_compress("text/html", 2048));

        // Should not compress below min size
        assert!(!middleware.should_compress("text/html", 512));

        // Should not compress non-text content
        assert!(!middleware.should_compress("image/png", 2048));

        // Should compress JSON content
        assert!(middleware.should_compress("application/json", 2048));
    }

    #[test]
    fn test_gzip_compression() {
        let middleware = CompressionMiddleware::new(create_test_config());
        // Use larger, more repetitive test data that compresses well
        let test_data = b"Hello, World! This is a test string for compression. ".repeat(100);

        let compressed = middleware
            .compress(&test_data, CompressionMethod::Gzip)
            .unwrap();

        // Compressed data should be different and smaller for this larger test string
        assert_ne!(compressed.as_ref(), test_data.as_slice());
        assert!(compressed.len() < test_data.len());
    }

    #[test]
    fn test_brotli_compression() {
        let middleware = CompressionMiddleware::new(create_test_config());
        let test_data = b"Hello, World! This is a test string for compression. ".repeat(100);

        let compressed = middleware
            .compress(&test_data, CompressionMethod::Brotli)
            .unwrap();

        // Compressed data should be different and smaller
        assert_ne!(compressed.as_ref(), test_data.as_slice());
        assert!(compressed.len() < test_data.len());
    }

    #[test]
    fn test_compression_disabled() {
        let mut config = create_test_config();
        config.enabled = false;
        let middleware = CompressionMiddleware::new(config);

        // Should not compress when disabled
        assert!(!middleware.should_compress("text/html", 2048));
    }

    #[test]
    fn test_is_compressible_type() {
        let middleware = CompressionMiddleware::new(create_test_config());

        assert!(middleware.is_compressible_type("text/html"));
        assert!(middleware.is_compressible_type("text/html; charset=utf-8"));
        assert!(middleware.is_compressible_type("application/json"));
        assert!(!middleware.is_compressible_type("image/png"));
        assert!(!middleware.is_compressible_type("video/mp4"));
    }

    #[test]
    fn test_best_compression_selection() {
        let middleware = CompressionMiddleware::new(create_test_config());

        // Test various Accept-Encoding headers
        assert!(matches!(
            middleware.get_best_compression(Some("gzip, deflate, br")),
            CompressionMethod::Brotli
        ));

        assert!(matches!(
            middleware.get_best_compression(Some("gzip, deflate")),
            CompressionMethod::Gzip
        ));

        assert!(matches!(
            middleware.get_best_compression(Some("deflate")),
            CompressionMethod::Deflate
        ));

        assert!(matches!(
            middleware.get_best_compression(None),
            CompressionMethod::None
        ));
    }
}
