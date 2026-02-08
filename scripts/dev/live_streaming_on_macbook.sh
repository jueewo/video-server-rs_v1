#!/bin/bash

# Start the video server
#

# List available devices
# ffmpeg -f avfoundation -list_devices true -i ""

# Start the live stream
ffmpeg -f avfoundation -framerate 30 -video_size 1280x720 -i "0:0" \
  -c:v libx264 -preset veryfast -tune zerolatency \
  -c:a aac -b:a 128k -ar 44100 \
  -f flv "rtmp://localhost:1936/live?token=supersecret123"


# Test page
# open http://localhost:3000/test.html
