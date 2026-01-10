# Video Server Documentation

**Last Updated:** January 2024

This directory contains comprehensive documentation for the Rust + MediaMTX HLS live streaming server with OIDC authentication.

## 📚 Documentation Structure

```
docs/
├── README.md                           # This file
├── auth/                               # Authentication documentation
│   ├── EMERGENCY_LOGIN.md             # Emergency login feature guide
│   ├── EMERGENCY_LOGIN_QUICKSTART.md  # Quick start for emergency login
│   ├── EMERGENCY_LOGIN_IMPLEMENTATION.md # Implementation details
│   ├── OIDC_QUICKSTART.md             # OIDC quick start guide
│   ├── OIDC_IMPLEMENTATION.md         # OIDC implementation details
│   ├── OIDC_TROUBLESHOOTING.md        # OIDC troubleshooting guide
│   ├── CASDOOR_SETUP.md               # Casdoor setup guide
│   └── CASDOOR_PKCE_GUIDE.md          # PKCE implementation guide
├── features/                           # Feature documentation
│   ├── IMAGE_SERVING.md               # Image serving guide
│   └── IMAGE_QUICKSTART.md            # Image serving quick start
├── architecture/                       # Architecture documentation
│   ├── MODULAR_ARCHITECTURE.md        # Modular architecture overview
│   └── MODULAR_QUICKSTART.md          # Modular architecture quick start
├── LIVE_STREAMING_GUIDE.md            # Complete streaming guide
└── MEDIAMTX_MIGRATION.md              # MediaMTX integration details
```

## 🚀 Quick Start

### For New Users
1. Read [QUICKSTART.md](../QUICKSTART.md) in the root directory
2. Follow [auth/OIDC_QUICKSTART.md](./auth/OIDC_QUICKSTART.md) for authentication
3. See [LIVE_STREAMING_GUIDE.md](./LIVE_STREAMING_GUIDE.md) for streaming setup

### For Production Deployment
1. Configure OIDC: [auth/CASDOOR_SETUP.md](./auth/CASDOOR_SETUP.md)
2. Set up emergency login: [auth/EMERGENCY_LOGIN_QUICKSTART.md](./auth/EMERGENCY_LOGIN_QUICKSTART.md)
3. Review security: [auth/OIDC_IMPLEMENTATION.md](./auth/OIDC_IMPLEMENTATION.md)

## 📖 Documentation by Topic

### 🔐 Authentication

#### OIDC Authentication (Primary)
- **[OIDC Quick Start](./auth/OIDC_QUICKSTART.md)** - Get started with OIDC
- **[Casdoor Setup](./auth/CASDOOR_SETUP.md)** - Configure Casdoor provider
- **[OIDC Implementation](./auth/OIDC_IMPLEMENTATION.md)** - Technical details
- **[PKCE Guide](./auth/CASDOOR_PKCE_GUIDE.md)** - PKCE flow explanation
- **[Troubleshooting](./auth/OIDC_TROUBLESHOOTING.md)** - Fix common issues

#### Emergency Login (Disaster Recovery)
- **[Emergency Login Guide](./auth/EMERGENCY_LOGIN.md)** - Complete documentation
- **[Quick Start](./auth/EMERGENCY_LOGIN_QUICKSTART.md)** - 5-minute setup
- **[Implementation](./auth/EMERGENCY_LOGIN_IMPLEMENTATION.md)** - Technical details

**When to use:**
- OIDC: ✅ Production (always)
- Emergency Login: ⚠️ Only when OIDC is down

### 📺 Live Streaming

- **[Live Streaming Guide](./LIVE_STREAMING_GUIDE.md)** - Complete guide
  - RTMP streaming setup
  - FFmpeg commands
  - Audio configuration
  - Troubleshooting
  - Performance tuning

- **[MediaMTX Migration](./MEDIAMTX_MIGRATION.md)** - Architecture details
  - Integration architecture
  - Advanced configuration
  - WebRTC setup
  - Production deployment

### 🖼️ Image Serving

