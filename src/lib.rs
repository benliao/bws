pub mod config;
pub mod handlers;
pub mod server;
pub mod ssl;

// Re-export main types for convenience
pub use config::{ServerConfig, SiteConfig};
pub use server::WebServerService;
pub use ssl::{AcmeConfig, SslManager};

// Legacy compatibility exports
use std::fs;

// Legacy function for backward compatibility
pub fn read_file_bytes(file_path: &str) -> std::io::Result<Vec<u8>> {
    fs::read(file_path)
}

// Legacy function for backward compatibility
pub fn get_mime_type(file_path: &str) -> &'static str {
    let path = std::path::Path::new(file_path);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") | Some("mjs") => "application/javascript",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("txt") => "text/plain",
        Some("pdf") => "application/pdf",
        Some("woff") | Some("woff2") => "font/woff",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        Some("eot") => "application/vnd.ms-fontobject",
        Some("xml") => "application/xml",
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
        Some("md") => "text/markdown",
        Some("yaml") | Some("yml") => "application/x-yaml",
        Some("toml") => "application/toml",
        _ => "application/octet-stream",
    }
}
