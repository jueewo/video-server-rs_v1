use dashmap::DashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::process::{Child, Command};
use tracing::{debug, info, warn};

/// A running Bun sidecar process.
struct AppSidecar {
    port: u16,
    process: Child,
    last_request: Instant,
    #[allow(dead_code)]
    app_dir: PathBuf,
}

/// Manages lifecycle of Bun sidecar processes for runtime apps.
pub struct SidecarManager {
    sidecars: DashMap<String, AppSidecar>,
    http: reqwest::Client,
}

impl SidecarManager {
    pub fn new() -> Self {
        Self {
            sidecars: DashMap::new(),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("failed to create HTTP client"),
        }
    }

    /// Build the DashMap key from workspace + folder.
    fn key(workspace_id: &str, folder: &str) -> String {
        format!("{}/{}", workspace_id, folder)
    }

    /// Ensure a sidecar is running for the given app. Returns the port.
    pub async fn ensure_running(
        &self,
        workspace_id: &str,
        folder: &str,
        app_dir: &Path,
    ) -> anyhow::Result<u16> {
        let key = Self::key(workspace_id, folder);

        // Fast path: already running
        if let Some(mut entry) = self.sidecars.get_mut(&key) {
            // Check if process is still alive
            match entry.process.try_wait() {
                Ok(Some(status)) => {
                    warn!("Sidecar {} exited with {}, will respawn", key, status);
                    // Fall through to spawn
                }
                Ok(None) => {
                    // Still running
                    entry.last_request = Instant::now();
                    return Ok(entry.port);
                }
                Err(e) => {
                    warn!("Failed to check sidecar {} status: {}, will respawn", key, e);
                }
            }
            // Drop the entry ref before removing
            drop(entry);
            self.sidecars.remove(&key);
        }

        // Spawn new sidecar
        let (child, port) = self.spawn_sidecar(app_dir).await?;

        info!("Sidecar started for {} on port {}", key, port);

        self.sidecars.insert(
            key,
            AppSidecar {
                port,
                process: child,
                last_request: Instant::now(),
                app_dir: app_dir.to_path_buf(),
            },
        );

        Ok(port)
    }

    /// Spawn a Bun process for the app.
    async fn spawn_sidecar(&self, app_dir: &Path) -> anyhow::Result<(Child, u16)> {
        // Find an available port
        let port = Self::find_available_port().await?;

        // Determine the entry point
        let entry = if app_dir.join("server.ts").exists() {
            "server.ts"
        } else if app_dir.join("server.js").exists() {
            "server.js"
        } else {
            anyhow::bail!("No server.ts or server.js found in {}", app_dir.display());
        };

        // Install dependencies if package.json exists and node_modules doesn't
        if app_dir.join("package.json").exists() && !app_dir.join("node_modules").exists() {
            info!("Installing dependencies for {}", app_dir.display());
            let install = Command::new("bun")
                .arg("install")
                .current_dir(app_dir)
                .output()
                .await?;
            if !install.status.success() {
                let stderr = String::from_utf8_lossy(&install.stderr);
                anyhow::bail!("bun install failed: {}", stderr);
            }
        }

        let child = Command::new("bun")
            .arg("run")
            .arg(entry)
            .arg(format!("--port={}", port))
            .current_dir(app_dir)
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // Wait for the sidecar to become ready
        self.wait_for_ready(port).await?;

        Ok((child, port))
    }

    /// Find a random available TCP port.
    async fn find_available_port() -> anyhow::Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        drop(listener);
        Ok(port)
    }

    /// Poll the sidecar's /health endpoint until it responds.
    async fn wait_for_ready(&self, port: u16) -> anyhow::Result<()> {
        let url = format!("http://127.0.0.1:{}/health", port);
        let deadline = Instant::now() + Duration::from_secs(10);

        while Instant::now() < deadline {
            match self.http.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    debug!("Sidecar on port {} is ready", port);
                    return Ok(());
                }
                _ => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }

        anyhow::bail!("Sidecar on port {} failed to become ready within 10s", port)
    }

    /// Stop a specific sidecar.
    pub async fn stop(&self, workspace_id: &str, folder: &str) {
        let key = Self::key(workspace_id, folder);
        if let Some((_, mut sidecar)) = self.sidecars.remove(&key) {
            info!("Stopping sidecar {}", key);
            let _ = sidecar.process.kill().await;
        }
    }

    /// Clean up sidecars that have been idle for longer than `max_idle`.
    pub async fn cleanup_idle(&self, max_idle: Duration) {
        let now = Instant::now();
        let mut to_remove = Vec::new();

        for entry in self.sidecars.iter() {
            if now.duration_since(entry.last_request) > max_idle {
                to_remove.push(entry.key().clone());
            }
        }

        for key in to_remove {
            if let Some((_, mut sidecar)) = self.sidecars.remove(&key) {
                info!("Stopping idle sidecar: {} (port {})", key, sidecar.port);
                let _ = sidecar.process.kill().await;
            }
        }
    }

    /// Get the HTTP client for proxying requests.
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http
    }

    /// Number of active sidecars.
    pub fn active_count(&self) -> usize {
        self.sidecars.len()
    }

    /// List active sidecars for debugging.
    pub fn list_active(&self) -> Vec<(String, u16)> {
        self.sidecars
            .iter()
            .map(|e| (e.key().clone(), e.port))
            .collect()
    }
}

impl Drop for SidecarManager {
    fn drop(&mut self) {
        // Best-effort kill of all sidecars on shutdown
        for mut entry in self.sidecars.iter_mut() {
            let _ = entry.process.start_kill();
        }
        if !self.sidecars.is_empty() {
            info!("Killed {} sidecars on shutdown", self.sidecars.len());
        }
    }
}