- **[Image Serving Guide](./features/IMAGE_SERVING.md)** - Complete guide
- **[Quick Start](./features/IMAGE_QUICKSTART.md)** - Get started quickly

### 🏗️ Architecture

- **[Modular Architecture](./architecture/MODULAR_ARCHITECTURE.md)** - System design
- **[Quick Start](./architecture/MODULAR_QUICKSTART.md)** - Understanding modules

## 🎯 Common Tasks

### Setting Up Authentication

```bash
# 1. Configure OIDC in .env
OIDC_ISSUER_URL=http://localhost:8088
OIDC_CLIENT_ID=your-client-id
OIDC_CLIENT_SECRET=your-client-secret
OIDC_REDIRECT_URI=http://localhost:3000/oidc/callback

# 2. Configure emergency login (disabled by default)
ENABLE_EMERGENCY_LOGIN=false
SU_USER=admin
SU_PWD=your-secure-password

# 3. Start the server
cargo run
```

See: [auth/OIDC_QUICKSTART.md](./auth/OIDC_QUICKSTART.md)

### Starting a Live Stream

```bash
# 1. Start MediaMTX
mediamtx mediamtx.yml

# 2. Start server
cargo run

# 3. Stream with OBS or FFmpeg
ffmpeg -f avfoundation -i "0:0" \
  -c:v libx264 -preset ultrafast \
  -c:a aac \
  -f flv rtmp://localhost:1935/live?token=supersecret123
```

See: [LIVE_STREAMING_GUIDE.md](./LIVE_STREAMING_GUIDE.md)

### Emergency Recovery

```bash
# 1. SSH into server
ssh admin@your-server.com

# 2. Enable emergency login
echo "ENABLE_EMERGENCY_LOGIN=true" >> .env

# 3. Restart server
systemctl restart video-server

# 4. Login at /login/emergency
# 5. Fix OIDC issue
# 6. Disable emergency login
# 7. Restart server
```

See: [auth/EMERGENCY_LOGIN_QUICKSTART.md](./auth/EMERGENCY_LOGIN_QUICKSTART.md)

## 🔧 Current Features

| Feature | Status | Documentation |
|---------|--------|---------------|
| **OIDC Authentication** | ✅ | [auth/OIDC_IMPLEMENTATION.md](./auth/OIDC_IMPLEMENTATION.md) |
| **Emergency Login** | ✅ | [auth/EMERGENCY_LOGIN.md](./auth/EMERGENCY_LOGIN.md) |
| **Live RTMP Streaming** | ✅ | [LIVE_STREAMING_GUIDE.md](./LIVE_STREAMING_GUIDE.md) |
| **HLS Output** | ✅ | [LIVE_STREAMING_GUIDE.md](./LIVE_STREAMING_GUIDE.md) |
| **WebRTC Output** | ✅ | [MEDIAMTX_MIGRATION.md](./MEDIAMTX_MIGRATION.md) |
| **Image Serving** | ✅ | [features/IMAGE_SERVING.md](./features/IMAGE_SERVING.md) |
| **Session Management** | ✅ | [auth/OIDC_IMPLEMENTATION.md](./auth/OIDC_IMPLEMENTATION.md) |
| **Modular Architecture** | ✅ | [architecture/MODULAR_ARCHITECTURE.md](./architecture/MODULAR_ARCHITECTURE.md) |
| **Recording** | ✅ | [MEDIAMTX_MIGRATION.md](./MEDIAMTX_MIGRATION.md) |
| **Monitoring** | ✅ | [MEDIAMTX_MIGRATION.md](./MEDIAMTX_MIGRATION.md) |

