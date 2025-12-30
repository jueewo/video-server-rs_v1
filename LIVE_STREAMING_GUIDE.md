# Live Streaming Troubleshooting Guide

## Quick Start

### 1. Start the Server
```bash
cargo run --release
```

You should see:
```
🎥 HLS Live Streaming Server Started!
Server: http://0.0.0.0:3000
Test page: http://0.0.0.0:3000/test
```

### 2. Start Streaming (macOS)

**With Audio:**
```bash
ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i "0:0" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -c:a aac -b:a 128k -ar 44100 \
  -f flv "rtmp://localhost:1936/live?token=supersecret123"
```

**Video Only:**
```bash
ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i "0" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -f flv "rtmp://localhost:1936/live?token=supersecret123"
```

### 3. Watch the Stream
Open in browser: http://localhost:3000/test

## Common Issues & Solutions

### Issue 1: Stream shows "last seconds" only after stopping

**Symptoms:**
- No video appears while streaming
- Only see last few seconds after stopping FFmpeg
- Test page shows "waiting for segments"

**Cause:**
FFmpeg is buffering and not writing segments in real-time.

**Solution:**
The server has been configured with:
- `omit_endlist` flag - keeps playlist open for live streaming
- `event` playlist type - signals live event
- Low segment cache time (10 seconds)

**Verify it's working:**
1. Start streaming
2. Check if segments are being created:
   ```bash
   watch -n 1 'ls -lh storage/private/live/'
   ```
3. You should see new `.ts` files appearing every 2 seconds

### Issue 2: No audio in stream

**Symptoms:**
- Video works but no sound
- Browser console shows no audio track

**Causes:**
1. Input device has no audio (using `"0"` instead of `"0:0"`)
2. Wrong audio device selected
3. macOS permissions not granted

**Solutions:**

**A. List available devices:**
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

**B. Use correct device format:**
- `"0:0"` = video device 0 + audio device 0 ✅
- `"0"` = video device 0 only (no audio) ❌

**C. Grant microphone permission:**
- macOS will prompt for microphone access
- If denied, go to System Preferences > Security & Privacy > Microphone
- Enable access for Terminal/iTerm

**D. Test audio input:**
```bash
# Record 5 seconds of audio to test
ffmpeg -f avfoundation -i ":0" -t 5 test_audio.wav
ffplay test_audio.wav
```

### Issue 3: "bufferAppendError" in browser

**Symptoms:**
- HLS.js error: `mediaError bufferAppendError`
- Video won't play in browser

**Cause:**
Incompatible video codec or format.

**Solution:**
The server-side FFmpeg now re-encodes to browser-compatible format:
- H.264 Baseline profile
- yuv420p pixel format
- AAC audio

If still having issues, check FFmpeg output for errors.

### Issue 4: High latency (5-10 second delay)

**Symptoms:**
- Significant delay between streaming and playback

**Current Configuration:**
- Segment duration: 2 seconds
- Playlist size: 6 segments
- Expected latency: 4-8 seconds

**To reduce latency further:**

Edit `src/main.rs` and change:
```rust
.arg("-hls_time").arg("1")         // 1-second segments
.arg("-hls_list_size").arg("3")    // Only 3 segments
```

Trade-off: Lower latency = more CPU usage and potential buffering.

### Issue 5: FFmpeg crashes or exits immediately

**Check FFmpeg output:**
The server now shows FFmpeg stderr output. Look for:

**Common errors:**

1. **"Connection refused"**
   - Server not running
   - Wrong port
   - Firewall blocking port 1936

2. **"Invalid token"**
   - Wrong token in URL
   - URL encoding issues

3. **"Device busy"**
   - Camera already in use by another app
   - Close other apps using camera (Zoom, Skype, etc.)

### Issue 6: Choppy/stuttering video

**Causes:**
- Network bandwidth too low
- CPU overloaded
- Wrong framerate

**Solutions:**

**A. Reduce bitrate:**
```bash
ffmpeg -f avfoundation -i "0:0" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -b:v 1000k \  # Limit video bitrate
  -maxrate 1200k -bufsize 2000k \
  -c:a aac -b:a 128k \
  -f flv "rtmp://localhost:1936/live?token=supersecret123"
```

