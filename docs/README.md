# Video Server Documentation

**Last Updated:** January 9, 2026

This directory contains documentation for the Rust + MediaMTX HLS live streaming server.

## Documentation Files

### 📘 [LIVE_STREAMING_GUIDE.md](./LIVE_STREAMING_GUIDE.md)
**Complete streaming guide and troubleshooting**

Covers:
- Quick start guide for MediaMTX-based setup
- Streaming from macOS/Linux/OBS
- FFmpeg streaming commands
- Common issues and solutions
- Audio configuration
- Testing and monitoring
- Performance optimization
- Security best practices

### 🚀 [MEDIAMTX_MIGRATION.md](./MEDIAMTX_MIGRATION.md)
**Architecture details and advanced configuration**

Covers:
- MediaMTX integration architecture
- Advanced configuration options
- WebRTC ultra-low latency setup
- Multi-quality streaming (ABR)
- Recording configuration
- Production deployment
- Monitoring and metrics

## Quick Links

### Current Setup
- **Start MediaMTX:** `mediamtx mediamtx.yml`
- **Start Rust server:** `cargo run --release`
- **Login:** http://localhost:3000/login
- **Test page:** http://localhost:3000/test
- **Stream URL:** `rtmp://localhost:1935/live?token=supersecret123`

### Getting Started

1. ✅ Follow [LIVE_STREAMING_GUIDE.md](./LIVE_STREAMING_GUIDE.md) for setup
2. 🔒 Change default token in `src/main.rs`
3. 📊 Set up monitoring (metrics at :9998)
4. 🌐 Deploy with HTTPS using provided `Caddyfile`
5. 🚀 Consider WebRTC for ultra-low latency

## Architecture Overview

### Current Architecture (MediaMTX Integrated)
```
┌─────────────┐
│ OBS/FFmpeg  │ Stream with video + audio
└──────┬──────┘
       │ RTMP: rtmp://localhost:1935/live?token=supersecret123
       ↓
┌──────────────────┐     ┌─────────────────────┐
│    MediaMTX      │────→│   Rust Server       │
│  - RTMP Ingest   │Auth │   - Authentication  │
│  - HLS Output    │     │   - Session Mgmt    │
│  - WebRTC Output │     │   - HLS Proxy       │
│  - Recording     │     │   - Web UI          │
└────┬────────┬────┘     └──────────┬──────────┘
     │        │                     │
     │ HLS    │ WebRTC             HTTP
     │        │                     │
     └────────┴─────────────────────↓
              ┌─────────────────────┐
              │      Browser        │
              │   HLS.js Player     │
              │   (2-3s latency)    │
              └─────────────────────┘
```

### Benefits of MediaMTX Integration

✅ **Production-ready** - Battle-tested streaming server  
✅ **Low latency** - 2-3s with HLS, <1s with WebRTC  
✅ **Multiple protocols** - RTMP, HLS, WebRTC, RTSP, SRT  
✅ **Built-in features** - Recording, authentication hooks, metrics  
✅ **Better reliability** - Automatic error recovery  
✅ **Lower maintenance** - Less code to manage

## Current Features

| Feature | Status | Description |
|---------|--------|-------------|
| **Live RTMP Streaming** | ✅ | Standard RTMP ingest on port 1935 |
| **HLS Output** | ✅ | Low-latency HLS (2-3s) via MediaMTX |
| **WebRTC Output** | ✅ | Ultra-low latency (<1s) option |
| **Authentication** | ✅ | Session-based auth with MediaMTX hooks |
| **Recording** | ✅ | Automatic recording with 24h retention |
| **Monitoring** | ✅ | Prometheus metrics endpoint |
| **Multi-viewer** | ✅ | Unlimited concurrent viewers |
| **CORS Support** | ✅ | Configured for cross-origin requests |
| **Production Ready** | ✅ | Stable and tested |

## Port Reference

| Port | Service | Purpose |
|------|---------|---------|
| 3000 | Rust HTTP | Web UI, auth, HLS proxy |
| 1935 | MediaMTX RTMP | RTMP ingest (standard port) |
| 8888 | MediaMTX HLS | HLS output |
| 8889 | MediaMTX WebRTC | WebRTC output |
| 9997 | MediaMTX API | Control API |
| 9998 | MediaMTX Metrics | Prometheus metrics |

## Getting Help

### Issues with Current Setup
See [LIVE_STREAMING_GUIDE.md](./LIVE_STREAMING_GUIDE.md) troubleshooting section

### Questions about Migration
See [MEDIAMTX_MIGRATION.md](./MEDIAMTX_MIGRATION.md) FAQ and troubleshooting

### Common Problems

**No video showing:**
- Ensure MediaMTX is running: `lsof -i :1935`
- Login first: http://localhost:3000/login
- Check stream status: `curl http://localhost:9997/v3/paths/get/live`
- Check browser console for errors

**No audio:**
- Use `"0:0"` format in FFmpeg (video:audio), not just `"0"`
- Check microphone permissions on macOS
- List devices: `ffmpeg -f avfoundation -list_devices true -i ""`

**High latency:**
- HLS: 2-3s is normal and optimized
- For <1s: Use WebRTC at http://localhost:8889/live/whep
- Further tuning: Edit `mediamtx.yml` HLS settings

**Connection refused:**
- MediaMTX not running: `mediamtx mediamtx.yml`
- Wrong port (use 1935, not 1936)
- Check firewall settings

**Authentication failed:**
- Wrong token in stream URL
- Rust server not running
- Check: `curl "http://localhost:3000/api/stream/validate?token=supersecret123"`

## Development Roadmap

### Completed ✅
- [x] MediaMTX integration
- [x] RTMP → HLS streaming (2-3s latency)
- [x] Session-based authentication
- [x] HLS.js player integration
- [x] Audio support
- [x] WebRTC support (<1s latency)
- [x] Recording functionality (24h retention)
- [x] Prometheus metrics
- [x] Authentication hooks

### Planned 🚀
- [ ] JWT/OAuth authentication
- [ ] Multi-quality streaming (ABR)
- [ ] PostgreSQL migration
- [ ] Admin dashboard
- [ ] Multiple concurrent streams
- [ ] CDN integration
- [ ] Stream analytics
- [ ] Chat integration
- [ ] User management system

## Contributing

When adding new features or documentation:

1. Update relevant documentation files
2. Add examples and code snippets
3. Include troubleshooting tips
4. Test on both macOS and Linux if possible
5. Update this README and PROJECT_STATUS.md
6. Verify MediaMTX compatibility

## License

See main project README for license information.
