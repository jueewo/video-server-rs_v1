# Live Streaming Guide

**Last Updated:** January 9, 2026

Complete guide for live streaming with Rust + MediaMTX.

## Quick Start

### 1. Start MediaMTX
```bash
cd video-server-rs_v1
mediamtx mediamtx.yml
```

You should see:
```
INF MediaMTX v1.5.1
INF [RTMP] listener opened on :1935
INF [HLS] listener opened on :8888
INF [WebRTC] listener opened on :8889
INF [API] listener opened on :9997
INF [Metrics] listener opened on :9998
```

### 2. Start Rust Server
```bash
cargo run --release
```

You should see a detailed startup banner with all URLs.

### 3. Start Streaming (macOS)

**With Audio:**
```bash
ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i "0:0" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -c:a aac -b:a 128k -ar 44100 \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

**Video Only:**
```bash
ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i "0" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

### 4. Watch the Stream

1. Login: http://localhost:3000/login
2. Watch: http://localhost:3000/test

## Architecture

```
┌─────────────┐
│ OBS/FFmpeg  │ RTMP stream (port 1935)
└──────┬──────┘
       │ rtmp://localhost:1935/live?token=xxx
       ↓
┌──────────────────┐
│    MediaMTX      │ Handles streaming protocols
│  - RTMP Input    │ - Receives RTMP stream
│  - HLS Output    │ - Converts to HLS
│  - WebRTC Output │ - Converts to WebRTC
│  - Recording     │ - Records to disk
└────┬────────┬────┘
     │        │
     │        │ Calls auth endpoints
     │        ↓
     │   ┌─────────────────┐
     │   │  Rust Server    │
     │   │  - Auth API     │
     │   │  - HLS Proxy    │
     │   │  - Session Mgmt │
     │   │  - Web UI       │
     │   └────────┬────────┘
     │            │
     ↓            ↓
┌────────────────────┐
│     Browser        │
│  - HLS.js Player   │
│  - 2-3s latency    │
└────────────────────┘
```

## Streaming from Different Sources

### macOS Camera + Microphone

**List available devices:**
```bash
ffmpeg -f avfoundation -list_devices true -i ""
```

Output example:
```
[AVFoundation indev] AVFoundation video devices:
[AVFoundation indev] [0] FaceTime HD Camera
[AVFoundation indev] [1] Capture screen 0
[AVFoundation indev] AVFoundation audio devices:
[AVFoundation indev] [0] MacBook Pro Microphone
[AVFoundation indev] [1] External Microphone
```

**Stream with specific devices:**
```bash
# Device format: "video_device:audio_device"
ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i "0:0" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -c:a aac -b:a 128k -ar 44100 \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

**Stream with external microphone:**
```bash
# Use video device 0 and audio device 1
ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i "0:1" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -c:a aac -b:a 128k -ar 44100 \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

### Linux Webcam + Microphone

**List video devices:**
```bash
ls -la /dev/video*
```

**List audio devices:**
```bash
arecord -l
```

**Stream:**
```bash
ffmpeg -f v4l2 -video_size 1280x720 -framerate 30 -i /dev/video0 \
  -f alsa -i hw:0 \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -c:a aac -b:a 128k -ar 44100 \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

### Stream from Video File

**Re-stream a video file:**
```bash
ffmpeg -re -i input.mp4 \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -c:a aac -b:a 128k \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

**Loop a video file:**
```bash
ffmpeg -stream_loop -1 -re -i input.mp4 \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -c:a aac -b:a 128k \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

### Screen Capture (macOS)

**Capture screen 0:**
```bash
ffmpeg -f avfoundation -framerate 30 -i "1:0" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -c:a aac -b:a 128k \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

**Capture screen + microphone:**
```bash
ffmpeg -f avfoundation -framerate 30 -i "1:0" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -c:a aac -b:a 128k \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

## OBS Studio Setup

OBS Studio provides a user-friendly GUI for streaming.

### Installation

**macOS:**
```bash
brew install --cask obs
```

**Linux:**
```bash
sudo apt install obs-studio
```

**Or download from:** https://obsproject.com/

### Configuration

#### 1. Stream Settings

1. Open OBS Studio
2. Settings → Stream
3. Configure:
   - **Service:** Custom
   - **Server:** `rtmp://localhost:1935/live`
   - **Stream Key:** `?token=supersecret123`

