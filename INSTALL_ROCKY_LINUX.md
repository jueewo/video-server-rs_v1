# Installation Guide - Rocky Linux

Quick setup guide for Rocky Linux 8/9.

## System Dependencies

```bash
# Enable EPEL and PowerTools repositories
sudo dnf install -y epel-release
sudo dnf config-manager --set-enabled powertools  # Rocky 8
# or
sudo dnf config-manager --set-enabled crb         # Rocky 9

# Install required tools
sudo dnf install -y \
  ffmpeg \
  ffmpeg-libs \
  ghostscript \
  libwebp-tools \
  sqlite \
  sqlite-libs \
  wget \
  curl
```

## MediaMTX Installation

```bash
# Download latest release
wget https://github.com/bluenviron/mediamtx/releases/latest/download/mediamtx_v1.5.1_linux_amd64.tar.gz

# Extract and install
tar -xzf mediamtx_v1.5.1_linux_amd64.tar.gz
sudo mv mediamtx /usr/local/bin/
sudo chmod +x /usr/local/bin/mediamtx

# Clean up
rm mediamtx_v1.5.1_linux_amd64.tar.gz
```

## Rust Installation

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify
rustc --version
cargo --version
```

## Verification

```bash
# Check all tools are installed
ffmpeg -version
gs --version
cwebp -version
sqlite3 --version
mediamtx --version
```

## Firewall Configuration

```bash
# Allow required ports
sudo firewall-cmd --permanent --add-port=3000/tcp   # Rust server
sudo firewall-cmd --permanent --add-port=1935/tcp   # RTMP
sudo firewall-cmd --permanent --add-port=8888/tcp   # HLS
sudo firewall-cmd --permanent --add-port=8889/tcp   # WebRTC
sudo firewall-cmd --reload
```

## SELinux (if enabled)

```bash
# Allow network connections
sudo setsebool -P httpd_can_network_connect 1

# Or disable SELinux (not recommended for production)
sudo setenforce 0
```

## Build and Run

```bash
# Clone repository
git clone <your-repo-url>
cd video-server-rs_v1

# Build CSS
npm install
npm run build:css

# Build Rust
cargo build --release

# Run
./target/release/video-server-rs
```

## SystemD Service (Optional)

Create `/etc/systemd/system/video-server.service`:

```ini
[Unit]
Description=Video Server RS
After=network.target

[Service]
Type=simple
User=mediaserver
WorkingDirectory=/opt/video-server-rs_v1
ExecStart=/opt/video-server-rs_v1/target/release/video-server-rs
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable video-server
sudo systemctl start video-server
sudo systemctl status video-server
```

## Troubleshooting

### FFmpeg Not Found

```bash
# Install RPM Fusion repository
sudo dnf install -y --nogpgcheck \
  https://mirrors.rpmfusion.org/free/el/rpmfusion-free-release-$(rpm -E %rhel).noarch.rpm
sudo dnf install -y ffmpeg
```

### WebP Tools Missing

```bash
# Install from EPEL
sudo dnf install -y libwebp libwebp-tools
```

### Port Already in Use

```bash
# Find process using port
sudo lsof -i :3000

# Kill if needed
sudo kill -9 <PID>
```

## Production Checklist

- [ ] All dependencies installed and verified
- [ ] Firewall configured
- [ ] SELinux configured
- [ ] SystemD service created
- [ ] Logs configured (`journalctl -u video-server -f`)
- [ ] Backup strategy for `media.db`
- [ ] HTTPS configured (Caddy or Nginx)
- [ ] Monitoring configured

## Package Versions (Rocky 9)

- FFmpeg: 6.0+
- Ghostscript: 9.54+
- WebP: 1.2+
- SQLite: 3.34+

## Support

- Rocky Linux docs: https://docs.rockylinux.org/
- EPEL repository: https://docs.fedoraproject.org/en-US/epel/
- FFmpeg on RHEL: https://rpmfusion.org/