**B. Lower resolution:**
```bash
-video_size 640x480  # Instead of 1280x720
```

**C. Lower framerate:**
```bash
-framerate 24  # Instead of 30
```

## Testing Checklist

Before streaming, verify:

- [ ] Server is running (`cargo run --release`)
- [ ] Port 1936 is not blocked
- [ ] FFmpeg is installed (`ffmpeg -version`)
- [ ] Camera/microphone permissions granted
- [ ] No other apps using camera
- [ ] Correct device numbers in FFmpeg command
- [ ] Using `"0:0"` format for video+audio

## Advanced: OBS Studio Setup

OBS Studio provides a GUI alternative to FFmpeg:

1. **Install OBS Studio**
   - Download from: https://obsproject.com/

2. **Configure Stream Settings:**
   - Settings → Stream
   - Service: Custom
   - Server: `rtmp://localhost:1936/live`
   - Stream Key: `?token=supersecret123`

3. **Configure Output:**
   - Settings → Output
   - Video Encoder: x264
   - Rate Control: CBR
   - Bitrate: 2500 Kbps
   - Preset: veryfast
   - Tune: zerolatency
   - Profile: baseline

4. **Add Sources:**
   - Add → Video Capture Device (for camera)
   - Add → Audio Input Capture (for microphone)

5. **Start Streaming:**
   - Click "Start Streaming"
   - Open: http://localhost:3000/test

## Monitoring

### Check if segments are being created:
```bash
watch -n 1 'ls -lh storage/private/live/'
```

### Monitor FFmpeg output:
The server shows FFmpeg stderr in real-time. Watch for:
- "Opening 'segment00001.ts' for writing"
- "frame= 1234 fps= 30 q=28.0"
- Audio/video stream info

### Check network usage:
```bash
# macOS
nettop -m tcp

# Linux
iftop
```

## File Structure

```
storage/
└── private/
    └── live/
        ├── index.m3u8      # HLS playlist (auto-updated)
        ├── segment00001.ts # Video segment 1
        ├── segment00002.ts # Video segment 2
        └── ...
```

## Port Reference

- **3000** - HTTP server (HLS playback)
- **1936** - RTMP ingest (internal, not exposed externally)

## Security Notes

⚠️ **Important:**
- Change `RTMP_PUBLISH_TOKEN` in `src/main.rs` before production
- Live stream requires login: http://localhost:3000/login
- Use HTTPS in production with proper authentication

## Getting Help

If issues persist:

1. Check FFmpeg output in server terminal
2. Open browser console (F12) and check for errors
3. Visit test page: http://localhost:3000/test
4. Check Debug Log on test page for detailed HLS.js errors

## Performance Tips

For production use:

1. **Use hardware acceleration:**
   ```bash
   # macOS
   -c:v h264_videotoolbox
   
   # Linux with NVIDIA
   -c:v h264_nvenc
   ```

2. **Optimize encoding preset:**
   - `ultrafast` - lowest CPU, worst quality
   - `veryfast` - current default
   - `fast` - better quality, more CPU
   - `medium` - best quality/CPU balance

3. **Enable multi-threading:**
   ```bash
   -threads 4
   ```

4. **Monitor CPU usage:**
   ```bash
   top -pid $(pgrep ffmpeg)
   ```

## FAQ

**Q: Can I stream to multiple viewers?**
A: Yes! The HLS stream can be viewed by unlimited concurrent viewers.

**Q: Can I record the stream?**
A: Yes, add recording to FFmpeg on server side or use OBS's recording feature.

**Q: What's the maximum resolution?**
A: Limited by your CPU. 1080p at 30fps requires significant processing.

**Q: Can I stream from a file instead of camera?**
A: Yes:
```bash
ffmpeg -re -i input.mp4 -c:v libx264 -preset veryfast \
  -f flv "rtmp://localhost:1936/live?token=supersecret123"
```

**Q: How do I enable CORS for my domain?**
A: The server already supports `*.appkask.com` subdomains via the CORS configuration.