use log::{error, info, warn};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::sync::mpsc;

pub struct CertificateWatcher {
    cert_dir: String,
    domains: Vec<String>,
    _watcher: Option<RecommendedWatcher>, // Keep watcher alive
}

impl CertificateWatcher {
    pub fn new(cert_dir: String, domains: Vec<String>) -> Self {
        Self {
            cert_dir,
            domains,
            _watcher: None,
        }
    }

    pub fn start_watching(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (tx, mut rx) = mpsc::unbounded_channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    if let Err(e) = tx.send(event) {
                        error!("Failed to send file watcher event: {}", e);
                    }
                }
                Err(e) => {
                    error!("File watcher error: {}", e);
                }
            },
            notify::Config::default(),
        )?;

        // Watch the certificate directory
        watcher.watch(Path::new(&self.cert_dir), RecursiveMode::Recursive)?;
        info!(
            "Certificate watcher started for directory: {}",
            self.cert_dir
        );

        // Store watcher to keep it alive
        self._watcher = Some(watcher);

        let domains = self.domains.clone();
        let cert_dir = self.cert_dir.clone();

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        for path in event.paths {
                            if let Some(filename) = path.file_name() {
                                if let Some(filename_str) = filename.to_str() {
                                    // Check if this is a certificate file for one of our domains
                                    for domain in &domains {
                                        let cert_file = format!("{}.crt", domain);
                                        let key_file = format!("{}.key", domain);

                                        if filename_str == cert_file || filename_str == key_file {
                                            info!("Certificate file changed: {:?}", path);

                                            // Check if both cert and key exist now
                                            let cert_path = Path::new(&cert_dir).join(&cert_file);
                                            let key_path = Path::new(&cert_dir).join(&key_file);

                                            if cert_path.exists() && key_path.exists() {
                                                info!("Both certificate and key files exist for domain: {}", domain);
                                                warn!("🔄 HTTPS UPGRADE AVAILABLE!");
                                                warn!(
                                                    "To enable HTTPS for {}, restart the server:",
                                                    domain
                                                );
                                                warn!("   pkill -f bws");
                                                warn!("   ./target/x86_64-unknown-linux-musl/release/bws --config config-auto-acme.toml");
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            info!("Certificate watcher task terminated");
        });

        Ok(())
    }
}