## 📊 Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      Video Server (Rust)                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐        │
│  │  user-auth   │  │video-manager │  │image-manager │        │
│  │   (crate)    │  │   (crate)    │  │   (crate)    │        │
│  │              │  │              │  │              │        │
│  │ - OIDC       │  │ - Video CRUD │  │ - Image CRUD │        │
│  │ - Emergency  │  │ - HLS Proxy  │  │ - Upload     │        │
│  │ - Sessions   │  │ - Storage    │  │ - Serve      │        │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘        │
│         │                 │                  │                 │
│         └─────────────────┴──────────────────┘                 │
│                           │                                     │
│                    ┌──────▼──────┐                            │
│                    │   SQLite    │                            │
│                    │  Database   │                            │
│                    └─────────────┘                            │
└─────────────────────────────────────────────────────────────────┘
         │                            │
         ▼                            ▼
┌─────────────────┐         ┌──────────────────┐
│  Casdoor OIDC   │         │    MediaMTX      │
│   Provider      │         │  Streaming Server│
└─────────────────┘         └──────────────────┘
```

See: [architecture/MODULAR_ARCHITECTURE.md](./architecture/MODULAR_ARCHITECTURE.md)

## 🆘 Troubleshooting

### Authentication Issues
- **OIDC not working:** [auth/OIDC_TROUBLESHOOTING.md](./auth/OIDC_TROUBLESHOOTING.md)
- **Need emergency access:** [auth/EMERGENCY_LOGIN_QUICKSTART.md](./auth/EMERGENCY_LOGIN_QUICKSTART.md)

### Streaming Issues
- **No video showing:** [LIVE_STREAMING_GUIDE.md](./LIVE_STREAMING_GUIDE.md#troubleshooting)
- **No audio:** [LIVE_STREAMING_GUIDE.md](./LIVE_STREAMING_GUIDE.md#audio-configuration)
- **High latency:** [MEDIAMTX_MIGRATION.md](./MEDIAMTX_MIGRATION.md#performance)

### General Issues
- Check [TROUBLESHOOTING.md](../TROUBLESHOOTING.md) in root directory
- Review server logs: `journalctl -u video-server -f`
- Check MediaMTX status: `curl http://localhost:9997/v3/paths/get/live`

## 🔗 External Resources

- **MediaMTX:** https://github.com/bluenviron/mediamtx
- **Casdoor:** https://casdoor.org/
- **HLS.js:** https://github.com/video-dev/hls.js/
- **OIDC Spec:** https://openid.net/connect/

## 📝 Contributing

When adding documentation:

1. **Choose the right location:**
   - Authentication → `auth/`
   - Features → `features/`
   - Architecture → `architecture/`
   - Streaming → Root `docs/`

2. **Follow the format:**
   - Clear headers and sections
   - Code examples with syntax highlighting
   - Troubleshooting sections
   - Links to related docs

3. **Update this README:**
   - Add to table of contents
   - Update feature status table
   - Add to common tasks if applicable

4. **Test your documentation:**
   - Follow your own instructions
   - Verify all links work
   - Check code examples compile/run

## 🎓 Learning Path

### Beginner
1. [QUICKSTART.md](../QUICKSTART.md) - Get the server running
2. [auth/OIDC_QUICKSTART.md](./auth/OIDC_QUICKSTART.md) - Set up authentication
3. [LIVE_STREAMING_GUIDE.md](./LIVE_STREAMING_GUIDE.md) - Start streaming

### Intermediate
1. [architecture/MODULAR_ARCHITECTURE.md](./architecture/MODULAR_ARCHITECTURE.md) - Understand the codebase
2. [features/IMAGE_SERVING.md](./features/IMAGE_SERVING.md) - Add image support
3. [MEDIAMTX_MIGRATION.md](./MEDIAMTX_MIGRATION.md) - Advanced streaming

### Advanced
1. [auth/OIDC_IMPLEMENTATION.md](./auth/OIDC_IMPLEMENTATION.md) - Deep dive into auth
2. [auth/EMERGENCY_LOGIN_IMPLEMENTATION.md](./auth/EMERGENCY_LOGIN_IMPLEMENTATION.md) - Security patterns
3. [PROJECT_STATUS.md](../PROJECT_STATUS.md) - Contribute to the project

## 📄 License

See main project README for license information.

---

**Need help?** Check the troubleshooting guides or open an issue on GitHub.