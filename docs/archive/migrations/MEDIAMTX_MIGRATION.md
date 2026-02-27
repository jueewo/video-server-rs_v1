# MediaMTX Migration Guide

## Overview

This guide explains how to migrate from the current FFmpeg-based live streaming solution to MediaMTX, a production-ready, high-performance media server.

## Why MediaMTX?

### Current Setup Limitations

**Architecture:**
```
[OBS/FFmpeg] → RTMP → [Rust + FFmpeg spawn] → HLS → [Browser]
                          ↑ (process management, fragile)
```

**Issues:**
- FFmpeg process spawning/monitoring in Rust is complex
- High latency with HLS (4-8 seconds)
- Manual segment file handling
- Limited error recovery
- No WebRTC support
- Higher CPU usage

### MediaMTX Advantages

**New Architecture:**
```
[OBS/FFmpeg] → RTMP → [MediaMTX] → HLS/WebRTC → [Browser]
                           ↓
                    [Rust Server] ← Authentication
```

**Benefits:**

#### 1. Production-Ready & Reliable
- Purpose-built for live streaming
- Battle-tested by thousands of users
- Automatic process management and recovery
- Better error handling and logging
- Active development and community support

#### 2. Multiple Protocol Support
- **RTMP** - for OBS/FFmpeg input (what you're using now)
- **HLS** - for browser playback (what you're serving now)
- **WebRTC** - sub-second latency streaming! 🚀
- **RTSP** - IP camera support
- **SRT** - secure reliable transport
- **MPEG-TS** - broadcast standard
- **And more...**

#### 3. Much Lower Latency
- **Current HLS setup:** 4-8 seconds delay
- **MediaMTX with LL-HLS:** 2-3 seconds delay
- **MediaMTX with WebRTC:** <1 second delay ⚡

#### 4. Built-in Features
- Authentication hooks (integrates with your Rust server)
- Automatic recording
- Metrics and monitoring endpoints
- Multiple simultaneous streams
- Hardware transcoding support
- Stream fallback/redundancy
- Webhook notifications

#### 5. Better Resource Usage
- More efficient than spawning FFmpeg processes
- Lower CPU and memory usage
- Optimized for concurrent viewers
- Better network handling

## Installation

### macOS

```bash
# Using Homebrew
brew install mediamtx

# Or download binary
wget https://github.com/bluenviron/mediamtx/releases/latest/download/mediamtx_v1.5.1_darwin_amd64.tar.gz
tar -xzf mediamtx_v1.5.1_darwin_amd64.tar.gz
sudo mv mediamtx /usr/local/bin/
```

### Linux

```bash
# Download latest release
wget https://github.com/bluenviron/mediamtx/releases/latest/download/mediamtx_v1.5.1_linux_amd64.tar.gz
tar -xzf mediamtx_v1.5.1_linux_amd64.tar.gz
sudo mv mediamtx /usr/local/bin/

# Create systemd service (optional)
sudo tee /etc/systemd/system/mediamtx.service > /dev/null <<EOL
[Unit]
Description=MediaMTX
After=network.target

[Service]
Type=simple
User=mediamtx
ExecStart=/usr/local/bin/mediamtx /etc/mediamtx/mediamtx.yml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOL

sudo systemctl daemon-reload
sudo systemctl enable mediamtx
sudo systemctl start mediamtx
```

### Verify Installation

```bash
mediamtx --version
```

## Configuration

### Create MediaMTX Configuration

Create `mediamtx.yml` in your project root:

```yaml
###############################################
# MediaMTX Configuration for Live Streaming
###############################################

# Log level: debug, info, warn
logLevel: info
logDestinations: [stdout]

# API for metrics and monitoring
api: yes
apiAddress: :9997

# Metrics
metrics: yes
metricsAddress: :9998

###############################################
# Paths (Streams)
###############################################
paths:
  # Live stream path
  live:
    # Source configuration
    source: publisher
    sourceProtocol: rtmp
    sourceOnDemand: no
    
    # Publisher authentication
    # Calls your Rust server to validate the publish token
    publishUser: ""
    publishPass: ""
    publishIPs: []
    
    # Validate publisher via HTTP callback
    runOnInit: curl -sf "http://localhost:3000/api/stream/validate?token=$MTX_QUERY" || exit 1
    runOnInitRestart: yes
    
    # Reader authentication
    # Calls your Rust server to check if viewer is authenticated
    readUser: ""
    readPass: ""
    runOnRead: curl -sf "http://localhost:3000/api/stream/authorize?session=$MTX_QUERY" || exit 1
    runOnReadRestart: yes
    
    # Recording (optional)
    record: no
    # recordPath: ./recordings/%path/%Y-%m-%d_%H-%M-%S
    # recordFormat: mp4
    # recordSegmentDuration: 1h
    
    # Webhooks (optional)
    # runOnReady: curl -X POST http://localhost:3000/api/webhooks/stream-ready
    # runOnNotReady: curl -X POST http://localhost:3000/api/webhooks/stream-ended

###############################################
# RTMP (Input from OBS/FFmpeg)
###############################################
rtmp: yes
rtmpAddress: :1935
rtmpEncryption: "no"
rtmpServerKey: ""
rtmpServerCert: ""

###############################################
# HLS (Output for browsers)
###############################################
hls: yes
hlsAddress: :8888
hlsEncryption: no
hlsServerKey: ""
hlsServerCert: ""

# Low-latency HLS configuration
hlsVariant: lowLatency
hlsSegmentCount: 7
hlsSegmentDuration: 1s
hlsPartDuration: 200ms
hlsSegmentMaxSize: 50M

# Delete old segments
hlsAllowOrigin: "*"
hlsAlwaysRemux: no
hlsMuxerCloseAfter: 60s

###############################################
# WebRTC (Ultra-low latency)
###############################################
webrtc: yes
webrtcAddress: :8889
webrtcEncryption: no
webrtcServerKey: ""
webrtcServerCert: ""

# ICE servers for WebRTC
webrtcICEServers2:
  - urls: [stun:stun.l.google.com:19302]

# Local network settings
webrtcICEHostNAT1To1IPs: []
webrtcICEUDPMuxAddress: :8189
webrtcICETCPMuxAddress: :8189

###############################################
# Performance Tuning
###############################################
readTimeout: 10s
writeTimeout: 10s
readBufferCount: 512
udpMaxPayloadSize: 1472
```

## Rust Server Integration

### 1. Add Dependencies to `Cargo.toml`

```toml
[dependencies]
# Existing dependencies...
reqwest = { version = "0.11", features = ["json"] }
```

### 2. Add Authentication Endpoints

Add these handlers to `src/main.rs`:

```rust
use std::collections::HashMap;
use axum::extract::Query;

// Validate stream publisher (for RTMP push)
async fn validate_stream_handler(
    Query(params): Query<HashMap<String, String>>,
) -> Result<StatusCode, StatusCode> {
    let token = params.get("token").ok_or(StatusCode::UNAUTHORIZED)?;
    
    if token == RTMP_PUBLISH_TOKEN {
        println!("✅ Stream publisher authorized: token={}", token);
        Ok(StatusCode::OK)
    } else {
        println!("❌ Stream publisher rejected: invalid token");
        Err(StatusCode::UNAUTHORIZED)
    }
}

// Authorize stream viewer (for HLS/WebRTC playback)
async fn authorize_stream_handler(
    session: Session,
) -> Result<StatusCode, StatusCode> {
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);
    
    if authenticated {
        let user_id: Option<u32> = session.get("user_id").await.ok().flatten();
        println!("✅ Stream viewer authorized: user_id={:?}", user_id);
        Ok(StatusCode::OK)
    } else {
        println!("❌ Stream viewer rejected: not authenticated");
        Err(StatusCode::UNAUTHORIZED)
    }
}

// Optional: Webhook handlers for stream events
async fn webhook_stream_ready() -> StatusCode {
    println!("📡 Stream is now live!");
    StatusCode::OK
}

async fn webhook_stream_ended() -> StatusCode {
    println!("📡 Stream has ended");
    StatusCode::OK
}
```

### 3. Update Router

```rust
let app = Router::new()
    .route("/", get(index_handler))
    .route("/login", get(login_handler))
    .route("/logout", get(logout_handler))
    .route("/test", get(test_page_handler))
    
    // MediaMTX authentication endpoints
    .route("/api/stream/validate", get(validate_stream_handler))
    .route("/api/stream/authorize", get(authorize_stream_handler))
    
    // Optional: MediaMTX webhook endpoints
    .route("/api/webhooks/stream-ready", post(webhook_stream_ready))
    .route("/api/webhooks/stream-ended", post(webhook_stream_ended))
    
    // Proxy HLS requests to MediaMTX
    .route("/hls/*path", get(hls_proxy_handler))
    
    .with_state(state)
    .layer(/* ... existing layers ... */);
```

### 4. Add HLS Proxy Handler

Replace the existing `hls_handler` with a proxy to MediaMTX:

```rust
use reqwest::Client;

async fn hls_proxy_handler(
    Path(path): Path<String>,
    session: Session,
) -> Result<Response, StatusCode> {
    // Check authentication
    let authenticated: bool = session
        .get("authenticated")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);
    
    if !authenticated {
        println!("❌ HLS request rejected: not authenticated");
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // Proxy request to MediaMTX HLS server
    let mediamtx_url = format!("http://localhost:8888/{}", path);
    
    let client = Client::new();
    let response = client
        .get(&mediamtx_url)
        .send()
        .await
        .map_err(|e| {
            println!("❌ MediaMTX proxy error: {}", e);
            StatusCode::BAD_GATEWAY
        })?;
    
    // Determine content type
    let content_type = if path.ends_with(".m3u8") {
        "application/vnd.apple.mpegurl"
    } else if path.ends_with(".ts") {
        "video/MP2T"
    } else {
        "application/octet-stream"
    };
    
    // Get response body
    let bytes = response.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    
    // Build response with proper headers
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, if path.ends_with(".m3u8") {
            "no-cache, no-store, must-revalidate"
        } else {
            "max-age=10"
        })
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS")
        .body(axum::body::Body::from(bytes))
        .unwrap())
}
```

### 5. Remove FFmpeg Spawning Code

Remove or comment out the entire FFmpeg spawning code block (around lines 157-220):

```rust
// REMOVE THIS SECTION:
// spawn(async move {
//     loop {
//         println!("Starting secure live ingest (RTMP -> HLS)...");
//         let mut child = Command::new("ffmpeg")
//         ...
//     }
// });
```

## Running the System

### 1. Start MediaMTX

```bash
# In one terminal
mediamtx mediamtx.yml
```

You should see:
```
INF MediaMTX v1.5.1
INF [RTMP] listener opened on :1935
INF [HLS] listener opened on :8888
INF [WebRTC] listener opened on :8889
INF [API] listener opened on :9997
```

### 2. Start Your Rust Server

```bash
# In another terminal
cargo run --release
```

### 3. Stream to MediaMTX

```bash
# macOS with camera and microphone
ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i "0:0" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -c:a aac -b:a 128k -ar 44100 \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

### 4. Watch the Stream

**Option A: HLS (your current test page)**
- Open: `http://localhost:3000/test`
- Update URL to: `http://localhost:3000/hls/live/index.m3u8`

**Option B: WebRTC (ultra-low latency)**
- Open MediaMTX's built-in player: `http://localhost:8889/live`
- Or integrate WebRTC into your own player

## Port Reference

| Service | Port | Purpose |
|---------|------|---------|
| Rust Server | 3000 | HTTP API, authentication, HLS proxy |
| MediaMTX RTMP | 1935 | RTMP ingest (from OBS/FFmpeg) |
| MediaMTX HLS | 8888 | HLS output (proxied by Rust) |
| MediaMTX WebRTC | 8889 | WebRTC output (direct or proxied) |
| MediaMTX API | 9997 | Monitoring and control |
| MediaMTX Metrics | 9998 | Prometheus metrics |

## Migration Checklist

- [ ] Install MediaMTX
- [ ] Create `mediamtx.yml` configuration
- [ ] Add `reqwest` dependency to `Cargo.toml`
- [ ] Add authentication endpoints to Rust server
- [ ] Add HLS proxy handler
- [ ] Remove FFmpeg spawning code
- [ ] Update routes
- [ ] Test publisher authentication
- [ ] Test viewer authentication
- [ ] Test HLS playback
- [ ] Test WebRTC playback (optional)
- [ ] Update documentation
- [ ] Deploy to production

## Testing

### 1. Test MediaMTX Directly

```bash
# Check if MediaMTX is running
curl http://localhost:9997/v3/paths/list

# Should return JSON with available paths
```

### 2. Test Stream Publishing

```bash
# Stream a test file
ffmpeg -re -i test.mp4 -c copy \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

### 3. Test Authentication

```bash
# Should succeed with valid token
curl "http://localhost:3000/api/stream/validate?token=supersecret123"

# Should fail with invalid token
curl "http://localhost:3000/api/stream/validate?token=wrong"
```

### 4. Test HLS Playback

```bash
# After logging in via browser, test HLS endpoint
curl -H "Cookie: your-session-cookie" \
  http://localhost:3000/hls/live/index.m3u8
```

## Monitoring

### MediaMTX Metrics

```bash
# View metrics
curl http://localhost:9998/metrics

# Key metrics to watch:
# - mediamtx_paths (active streams)
# - mediamtx_rtmp_conns (RTMP connections)
# - mediamtx_hls_sessions (HLS viewers)
# - mediamtx_webrtc_sessions (WebRTC viewers)
```

### MediaMTX API

```bash
# List all active streams
curl http://localhost:9997/v3/paths/list

# Get specific stream info
curl http://localhost:9997/v3/paths/get/live

# Kick a publisher
curl -X POST http://localhost:9997/v3/paths/kick/live
```

## Troubleshooting

### MediaMTX won't start

```bash
# Check if ports are in use
lsof -i :1935
lsof -i :8888
lsof -i :8889

# Check MediaMTX logs
mediamtx mediamtx.yml 2>&1 | tee mediamtx.log
```

### Authentication not working

1. Check Rust server logs for auth requests
2. Test auth endpoints directly with `curl`
3. Verify `MTX_QUERY` variable is being passed correctly
4. Check MediaMTX logs for auth failures

### Stream not appearing

1. Check if publisher is connected:
   ```bash
   curl http://localhost:9997/v3/paths/get/live
   ```
2. Check MediaMTX logs for errors
3. Verify RTMP URL and token are correct
4. Test direct HLS access: `http://localhost:8888/live/index.m3u8`

### High latency with HLS

- Switch to WebRTC for <1 second latency
- Or tune LL-HLS settings in `mediamtx.yml`:
  ```yaml
  hlsSegmentDuration: 500ms
  hlsPartDuration: 100ms
  ```

## Production Deployment

### Security Checklist

- [ ] Change `RTMP_PUBLISH_TOKEN` to a strong random value
- [ ] Enable HTTPS for Rust server
- [ ] Enable TLS for MediaMTX (RTMPS, HLS over HTTPS)
- [ ] Use proper authentication system (not simple login handler)
- [ ] Restrict MediaMTX API access (firewall port 9997)
- [ ] Set up rate limiting
- [ ] Enable recording for archival
- [ ] Set up monitoring and alerts

### Performance Tips

1. **Use hardware encoding** when streaming:
   ```bash
   # macOS
   -c:v h264_videotoolbox
   
   # Linux with NVIDIA
   -c:v h264_nvenc
   ```

2. **Enable hardware transcoding** in MediaMTX (requires FFmpeg):
   ```yaml
   paths:
     live:
       runOnReady: ffmpeg -i rtsp://localhost:8554/live -c:v h264_nvenc ...
   ```

3. **Optimize for concurrent viewers:**
   - HLS naturally scales well (CDN-friendly)
   - WebRTC requires TURN server for many viewers
   - Consider adding a CDN for HLS segments

4. **Monitor system resources:**
   ```bash
   # CPU and memory usage
   htop
   
   # Network usage
   iftop
   ```

## Rollback Plan

If you need to revert to the old system:

1. Stop MediaMTX
2. Restore FFmpeg spawning code in `src/main.rs`
3. Restore old `hls_handler` function
4. Remove MediaMTX-related routes
5. Rebuild and restart Rust server

## Conclusion

MediaMTX provides a **production-ready, reliable, and feature-rich** alternative to manually spawning FFmpeg processes. The migration effort is moderate, but the benefits are substantial:

- ✅ Much lower latency (especially with WebRTC)
- ✅ Better reliability and error handling
- ✅ Less code to maintain
- ✅ More features out of the box
- ✅ Better scalability

For production use, MediaMTX is **highly recommended** over the current FFmpeg spawning approach.

## Additional Resources

- [MediaMTX GitHub](https://github.com/bluenviron/mediamtx)
- [MediaMTX Documentation](https://github.com/bluenviron/mediamtx/tree/main/docs)
- [WHIP/WHEP Protocol](https://www.ietf.org/archive/id/draft-ietf-wish-whip-01.html)
- [Low Latency HLS](https://developer.apple.com/documentation/http_live_streaming/protocol_extension_for_low-latency_hls_preliminary_specification)
