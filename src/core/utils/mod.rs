//! Utility functions for BWS Web Server
//!
//! This module provides common utility functions used throughout
//! the application for string manipulation, file operations, etc.

use crate::core::error::{BwsError, BwsResult};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// String manipulation utilities
pub mod string {
    /// Sanitize a string for use in file paths or URLs
    pub fn sanitize_path_component(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
            .collect()
    }

    /// Convert bytes to human-readable size
    pub fn humanize_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        const THRESHOLD: u64 = 1024;

        if bytes < THRESHOLD {
            return format!("{} B", bytes);
        }

        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
            size /= THRESHOLD as f64;
            unit_index += 1;
        }

        format!("{:.1} {}", size, UNITS[unit_index])
    }

    /// Parse human-readable size to bytes
    pub fn parse_size(input: &str) -> Option<u64> {
        let input = input.trim().to_uppercase();
        
        if let Ok(bytes) = input.parse::<u64>() {
            return Some(bytes);
        }

        let (number_str, unit) = if input.ends_with("KB") {
            (input.strip_suffix("KB")?, 1024)
        } else if input.ends_with("MB") {
            (input.strip_suffix("MB")?, 1024 * 1024)
        } else if input.ends_with("GB") {
            (input.strip_suffix("GB")?, 1024 * 1024 * 1024)
        } else if input.ends_with("TB") {
            (input.strip_suffix("TB")?, 1024_u64.pow(4))
        } else if input.ends_with("B") {
            (input.strip_suffix("B")?, 1)
        } else {
            return None;
        };

        let number: u64 = number_str.parse().ok()?;
        Some(number * unit)
    }

    /// Format duration in human-readable format
    pub fn humanize_duration(duration: std::time::Duration) -> String {
        let total_seconds = duration.as_secs();
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if days > 0 {
            format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
        } else if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }
}

/// File system utilities
pub mod fs {
    use super::*;

    /// Safely normalize a file path to prevent directory traversal
    pub fn normalize_path<P: AsRef<Path>>(path: P) -> BwsResult<PathBuf> {
        let path = path.as_ref();
        let mut normalized = PathBuf::new();

        for component in path.components() {
            match component {
                std::path::Component::Normal(name) => {
                    // Check for dangerous filenames
                    let name_str = name.to_string_lossy();
                    if name_str.contains('\0') {
                        return Err(BwsError::Validation(
                            "Path contains null byte".to_string()
                        ));
                    }
                    normalized.push(name);
                }
                std::path::Component::CurDir => {
                    // Skip current directory references
                    continue;
                }
                std::path::Component::ParentDir => {
                    // Remove parent directory references for security
                    normalized.pop();
                }
                _ => {
                    // Skip other component types (Prefix, RootDir)
                    continue;
                }
            }
        }

        Ok(normalized)
    }

    /// Check if a file extension is allowed for static serving
    pub fn is_safe_extension(extension: &str) -> bool {
        const SAFE_EXTENSIONS: &[&str] = &[
            "html", "htm", "css", "js", "json", "xml",
            "txt", "md", "pdf", "doc", "docx",
            "jpg", "jpeg", "png", "gif", "svg", "webp",
            "mp3", "mp4", "wav", "avi", "mov",
            "zip", "tar", "gz", "woff", "woff2", "ttf",
            "ico", "manifest", "map", "wasm"
        ];

        SAFE_EXTENSIONS.contains(&extension.to_lowercase().as_str())
    }

    /// Get MIME type for file extension
    pub fn get_mime_type(extension: &str) -> &'static str {
        match extension.to_lowercase().as_str() {
            "html" | "htm" => "text/html; charset=utf-8",
            "css" => "text/css; charset=utf-8",
            "js" => "application/javascript; charset=utf-8",
            "json" => "application/json; charset=utf-8",
            "xml" => "application/xml; charset=utf-8",
            "txt" => "text/plain; charset=utf-8",
            "md" => "text/markdown; charset=utf-8",
            "pdf" => "application/pdf",
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "ico" => "image/x-icon",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "ttf" => "font/ttf",
            "wasm" => "application/wasm",
            "manifest" => "application/manifest+json",
            "map" => "application/json",
            _ => "application/octet-stream",
        }
    }
}

/// Time utilities
pub mod time {
    use super::*;

    /// Get current Unix timestamp
    pub fn unix_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Format timestamp as ISO 8601 string
    pub fn format_iso8601(timestamp: SystemTime) -> String {
        let datetime = chrono::DateTime::<chrono::Utc>::from(timestamp);
        datetime.to_rfc3339()
    }

    /// Parse ISO 8601 string to SystemTime
    pub fn parse_iso8601(input: &str) -> Option<SystemTime> {
        chrono::DateTime::parse_from_rfc3339(input)
            .ok()
            .map(|dt| dt.into())
    }
}

/// Network utilities
pub mod net {
    use std::net::{IpAddr, SocketAddr};

    /// Check if an IP address is in a private range
    pub fn is_private_ip(ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                ipv4.is_private() || ipv4.is_loopback() || ipv4.is_link_local()
            }
            IpAddr::V6(ipv6) => {
                ipv6.is_loopback() || ((ipv6.segments()[0] & 0xfe00) == 0xfc00)
            }
        }
    }

    /// Extract client IP from various sources
    pub fn extract_client_ip(
        socket_addr: &SocketAddr,
        x_forwarded_for: Option<&str>,
        x_real_ip: Option<&str>,
    ) -> IpAddr {
        // Check X-Real-IP header first
        if let Some(real_ip) = x_real_ip {
            if let Ok(ip) = real_ip.parse() {
                return ip;
            }
        }

        // Check X-Forwarded-For header
        if let Some(forwarded) = x_forwarded_for {
            if let Some(first_ip) = forwarded.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse() {
                    return ip;
                }
            }
        }

        // Fall back to socket address
        socket_addr.ip()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_humanize_bytes() {
        assert_eq!(string::humanize_bytes(512), "512 B");
        assert_eq!(string::humanize_bytes(1024), "1.0 KB");
        assert_eq!(string::humanize_bytes(1536), "1.5 KB");
        assert_eq!(string::humanize_bytes(1048576), "1.0 MB");
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(string::parse_size("1024"), Some(1024));
        assert_eq!(string::parse_size("1KB"), Some(1024));
        assert_eq!(string::parse_size("1MB"), Some(1024 * 1024));
        assert_eq!(string::parse_size("invalid"), None);
    }

    #[test]
    fn test_normalize_path() {
        let path = fs::normalize_path("../test/../file.txt").unwrap();
        assert_eq!(path, PathBuf::from("file.txt"));
        
        let path = fs::normalize_path("./folder/./file.txt").unwrap();
        assert_eq!(path, PathBuf::from("folder/file.txt"));
    }

    #[test]
    fn test_safe_extension() {
        assert!(fs::is_safe_extension("html"));
        assert!(fs::is_safe_extension("css"));
        assert!(fs::is_safe_extension("js"));
        assert!(!fs::is_safe_extension("exe"));
        assert!(!fs::is_safe_extension("sh"));
    }
}
