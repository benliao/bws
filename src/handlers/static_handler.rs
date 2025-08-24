use crate::config::SiteConfig;
use pingora::http::ResponseHeader;
use pingora::prelude::*;
use std::path::Path;
use tokio::fs;

pub struct StaticFileHandler {
    // Future: Add caching, compression, etc.
}

impl StaticFileHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn handle(&self, session: &mut Session, site: &SiteConfig, path: &str) -> Result<()> {
        let file_path = self.resolve_file_path(site, path).await;

        match file_path {
            Some(resolved_path) => self.serve_file(session, site, &resolved_path).await,
            None => self.handle_not_found(session, site).await,
        }
    }

    async fn resolve_file_path(&self, site: &SiteConfig, request_path: &str) -> Option<String> {
        let clean_path = self.clean_path(request_path);

        // Security check: ensure the path is safe before proceeding
        if !self.is_path_safe(&site.static_dir, &clean_path) {
            log::warn!("Blocked path traversal attempt: {}", request_path);
            return None;
        }

        // Try exact path first
        let file_path = format!("{}/{}", site.static_dir, clean_path);
        if self.is_file_accessible(&file_path).await {
            return Some(file_path);
        }

        // If path ends with '/', try index files
        if clean_path.ends_with('/') || clean_path.is_empty() {
            for index_file in site.get_index_files() {
                let index_path = format!("{}/{}{}", site.static_dir, clean_path, index_file);
                if self.is_file_accessible(&index_path).await {
                    return Some(index_path);
                }
            }
        } else {
            // Try adding '/' and looking for index files
            for index_file in site.get_index_files() {
                let index_path = format!("{}/{}/{}", site.static_dir, clean_path, index_file);
                if self.is_file_accessible(&index_path).await {
                    return Some(index_path);
                }
            }
        }

        None
    }

    fn clean_path(&self, path: &str) -> String {
        // Remove query parameters and fragments
        let path = path.split('?').next().unwrap_or(path);
        let path = path.split('#').next().unwrap_or(path);

        // Handle root path
        if path == "/" {
            return "".to_string();
        }

        // Remove leading slash for joining with static_dir
        let clean = path.strip_prefix('/').unwrap_or(path);
        
        // Normalize path separators and remove dangerous sequences
        let clean = clean.replace('\\', "/"); // Normalize Windows paths
        let clean = clean.replace("//", "/"); // Remove double slashes
        
        // Split path and filter out dangerous components
        let components: Vec<&str> = clean
            .split('/')
            .filter(|component| {
                !component.is_empty() 
                && *component != "." 
                && *component != ".."
                && !component.contains('\0') // Null byte injection protection
            })
            .collect();
            
        components.join("/")
    }

    async fn is_file_accessible(&self, file_path: &str) -> bool {
        let path = Path::new(file_path);

        // Check if file exists and is a regular file
        if let Ok(metadata) = fs::metadata(path).await {
            if metadata.is_file() {
                // Additional security check: file size limit (100MB max)
                const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;
                if metadata.len() > MAX_FILE_SIZE {
                    log::warn!("File too large, rejecting: {} ({} bytes)", file_path, metadata.len());
                    return false;
                }
                return true;
            }
        }

        false
    }

    async fn serve_file(
        &self,
        session: &mut Session,
        site: &SiteConfig,
        file_path: &str,
    ) -> Result<()> {
        match fs::read(file_path).await {
            Ok(content) => {
                let mime_type = self.get_mime_type(file_path);
                let mut header = ResponseHeader::build(200, Some(4))?;

                // Basic headers
                header.insert_header("Content-Type", mime_type)?;
                header.insert_header("Content-Length", content.len().to_string())?;

                // Cache headers
                let is_static = self.is_static_file(file_path);
                for (key, value) in site.get_cache_headers(is_static) {
                    header.insert_header(key, value)?;
                }

                // CORS headers
                for (key, value) in site.get_cors_headers() {
                    header.insert_header(key, value)?;
                }

                // Custom site headers
                for (key, value) in &site.headers {
                    header.insert_header(key.clone(), value.clone())?;
                }

                // Check if content should be compressed
                let content_len = content.len();
                let should_compress = site.should_compress(mime_type, content_len);
                let final_content = if should_compress {
                    // TODO: Implement compression
                    // For now, just serve uncompressed
                    content.clone()
                } else {
                    content.clone()
                };

                session
                    .write_response_header(Box::new(header), false)
                    .await?;
                session
                    .write_response_body(Some(final_content.into()), true)
                    .await?;

                log::debug!("Served file: {} ({} bytes)", file_path, content_len);
            }
            Err(e) => {
                log::warn!("Failed to read file {}: {}", file_path, e);
                self.handle_not_found(session, site).await?;
            }
        }

        Ok(())
    }

    async fn handle_not_found(&self, session: &mut Session, site: &SiteConfig) -> Result<()> {
        // Check if site has custom 404 page
        if let Some(error_page) = site.get_error_page(404) {
            let error_page_path = format!("{}/{}", site.static_dir, error_page);
            if let Ok(content) = fs::read(&error_page_path).await {
                let mut header = ResponseHeader::build(404, Some(3))?;
                header.insert_header("Content-Type", "text/html")?;
                header.insert_header("Content-Length", content.len().to_string())?;

                // Custom site headers
                for (key, value) in &site.headers {
                    header.insert_header(key.clone(), value.clone())?;
                }

                session
                    .write_response_header(Box::new(header), false)
                    .await?;
                session
                    .write_response_body(Some(content.into()), true)
                    .await?;
                return Ok(());
            }
        }

        // Default 404 response
        let error_html = r#"<!DOCTYPE html>
<html>
<head>
    <title>404 Not Found</title>
    <style>
        body { font-family: Arial, sans-serif; text-align: center; margin-top: 100px; }
        h1 { color: #666; }
        p { color: #999; }
    </style>
</head>
<body>
    <h1>404 Not Found</h1>
    <p>The requested resource was not found on this server.</p>
</body>
</html>"#;

        let mut header = ResponseHeader::build(404, Some(3))?;
        header.insert_header("Content-Type", "text/html")?;
        header.insert_header("Content-Length", error_html.len().to_string())?;

        // Custom site headers
        for (key, value) in &site.headers {
            header.insert_header(key.clone(), value.clone())?;
        }

        session
            .write_response_header(Box::new(header), false)
            .await?;
        session
            .write_response_body(Some(error_html.as_bytes().to_vec().into()), true)
            .await?;

        Ok(())
    }

    fn get_mime_type(&self, file_path: &str) -> &'static str {
        let path = Path::new(file_path);
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("html") | Some("htm") => "text/html; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("js") | Some("mjs") => "application/javascript; charset=utf-8",
            Some("json") => "application/json; charset=utf-8",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("txt") => "text/plain; charset=utf-8",
            Some("pdf") => "application/pdf",
            Some("woff") | Some("woff2") => "font/woff",
            Some("ttf") => "font/ttf",
            Some("otf") => "font/otf",
            Some("eot") => "application/vnd.ms-fontobject",
            Some("xml") => "application/xml; charset=utf-8",
            Some("wasm") => "application/wasm",
            Some("webp") => "image/webp",
            Some("avif") => "image/avif",
            Some("mp4") => "video/mp4",
            Some("webm") => "video/webm",
            Some("mp3") => "audio/mpeg",
            Some("wav") => "audio/wav",
            Some("ogg") => "audio/ogg",
            Some("zip") => "application/zip",
            Some("gz") => "application/gzip",
            Some("tar") => "application/x-tar",
            Some("md") => "text/markdown; charset=utf-8",
            Some("yaml") | Some("yml") => "application/x-yaml; charset=utf-8",
            Some("toml") => "application/toml; charset=utf-8",
            Some("csv") => "text/csv; charset=utf-8",
            Some("manifest") => "text/cache-manifest; charset=utf-8",
            Some("webmanifest") => "application/manifest+json; charset=utf-8",
            Some("rss") => "application/rss+xml; charset=utf-8",
            Some("atom") => "application/atom+xml; charset=utf-8",
            _ => "application/octet-stream",
        }
    }

    fn is_static_file(&self, file_path: &str) -> bool {
        let path = Path::new(file_path);
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
            matches!(
                ext,
                "css"
                    | "js"
                    | "png"
                    | "jpg"
                    | "jpeg"
                    | "gif"
                    | "svg"
                    | "ico"
                    | "woff"
                    | "woff2"
                    | "ttf"
                    | "otf"
                    | "eot"
                    | "pdf"
                    | "zip"
                    | "mp4"
                    | "webm"
                    | "mp3"
                    | "wav"
                    | "ogg"
                    | "webp"
                    | "avif"
            )
        } else {
            false
        }
    }

    // Security function to prevent path traversal
    fn is_path_safe(&self, static_dir: &str, requested_path: &str) -> bool {
        let static_path = Path::new(static_dir);
        let requested_path = static_path.join(requested_path);

        // Canonicalize paths to resolve .. and . components
        if let (Ok(static_canonical), Ok(requested_canonical)) =
            (static_path.canonicalize(), requested_path.canonicalize())
        {
            requested_canonical.starts_with(static_canonical)
        } else {
            // If canonicalization fails, be conservative and reject
            // This handles cases where the path doesn't exist or has permission issues
            let static_abs = match std::fs::canonicalize(static_path) {
                Ok(path) => path,
                Err(_) => return false,
            };
            
            // For non-existent files, check if the parent directory is safe
            let mut check_path = requested_path.clone();
            while let Some(parent) = check_path.parent() {
                if let Ok(parent_canonical) = parent.canonicalize() {
                    return parent_canonical.starts_with(&static_abs);
                }
                check_path = parent.to_path_buf();
            }
            
            false
        }
    }
}