#### 2. Output Settings

Settings → Output → Streaming:

- **Video Encoder:** x264
- **Rate Control:** CBR
- **Bitrate:** 2500 Kbps (adjust based on upload speed)
- **Keyframe Interval:** 2 seconds
- **CPU Usage Preset:** veryfast
- **Profile:** baseline
- **Tune:** zerolatency

#### 3. Video Settings

Settings → Video:

- **Base (Canvas) Resolution:** 1920x1080
- **Output (Scaled) Resolution:** 1280x720
- **FPS:** 30

#### 4. Audio Settings

Settings → Audio:

- **Sample Rate:** 44.1 kHz
- **Channels:** Stereo
- **Desktop Audio Device:** (your audio output)
- **Microphone/Auxiliary Audio:** (your microphone)

#### 5. Add Sources

1. Click **+** under Sources
2. Add **Video Capture Device** (for camera)
3. Add **Audio Input Capture** (for microphone)
4. Add **Display Capture** (for screen sharing)

#### 6. Start Streaming

Click **Start Streaming** button.

Watch at: http://localhost:3000/test (after login)

## Common Issues & Solutions

### Issue 1: "Connection refused" when streaming

**Symptoms:**
- FFmpeg/OBS can't connect
- Error: `Connection refused`

**Causes:**
1. MediaMTX not running
2. Wrong port number
3. Firewall blocking port

**Solutions:**

**A. Check MediaMTX is running:**
```bash
# Check if port 1935 is open
lsof -i :1935

# Should show MediaMTX process
```

**B. Restart MediaMTX:**
```bash
# Stop if running
pkill mediamtx

# Start with config
mediamtx mediamtx.yml
```

**C. Check firewall:**
```bash
# macOS - temporarily disable firewall
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --setglobalstate off

# Linux
sudo ufw allow 1935/tcp
```

### Issue 2: "Unauthorized" / Authentication failed

**Symptoms:**
- Stream connects but immediately disconnects
- MediaMTX logs show: `authentication failed`

**Causes:**
1. Wrong token in URL
2. Rust server not running
3. Auth endpoint not responding

**Solutions:**

**A. Verify token:**
```bash
# Test auth endpoint directly
curl "http://localhost:3000/api/stream/validate?token=supersecret123"

# Should return: 200 OK
```

**B. Check Rust server is running:**
```bash
curl http://localhost:3000/health

# Should return: OK
```

**C. Check MediaMTX logs:**
Look for authentication errors in the MediaMTX terminal.

### Issue 3: No audio in stream

**Symptoms:**
- Video works but no sound
- Browser console shows no audio track

**Causes:**
1. Input device has no audio (using `"0"` instead of `"0:0"`)
2. Wrong audio device selected
3. macOS permissions not granted
4. Audio input muted

**Solutions:**

**A. Use correct device format:**
```bash
# ❌ Wrong - video only
-i "0"

# ✅ Correct - video + audio
-i "0:0"
```

**B. List and test devices:**
```bash
# List all devices
ffmpeg -f avfoundation -list_devices true -i ""

# Test audio recording
ffmpeg -f avfoundation -i ":0" -t 5 test_audio.wav
ffplay test_audio.wav
```

**C. Grant microphone permissions (macOS):**
- System Preferences → Security & Privacy → Microphone
- Enable access for Terminal/iTerm

**D. Check audio levels:**
- Ensure microphone is not muted
- Check input volume in System Preferences
- Test microphone in other apps

### Issue 4: Stream appears but no video shows in browser

**Symptoms:**
- MediaMTX receives stream
- No video in test player
- Browser console errors

**Causes:**
1. Not logged in
2. CORS issues
3. HLS.js errors
4. Codec incompatibility

**Solutions:**

**A. Login first:**
```bash
# Visit login page
open http://localhost:3000/login

# Or login via curl
curl http://localhost:3000/login
```

**B. Check stream is active:**
```bash
# Check MediaMTX API
curl http://localhost:9997/v3/paths/get/live

# Should show stream details with "ready: true"
```

