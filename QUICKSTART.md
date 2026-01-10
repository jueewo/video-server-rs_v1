# 🚀 Quick Start Guide

**Last Updated:** January 9, 2026

This video server uses MediaMTX for production-ready live streaming. Follow these steps to get started.

## Prerequisites

- Rust (already installed ✓)
- FFmpeg (for streaming from your camera)
- MediaMTX (needs to be installed)

## Step 1: Install MediaMTX

### macOS
```bash
brew install mediamtx
```

### Linux
```bash
wget https://github.com/bluenviron/mediamtx/releases/latest/download/mediamtx_v1.5.1_linux_amd64.tar.gz
tar -xzf mediamtx_v1.5.1_linux_amd64.tar.gz
sudo mv mediamtx /usr/local/bin/
```

### Verify Installation
```bash
mediamtx --version
```

## Step 2: Start MediaMTX

Open a terminal and navigate to your project directory:

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

**Keep this terminal open!**

## Step 3: Start Rust Server

Open another terminal and run:

```bash
cd video-server-rs_v1
cargo run --release
```

You should see a detailed startup banner with all the URLs and configuration.

**Keep this terminal open too!**

## Step 4: Login

Open your browser and visit:
```
http://localhost:3000/login
```

This creates a session so you can watch the live stream.

## Step 5: Start Streaming

### Option A: Using FFmpeg (macOS)

Open a third terminal:

```bash
# List your devices first
ffmpeg -f avfoundation -list_devices true -i ""

# Stream with camera and microphone (device 0:0)
ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i "0:0" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -c:a aac -b:a 128k -ar 44100 \
  -f flv "rtmp://localhost:1935/live?token=supersecret123"
```

### Option B: Using OBS Studio

1. Open OBS Studio
2. Go to Settings → Stream
3. Set:
   - Service: **Custom**
   - Server: **rtmp://localhost:1935/live**
   - Stream Key: **?token=supersecret123**
4. Click "Start Streaming"

## Step 6: Watch the Stream

Open your browser and visit:
```
http://localhost:3000/test
```

You should see your live stream with 2-3 seconds delay!

## Terminals Summary

You need 3 terminals running:

1. **Terminal 1 - MediaMTX**
   ```bash
   mediamtx mediamtx.yml
   ```

2. **Terminal 2 - Rust Server**
   ```bash
   cargo run --release
   ```

3. **Terminal 3 - Streaming** (Optional if using OBS)
   ```bash
   ffmpeg -f avfoundation -i "0:0" -c:v libx264 -c:a aac ...
   ```

## Testing Commands

### Check MediaMTX is Running
```bash
curl http://localhost:9997/v3/paths/list
```

Should return JSON with available streaming paths.

### Check if Stream is Live
```bash
curl http://localhost:9997/v3/paths/get/live
```

Should show stream details when active.

### Check Rust Server
```bash
curl http://localhost:3000/health
```

Should return "OK".

### Test Authentication
```bash
# Should succeed with correct token
curl "http://localhost:3000/api/stream/validate?token=supersecret123"

# Should fail with wrong token
curl "http://localhost:3000/api/stream/validate?token=wrong"
```

## Troubleshooting

### "Connection refused" when streaming
- Make sure MediaMTX is running in Terminal 1
- Check port 1935 is not blocked: `lsof -i :1935`

### "Unauthorized" when watching
- Make sure you visited `/login` first
- Check Rust server is running in Terminal 2

### No video appears
- Check MediaMTX terminal for errors
- Check Rust server terminal for errors
- Verify stream is active: `curl http://localhost:9997/v3/paths/get/live`

### No audio
- Make sure you're using `"0:0"` format (video:audio)
- Check microphone permissions on macOS
- List devices: `ffmpeg -f avfoundation -list_devices true -i ""`

### High CPU usage
- High CPU on streaming client is normal (encoding video)
- Server CPU should be low (~5-10%, just proxying)
- MediaMTX is very efficient

## URLs Reference

| Service | URL | Purpose |
|---------|-----|---------|
| Web UI | http://localhost:3000 | Main page |
| Login | http://localhost:3000/login | Create session |
| Test Player | http://localhost:3000/test | Watch stream |
| Health Check | http://localhost:3000/health | Server status |
| MediaMTX API | http://localhost:9997/v3/paths/list | Stream info |
| Metrics | http://localhost:9998/metrics | Prometheus metrics |

## Ports

| Port | Service | Protocol |
|------|---------|----------|
| 3000 | Rust Server | HTTP |
| 1935 | MediaMTX | RTMP (input) |
| 8888 | MediaMTX | HLS (output) |
| 8889 | MediaMTX | WebRTC |
| 9997 | MediaMTX | API |
| 9998 | MediaMTX | Metrics |

## Next Steps

Once everything is working:

1. **Change the token** in `src/main.rs`:
   ```rust
   const RTMP_PUBLISH_TOKEN: &str = "YOUR_STRONG_RANDOM_TOKEN";
   ```

2. **Set up HTTPS** using the provided `Caddyfile`

3. **Try WebRTC** for ultra-low latency (<1 second)
   - Access: `http://localhost:8889/live/whep`

4. **Configure recording** in `mediamtx.yml` (already enabled with 24h retention)

5. **Set up monitoring** using the metrics endpoint at `http://localhost:9998/metrics`

6. **Review production checklist** in `PROJECT_STATUS.md`

## Documentation

For more details, see:
- `README.md` - Complete feature documentation
- `PROJECT_STATUS.md` - Current project status
- `docs/LIVE_STREAMING_GUIDE.md` - Streaming guide and troubleshooting
- `docs/MEDIAMTX_MIGRATION.md` - Architecture details and advanced configuration

## Getting Help

If something doesn't work:

1. Check all 3 terminals for error messages
2. Review the troubleshooting section above
3. Check `docs/LIVE_STREAMING_GUIDE.md` for common issues
4. Review `docs/MEDIAMTX_MIGRATION.md` for advanced configuration
5. MediaMTX documentation: https://github.com/bluenviron/mediamtx
6. Check browser console (F12) for client-side errors

## Features

✅ Live RTMP streaming  
✅ HLS playback (2-3s latency)  
✅ WebRTC support (<1s latency)  
✅ Automatic recording (24h retention)  
✅ Session authentication  
✅ Prometheus metrics  
✅ Health monitoring  

---

Happy streaming! 🎥 🚀
