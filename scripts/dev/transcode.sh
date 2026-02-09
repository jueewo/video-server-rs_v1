#!/bin/bash

MP4FILE=$1
BASENAME="${MP4FILE%.*}"

echo "transcoding file: " $MP4FILE

#exit
if [ -z "$MP4FILE" ]; then
  echo "Usage: $0 <mp4_file>"
  exit 1
fi

OUTPUTDIR='video_'$BASENAME
echo $OUTPUTDIR
mkdir $OUTPUTDIR
mkdir $OUTPUTDIR/segments

# Extract a thumbnail image as fallback (WebP format)
echo "Extracting thumbnail image..."
ffmpeg -i $MP4FILE -ss 00:00:01 -vframes 1 -q:v 2 $OUTPUTDIR/thumbnail.webp


# Transcode to HLS
echo "Transcoding to HLS..."
ffmpeg -i $MP4FILE \
  -c:v h264 -flags +cgop -g 30 -sc_threshold 0 \
  -c:a aac \
  -f hls \
  -hls_time 6 \
  -hls_playlist_type vod \
  -hls_segment_filename "$OUTPUTDIR/segments/%03d.ts" \
  -hls_base_url "segments/" \
  $OUTPUTDIR/master.m3u8

echo "DONE"