impl Default for StaticFileHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_type_detection() {
        let handler = StaticFileHandler::new();

        assert_eq!(
            handler.get_mime_type("test.html"),
            "text/html; charset=utf-8"
        );
        assert_eq!(handler.get_mime_type("test.css"), "text/css; charset=utf-8");
        assert_eq!(
            handler.get_mime_type("test.js"),
            "application/javascript; charset=utf-8"
        );
        assert_eq!(handler.get_mime_type("test.png"), "image/png");
        assert_eq!(handler.get_mime_type("test.wasm"), "application/wasm");
        assert_eq!(
            handler.get_mime_type("test.unknown"),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_clean_path() {
        let handler = StaticFileHandler::new();

        assert_eq!(handler.clean_path("/"), ""); // Root path becomes empty for joining with static_dir
        assert_eq!(handler.clean_path("/test.html"), "test.html");
        assert_eq!(handler.clean_path("/path/to/file.css"), "path/to/file.css");
        assert_eq!(handler.clean_path("/test.html?query=1"), "test.html");
        assert_eq!(handler.clean_path("/test.html#fragment"), "test.html");
        assert_eq!(
            handler.clean_path("/test.html?query=1#fragment"),
            "test.html"
        );

        // Test security: path traversal protection
        // "../.." components are filtered out, leaving only valid path components
        assert_eq!(handler.clean_path("/../../../etc/passwd"), "etc/passwd");
        assert_eq!(handler.clean_path("/test/../../../secret"), "test/secret"); // ".." filtered out, leaving "test" and "secret"
        assert_eq!(handler.clean_path("./test.html"), "test.html"); // "." filtered out
        assert_eq!(handler.clean_path("/./test/../file.css"), "test/file.css"); // "." and ".." filtered out
        
        // Test null byte injection protection
        assert_eq!(handler.clean_path("/test\0.html"), ""); // Component with null byte is filtered out
        
        // Test double slash normalization
        assert_eq!(handler.clean_path("//test//file.js"), "test/file.js");
        
        // Test Windows path normalization
        assert_eq!(handler.clean_path("/path\\to\\file.css"), "path/to/file.css");
    }

    #[test]
    fn test_is_static_file() {
        let handler = StaticFileHandler::new();

        assert!(handler.is_static_file("test.css"));
        assert!(handler.is_static_file("test.js"));
        assert!(handler.is_static_file("test.png"));
        assert!(handler.is_static_file("test.woff"));
        assert!(!handler.is_static_file("test.html"));
        assert!(!handler.is_static_file("test.json"));
        assert!(!handler.is_static_file("test.unknown"));
    }

    #[test]
    fn test_path_safety() {
        let handler = StaticFileHandler::new();

        // Create a temporary directory for testing
        let temp_dir = std::env::temp_dir().join("bws_test_static");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Create a test file
        let index_file = temp_dir.join("index.html");
        std::fs::write(&index_file, "test content").unwrap();

        // Create assets directory and file
        let assets_dir = temp_dir.join("assets");
        std::fs::create_dir_all(&assets_dir).unwrap();
        let css_file = assets_dir.join("style.css");
        std::fs::write(&css_file, "body { color: black; }").unwrap();

        let static_dir = temp_dir.to_str().unwrap();

        assert!(handler.is_path_safe(static_dir, "index.html"));
        assert!(handler.is_path_safe(static_dir, "assets/style.css"));

        // Clean up
        std::fs::remove_dir_all(&temp_dir).unwrap();
    }
}
