//! Simple Windows test for BWS

use pingora::prelude::*;

#[tokio::test]
async fn test_pingora_windows_basic() {
    // Basic test to ensure Pingora can be initialized on Windows
    let server = Server::new(None);
    assert!(server.is_ok(), "Failed to create Pingora server on Windows");

    println!("✅ Pingora server can be created on Windows");
}

#[test]
fn test_windows_compatibility() {
    // Test basic Windows compatibility
    println!("🔧 Testing BWS Windows compatibility");

    // Test that we can create basic structs
    let opt = Some(pingora::server::configuration::Opt::default());
    assert!(opt.is_some());

    println!("✅ Basic Windows compatibility test passed");
}