**C. Check browser console:**
- Open browser console (F12)
- Look for CORS errors
- Look for HLS.js errors

**D. Test direct HLS access:**
```bash
# This should work after login
curl http://localhost:3000/hls/live/index.m3u8
```

### Issue 5: High latency (5+ seconds)

**Symptoms:**
- Significant delay between streaming and playback
- More than 3 seconds latency

**Current Configuration:**
- Segment duration: 500ms
- Part duration: 100ms
- Segment count: 3
- Expected latency: 2-3 seconds

**To reduce latency further:**

**Option A: Optimize MediaMTX (edit `mediamtx.yml`):**
```yaml
hls: yes
hlsVariant: lowLatency
hlsSegmentDuration: 500ms  # Already optimal
hlsPartDuration: 100ms      # Already optimal
hlsSegmentCount: 3          # Minimal buffering
```

**Option B: Use WebRTC instead:**
WebRTC provides sub-second latency (<1 second).

Access WebRTC stream:
```
http://localhost:8889/live/whep
```

Note: Requires WebRTC-compatible player (not HLS.js).

**Trade-offs:**
- Lower latency = more CPU usage
- Lower latency = more sensitive to network issues
- HLS: 2-3s latency, very stable
- WebRTC: <1s latency, less stable on poor networks

### Issue 6: Choppy/stuttering video

**Symptoms:**
- Video freezes or skips frames
- Buffering indicators

**Causes:**
- Network bandwidth too low
- CPU overloaded on streaming client
- Bitrate too high
- Framerate too high

**Solutions:**

**A. Reduce bitrate:**
```bash
ffmpeg -f avfoundation -i "0:0" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -b:v 1000k -maxrate 1200k -bufsize 2000k \  # Lower bitrate
  -c:a aac -b:a 128k \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

**B. Lower resolution:**
```bash
-video_size 640x480  # Instead of 1280x720
```

**C. Lower framerate:**
```bash
-framerate 24  # Instead of 30
```

**D. Use faster preset:**
```bash
-preset ultrafast  # Less CPU, lower quality
```

**E. Check network:**
```bash
# Test upload speed
speedtest-cli

# Monitor network usage
# macOS: nettop
# Linux: iftop
```

### Issue 7: Stream crashes or disconnects randomly

**Symptoms:**
- Stream works briefly then stops
- MediaMTX logs show disconnection
- FFmpeg exits with error

**Causes:**
1. Network instability
2. Camera/device disconnected
3. Insufficient upload bandwidth
4. CPU overload

**Solutions:**

**A. Enable auto-reconnect in FFmpeg:**
```bash
# Add these flags for auto-reconnect
-reconnect 1 -reconnect_streamed 1 -reconnect_delay_max 5
```

**B. Monitor MediaMTX logs:**
Look for specific error messages in MediaMTX terminal.

**C. Check system resources:**
```bash
# Monitor CPU usage
top

# Monitor disk space
df -h

# Monitor memory
free -m  # Linux
vm_stat  # macOS
```

**D. Use hardware encoding (if available):**
```bash
# macOS
-c:v h264_videotoolbox

# Linux with NVIDIA GPU
-c:v h264_nvenc

# Linux with Intel GPU
-c:v h264_vaapi
```

## Monitoring

### Check Stream Status

**MediaMTX API:**
```bash
# List all paths
curl http://localhost:9997/v3/paths/list

# Get specific path details
curl http://localhost:9997/v3/paths/get/live

# Check if stream is ready
curl http://localhost:9997/v3/paths/get/live | grep "ready"
```

**Rust Server:**
```bash
# Health check
curl http://localhost:3000/health

# MediaMTX status proxy
curl http://localhost:3000/api/mediamtx/status
```

### Monitor Recordings

```bash
# Watch recordings directory
watch -n 1 'ls -lh livestreams/live/'

# Check disk usage
du -sh livestreams/

# Find recordings older than 24h (should be auto-deleted)
find livestreams/ -type f -mtime +1
```

### Monitor Metrics

**Prometheus Metrics:**
```bash
# Get all metrics
curl http://localhost:9998/metrics

# Filter specific metrics
curl http://localhost:9998/metrics | grep mediamtx
```

**Common metrics:**
- `mediamtx_paths_total` - Number of active paths
- `mediamtx_readers_total` - Number of viewers
- `mediamtx_publishers_total` - Number of publishers
- `mediamtx_bytes_received_total` - Total bytes received
- `mediamtx_bytes_sent_total` - Total bytes sent

### Monitor System Resources

```bash
# CPU usage
top -pid $(pgrep mediamtx)
top -pid $(pgrep video-server)

# Memory usage
ps aux | grep mediamtx
ps aux | grep video-server

# Network usage
# macOS
nettop -m tcp

# Linux
iftop
nethogs
```

## Performance Optimization

### Client-Side Encoding

**Use hardware acceleration:**

**macOS (VideoToolbox):**
```bash
ffmpeg -f avfoundation -i "0:0" \
  -c:v h264_videotoolbox -b:v 2500k \
  -c:a aac -b:a 128k \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

**Linux with NVIDIA GPU:**
```bash
ffmpeg -f v4l2 -i /dev/video0 -f alsa -i hw:0 \
  -c:v h264_nvenc -preset p4 -b:v 2500k \
  -c:a aac -b:a 128k \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

**Linux with Intel GPU:**
```bash
ffmpeg -vaapi_device /dev/dri/renderD128 -f v4l2 -i /dev/video0 -f alsa -i hw:0 \
  -vf 'format=nv12,hwupload' \
  -c:v h264_vaapi -b:v 2500k \
  -c:a aac -b:a 128k \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

### Server-Side Optimization

**MediaMTX configuration (mediamtx.yml):**

**For lower latency:**
```yaml
hlsVariant: lowLatency
hlsSegmentDuration: 500ms
hlsPartDuration: 100ms
hlsSegmentCount: 3
```

**For stability (higher latency):**
```yaml
hlsVariant: mpegts
hlsSegmentDuration: 2s
hlsSegmentCount: 6
```

**Recording optimization:**
```yaml
recordFormat: fmp4        # More efficient than mp4
recordSegmentDuration: 1h # Larger segments = fewer files
```

## Security Best Practices

### Change Default Token

⚠️ **Critical:** Change the default token before production!

Edit `src/main.rs`:
```rust
const RTMP_PUBLISH_TOKEN: &str = "YOUR_STRONG_RANDOM_TOKEN_HERE";
```

Generate a secure token:
```bash
# macOS/Linux
openssl rand -hex 32

# Or use uuidgen
uuidgen
```

### Enable HTTPS

Use the provided `Caddyfile`:

```bash
# Install Caddy
brew install caddy  # macOS
sudo apt install caddy  # Linux

# Run with provided config
caddy run
```

### Configure MediaMTX TLS

Edit `mediamtx.yml`:
```yaml
rtmp: yes
rtmpEncryption: "optional"  # or "strict"
rtmpServerKey: server.key
rtmpServerCert: server.crt

hls: yes
hlsEncryption: yes
hlsServerKey: server.key
hlsServerCert: server.crt
```

### Implement Rate Limiting

Prevent abuse by rate limiting:
- Publishing attempts
- Authentication attempts
- API requests

### Firewall Configuration

Only expose necessary ports:
```bash
# Allow RTMP (input)
ufw allow 1935/tcp

# Allow HTTP (output)
ufw allow 3000/tcp

# Block MediaMTX API from public
ufw deny 9997/tcp

# Block metrics from public
ufw deny 9998/tcp
```

## Advanced Features

### Multiple Streams

Edit `mediamtx.yml` to add more paths:
```yaml
paths:
  stream1:
    runOnInit: curl -sf "http://localhost:3000/api/stream/validate?token=$MTX_QUERY" || exit 1
    record: yes
    
  stream2:
    runOnInit: curl -sf "http://localhost:3000/api/stream/validate?token=$MTX_QUERY" || exit 1
    record: yes
```

Stream to different paths:
```bash
rtmp://localhost:1935/stream1?token=supersecret123
rtmp://localhost:1935/stream2?token=supersecret123
```

### Multi-Quality Streaming (ABR)

MediaMTX can serve multiple quality levels. Configure in `mediamtx.yml`:
```yaml
paths:
  live:
    runOnPublish: |
      ffmpeg -i rtsp://localhost:8554/live \
        -c:v libx264 -b:v 4000k -s 1920x1080 -f rtsp rtsp://localhost:8554/live_1080p \
        -c:v libx264 -b:v 2500k -s 1280x720 -f rtsp rtsp://localhost:8554/live_720p \
        -c:v libx264 -b:v 1000k -s 854x480 -f rtsp rtsp://localhost:8554/live_480p
```

### Recording Configuration

Edit `mediamtx.yml`:
```yaml
paths:
  live:
    record: yes
    recordPath: ./livestreams/%path/%Y-%m-%d_%H-%M-%S
    recordFormat: fmp4
    recordPartDuration: 1h
    recordSegmentDuration: 1h
    recordDeleteAfter: 24h  # Auto-delete after 24 hours
```

### Webhooks

MediaMTX can call webhooks on events:
```yaml
paths:
  live:
    runOnReady: |
      curl -X POST http://localhost:3000/api/webhooks/stream-ready
    runOnDemand: |
      curl -X POST http://localhost:3000/api/webhooks/stream-ended
```

## Testing Checklist

Before going live, verify:

- [ ] MediaMTX starts without errors
- [ ] Rust server starts without errors
- [ ] Can login at http://localhost:3000/login
- [ ] MediaMTX API responds: `curl http://localhost:9997/v3/paths/list`
- [ ] Can start streaming with FFmpeg/OBS
- [ ] Stream appears in MediaMTX API: `curl http://localhost:9997/v3/paths/get/live`
- [ ] Can watch stream at http://localhost:3000/test
- [ ] Audio is working
- [ ] Video is smooth (no stuttering)
- [ ] Can stop and restart stream
- [ ] Recording works (files in `livestreams/` directory)
- [ ] Authentication works (can't watch without login)
- [ ] Health check works: `curl http://localhost:3000/health`
- [ ] Metrics endpoint works: `curl http://localhost:9998/metrics`

## FAQ

**Q: What's the minimum upload speed needed?**
A: At least 5 Mbps for 720p@30fps. Use 10+ Mbps for 1080p.

**Q: Can multiple people watch at the same time?**
A: Yes! HLS supports unlimited concurrent viewers.

**Q: How much disk space do recordings use?**
A: About 500 MB per hour at 2.5 Mbps bitrate.

**Q: Can I stream to multiple platforms simultaneously?**
A: Yes, use FFmpeg to re-stream to multiple destinations (YouTube, Twitch, etc.)

**Q: What's better: HLS or WebRTC?**
A: HLS is more stable (2-3s latency), WebRTC is faster (<1s) but less stable.

**Q: How do I stream to YouTube/Twitch?**
A: Use MediaMTX to re-stream or FFmpeg multi-output configuration.

**Q: Can I use a different port than 1935?**
A: Yes, change `rtmpAddress` in `mediamtx.yml`.

**Q: How do I enable RTMPS (secure RTMP)?**
A: Set `rtmpEncryption: yes` and configure TLS certificates in `mediamtx.yml`.

**Q: Can I password-protect specific streams?**
A: Yes, implement custom authentication in the Rust server's validate endpoint.

**Q: How do I reduce latency below 2 seconds?**
A: Use WebRTC output or further tune HLS settings (trade-off: stability).

## Additional Resources

- **MediaMTX Documentation:** https://github.com/bluenviron/mediamtx
- **FFmpeg Documentation:** https://ffmpeg.org/documentation.html
- **HLS.js Documentation:** https://github.com/video-dev/hls.js/
- **OBS Studio Forum:** https://obsproject.com/forum/
- **WebRTC Guide:** https://webrtc.org/getting-started/

## Support

For issues:
1. Check this guide's troubleshooting section
2. Review MediaMTX logs for streaming issues
3. Review Rust server logs for API issues
4. Check browser console for client-side errors
5. Review `PROJECT_STATUS.md` for known limitations

---

**Last Updated:** January 9, 2026  
**Status:** MediaMTX Integration Complete ✅