#!/bin/bash

#===============================================================================
# prepare-video.sh - Offline Video Preparation Script
#===============================================================================
#
# This script transcodes a video file to HLS format with multiple quality
# variants, exactly matching the server's upload pipeline behavior.
#
# Usage:
#   ./prepare-video.sh <input-video> <slug> [public|private]
#
# Example:
#   ./prepare-video.sh my-video.mp4 my-awesome-video public
#
# Output:
#   storage/videos/public/my-awesome-video/
#   ├── hls/
#   │   ├── master.m3u8
#   │   ├── 1080p/
#   │   │   ├── index.m3u8
#   │   │   ├── segment_000.ts
#   │   │   ├── segment_001.ts
#   │   │   └── ...
#   │   ├── 720p/
#   │   ├── 480p/
#   │   └── 360p/
#   ├── thumbnail.jpg
#   └── poster.jpg
#
# After running this script, use the "Register Video" button in the UI
# to register the video in the database.
#
#===============================================================================

set -e  # Exit on error

#===============================================================================
# Configuration
#===============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
STORAGE_DIR="${PROJECT_ROOT}/storage/videos"

# FFmpeg settings
FFMPEG_BIN="ffmpeg"
FFPROBE_BIN="ffprobe"
SEGMENT_DURATION=6
THREADS=0  # 0 = auto

# Quality presets (matching hls.rs QUALITY_PRESETS)
declare -A PRESET_1080P=(
    [name]="1080p"
    [width]=1920
    [height]=1080
    [video_bitrate]=5000
    [max_bitrate]=5000
    [buffer_size]=10000
    [audio_bitrate]=128
    [profile]="high"
    [level]="4.0"
)

declare -A PRESET_720P=(
    [name]="720p"
    [width]=1280
    [height]=720
    [video_bitrate]=2800
    [max_bitrate]=2800
    [buffer_size]=5600
    [audio_bitrate]=128
    [profile]="high"
    [level]="3.1"
)

declare -A PRESET_480P=(
    [name]="480p"
    [width]=854
    [height]=480
    [video_bitrate]=1400
    [max_bitrate]=1400
    [buffer_size]=2800
    [audio_bitrate]=96
    [profile]="main"
    [level]="3.0"
)

declare -A PRESET_360P=(
    [name]="360p"
    [width]=640
    [height]=360
    [video_bitrate]=800
    [max_bitrate]=800
    [buffer_size]=1600
    [audio_bitrate]=96
    [profile]="baseline"
    [level]="3.0"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

#===============================================================================
# Helper Functions
#===============================================================================

print_header() {
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
}

print_step() {
    echo -e "${BLUE}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

format_duration() {
    local seconds=$1
    printf "%02d:%02d:%02d" $((seconds/3600)) $((seconds%3600/60)) $((seconds%60))
}

format_bytes() {
    local bytes=$1
    if [ $bytes -ge 1073741824 ]; then
        echo "$(awk "BEGIN {printf \"%.2f\", $bytes/1073741824}") GB"
    elif [ $bytes -ge 1048576 ]; then
        echo "$(awk "BEGIN {printf \"%.2f\", $bytes/1048576}") MB"
    elif [ $bytes -ge 1024 ]; then
        echo "$(awk "BEGIN {printf \"%.2f\", $bytes/1024}") KB"
    else
        echo "$bytes bytes"
    fi
}

check_dependencies() {
    print_step "Checking dependencies..."

    if ! command -v $FFMPEG_BIN &> /dev/null; then
        print_error "ffmpeg not found. Please install FFmpeg."
        exit 1
    fi

    if ! command -v $FFPROBE_BIN &> /dev/null; then
        print_error "ffprobe not found. Please install FFmpeg."
        exit 1
    fi

    # Check ffmpeg version
    local ffmpeg_version=$($FFMPEG_BIN -version | head -n1 | cut -d' ' -f3)
    print_success "ffmpeg $ffmpeg_version found"

    # Check for required encoders
    if ! $FFMPEG_BIN -encoders 2>/dev/null | grep -q libx264; then
        print_error "libx264 encoder not found. Please install FFmpeg with H.264 support."
        exit 1
    fi

    print_success "All dependencies OK"
}

extract_metadata() {
    local input_file=$1

    print_step "Extracting video metadata..."

    # Get video info using ffprobe
    local probe_output=$($FFPROBE_BIN -v quiet -print_format json -show_format -show_streams "$input_file")

    # Extract video stream info
    VIDEO_WIDTH=$(echo "$probe_output" | grep -m1 '"width"' | grep -o '[0-9]\+')
    VIDEO_HEIGHT=$(echo "$probe_output" | grep -m1 '"height"' | grep -o '[0-9]\+')
    DURATION=$(echo "$probe_output" | grep -m1 '"duration"' | cut -d'"' -f4 | cut -d'.' -f1)
    FILE_SIZE=$(stat -f%z "$input_file" 2>/dev/null || stat -c%s "$input_file" 2>/dev/null)

    # Get codec info
    VIDEO_CODEC=$(echo "$probe_output" | grep -A20 '"codec_type": "video"' | grep -m1 '"codec_name"' | cut -d'"' -f4)
    AUDIO_CODEC=$(echo "$probe_output" | grep -A20 '"codec_type": "audio"' | grep -m1 '"codec_name"' | cut -d'"' -f4)

    print_success "Resolution: ${VIDEO_WIDTH}x${VIDEO_HEIGHT}"
    print_success "Duration: $(format_duration $DURATION)"
    print_success "File Size: $(format_bytes $FILE_SIZE)"
    print_success "Video Codec: $VIDEO_CODEC"
    print_success "Audio Codec: ${AUDIO_CODEC:-none}"
}

select_qualities() {
    print_step "Selecting quality presets based on source resolution..."

    QUALITIES=()

    # 1080p (1920x1080)
    if [ $VIDEO_WIDTH -ge 1920 ] && [ $VIDEO_HEIGHT -ge 1080 ]; then
        QUALITIES+=("1080p")
        print_success "Including 1080p"
    fi

    # 720p (1280x720)
    if [ $VIDEO_WIDTH -ge 1280 ] && [ $VIDEO_HEIGHT -ge 720 ]; then
        QUALITIES+=("720p")
        print_success "Including 720p"
    fi

    # 480p (854x480)
    if [ $VIDEO_WIDTH -ge 854 ] && [ $VIDEO_HEIGHT -ge 480 ]; then
        QUALITIES+=("480p")
        print_success "Including 480p"
    fi

    # 360p (640x360)
    if [ $VIDEO_WIDTH -ge 640 ] && [ $VIDEO_HEIGHT -ge 360 ]; then
        QUALITIES+=("360p")
        print_success "Including 360p"
    fi

    if [ ${#QUALITIES[@]} -eq 0 ]; then
        print_error "Source resolution too small for any quality preset"
        exit 1
    fi

    echo -e "${GREEN}Selected ${#QUALITIES[@]} quality variants: ${QUALITIES[*]}${NC}"
}

transcode_quality() {
    local input_file=$1
    local output_dir=$2
    local quality=$3

    # Get preset variables
    local preset_var="PRESET_${quality}"
    declare -n preset=$preset_var

    local quality_name=${preset[name]}
    local width=${preset[width]}
    local height=${preset[height]}
    local video_bitrate=${preset[video_bitrate]}
    local max_bitrate=${preset[max_bitrate]}
    local buffer_size=${preset[buffer_size]}
    local audio_bitrate=${preset[audio_bitrate]}
    local profile=${preset[profile]}
    local level=${preset[level]}

    print_step "Transcoding $quality_name ($width×$height, ${video_bitrate}k video, ${audio_bitrate}k audio)..."

    # Create quality directory
    local quality_dir="$output_dir/$quality_name"
    mkdir -p "$quality_dir"

    # Output paths
    local playlist_path="$quality_dir/index.m3u8"
    local segment_pattern="$quality_dir/segment_%03d.ts"

    # Transcode with progress
    $FFMPEG_BIN \
        -i "$input_file" \
        -c:v libx264 \
        -preset medium \
        -profile:v "$profile" \
        -level "$level" \
        -vf "scale=${width}:${height}:force_original_aspect_ratio=decrease,pad=${width}:${height}:(ow-iw)/2:(oh-ih)/2" \
        -b:v "${video_bitrate}k" \
        -maxrate "${max_bitrate}k" \
        -bufsize "${buffer_size}k" \
        -c:a aac \
        -b:a "${audio_bitrate}k" \
        -ar 44100 \
        -ac 2 \
        -f hls \
        -hls_time $SEGMENT_DURATION \
        -hls_playlist_type vod \
        -hls_segment_type mpegts \
        -hls_segment_filename "$segment_pattern" \
        -threads $THREADS \
        -y \
        "$playlist_path" \
        2>&1 | grep -E "time=|error" || true

    # Check if successful
    if [ ! -f "$playlist_path" ]; then
        print_error "$quality_name transcoding failed"
        return 1
    fi

    # Count segments
    local segment_count=$(ls "$quality_dir"/*.ts 2>/dev/null | wc -l | tr -d ' ')
    print_success "$quality_name complete: $segment_count segments"

    return 0
}

generate_master_playlist() {
    local output_dir=$1
    shift
    local qualities=("$@")

    print_step "Generating master playlist..."

    local master_path="$output_dir/master.m3u8"

    # Start playlist
    echo "#EXTM3U" > "$master_path"
    echo "#EXT-X-VERSION:3" >> "$master_path"

    # Add each quality variant
    for quality in "${qualities[@]}"; do
        local preset_var="PRESET_${quality}"
        declare -n preset=$preset_var

        local bandwidth=$(( (${preset[video_bitrate]} + ${preset[audio_bitrate]}) * 1000 ))
        local resolution="${preset[width]}x${preset[height]}"

        echo "#EXT-X-STREAM-INF:BANDWIDTH=$bandwidth,RESOLUTION=$resolution" >> "$master_path"
        echo "${preset[name]}/index.m3u8" >> "$master_path"
    done

    print_success "Master playlist created: $master_path"
}

generate_thumbnail() {
    local input_file=$1
    local output_file=$2
    local timestamp=$3

    print_step "Generating thumbnail at ${timestamp}s..."

    # Format timestamp
    local hours=$((timestamp / 3600))
    local minutes=$(((timestamp % 3600) / 60))
    local seconds=$((timestamp % 60))
    local ts=$(printf "%02d:%02d:%02d" $hours $minutes $seconds)

    # Generate thumbnail (400x225 for 16:9)
    $FFMPEG_BIN \
        -ss "$ts" \
        -i "$input_file" \
        -vframes 1 \
        -vf "scale=400:225:force_original_aspect_ratio=decrease,pad=400:225:(ow-iw)/2:(oh-ih)/2" \
        -q:v 2 \
        -y \
        "$output_file" \
        2>/dev/null

    if [ -f "$output_file" ]; then
        print_success "Thumbnail created: $output_file"
    else
        print_warning "Thumbnail generation failed (non-fatal)"
    fi
}

generate_poster() {
    local input_file=$1
    local output_file=$2
    local timestamp=$3

    print_step "Generating poster at ${timestamp}s..."

    # Format timestamp
    local hours=$((timestamp / 3600))
    local minutes=$(((timestamp % 3600) / 60))
    local seconds=$((timestamp % 60))
    local ts=$(printf "%02d:%02d:%02d" $hours $minutes $seconds)

    # Generate poster (1280x720 max)
    $FFMPEG_BIN \
        -ss "$ts" \
        -i "$input_file" \
        -vframes 1 \
        -vf "scale=1280:720:force_original_aspect_ratio=decrease" \
        -q:v 2 \
        -y \
        "$output_file" \
        2>/dev/null

    if [ -f "$output_file" ]; then
        print_success "Poster created: $output_file"
    else
        print_warning "Poster generation failed (non-fatal)"
    fi
}

print_usage() {
    cat << EOF
Usage: $0 <input-video> <slug> [visibility]

Arguments:
  input-video    Path to the video file to process
  slug           URL-friendly slug for the video (e.g., my-awesome-video)
  visibility     'public' or 'private' (default: public)

Examples:
  $0 my-video.mp4 my-awesome-video
  $0 ~/videos/tutorial.mov tutorial-video public
  $0 /path/to/video.mkv private-video private

Output Structure:
  storage/videos/{public|private}/{slug}/
  ├── hls/
  │   ├── master.m3u8        # Master playlist
  │   ├── 1080p/             # 1080p variant (if source >= 1080p)
  │   ├── 720p/              # 720p variant (if source >= 720p)
  │   ├── 480p/              # 480p variant (if source >= 480p)
  │   └── 360p/              # 360p variant (if source >= 360p)
  ├── thumbnail.jpg          # Small thumbnail (400x225)
  └── poster.jpg             # Large poster (1280x720)

After preparation:
  1. Go to http://localhost:3000/videos/new
  2. Select the folder from dropdown
  3. Fill in metadata
  4. Click "Register Video"

EOF
}

#===============================================================================
# Main Script
#===============================================================================

print_header "Video Preparation Script for HLS Streaming"

# Parse arguments
if [ $# -lt 2 ]; then
    print_error "Missing required arguments"
    echo ""
    print_usage
    exit 1
fi

INPUT_FILE="$1"
SLUG="$2"
VISIBILITY="${3:-public}"

# Validate arguments
if [ ! -f "$INPUT_FILE" ]; then
    print_error "Input file not found: $INPUT_FILE"
    exit 1
fi

if [[ ! "$SLUG" =~ ^[a-z0-9-]+$ ]]; then
    print_error "Slug must contain only lowercase letters, numbers, and hyphens"
    print_error "Got: $SLUG"
    exit 1
fi

if [ "$VISIBILITY" != "public" ] && [ "$VISIBILITY" != "private" ]; then
    print_error "Visibility must be 'public' or 'private'"
    print_error "Got: $VISIBILITY"
    exit 1
fi

# Display input info
echo ""
echo -e "${CYAN}Input File:${NC}    $INPUT_FILE"
echo -e "${CYAN}Slug:${NC}          $SLUG"
echo -e "${CYAN}Visibility:${NC}    $VISIBILITY"
echo ""

# Check dependencies
check_dependencies
echo ""

# Extract metadata
extract_metadata "$INPUT_FILE"
echo ""

# Select qualities
select_qualities
echo ""

# Create output directory
OUTPUT_DIR="$STORAGE_DIR/$VISIBILITY/$SLUG"
HLS_DIR="$OUTPUT_DIR/hls"

if [ -d "$OUTPUT_DIR" ]; then
    print_warning "Output directory already exists: $OUTPUT_DIR"
    read -p "Do you want to overwrite? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_error "Aborted"
        exit 1
    fi
    rm -rf "$OUTPUT_DIR"
fi

mkdir -p "$HLS_DIR"
print_success "Created output directory: $OUTPUT_DIR"
echo ""

# Start timer
START_TIME=$(date +%s)

# Generate thumbnails and poster
THUMBNAIL_TIMESTAMP=$((DURATION / 4))  # 25% into video
POSTER_TIMESTAMP=$((DURATION / 2))     # 50% into video

generate_thumbnail "$INPUT_FILE" "$OUTPUT_DIR/thumbnail.jpg" $THUMBNAIL_TIMESTAMP
generate_poster "$INPUT_FILE" "$OUTPUT_DIR/poster.jpg" $POSTER_TIMESTAMP
echo ""

# Transcode to HLS
print_header "HLS Transcoding"
echo ""

SUCCESSFUL_QUALITIES=()
FAILED_QUALITIES=()

for quality in "${QUALITIES[@]}"; do
    if transcode_quality "$INPUT_FILE" "$HLS_DIR" "${quality^^}"; then
        SUCCESSFUL_QUALITIES+=("$quality")
    else
        FAILED_QUALITIES+=("$quality")
        print_warning "$quality transcoding failed, continuing with other qualities"
    fi
    echo ""
done

# Check if at least one quality succeeded
if [ ${#SUCCESSFUL_QUALITIES[@]} -eq 0 ]; then
    print_error "All quality transcoding attempts failed"
    rm -rf "$OUTPUT_DIR"
    exit 1
fi

# Generate master playlist
generate_master_playlist "$HLS_DIR" "${SUCCESSFUL_QUALITIES[@]}"
echo ""

# Calculate total time
END_TIME=$(date +%s)
TOTAL_TIME=$((END_TIME - START_TIME))

# Print summary
print_header "Preparation Complete"
echo ""
echo -e "${GREEN}✓ Video prepared successfully!${NC}"
echo ""
echo -e "${CYAN}Summary:${NC}"
echo -e "  Output Directory:  $OUTPUT_DIR"
echo -e "  Qualities:         ${SUCCESSFUL_QUALITIES[*]}"
if [ ${#FAILED_QUALITIES[@]} -gt 0 ]; then
    echo -e "  ${YELLOW}Failed Qualities:  ${FAILED_QUALITIES[*]}${NC}"
fi
echo -e "  Processing Time:   $(format_duration $TOTAL_TIME)"
echo ""
echo -e "${CYAN}Next Steps:${NC}"
echo -e "  1. Start the server: ${BLUE}cargo run${NC}"
echo -e "  2. Navigate to:     ${BLUE}http://localhost:3000/videos/new${NC}"
echo -e "  3. Select folder:   ${BLUE}$SLUG${NC}"
echo -e "  4. Fill metadata and click 'Register Video'"
echo ""
echo -e "${CYAN}Directory Structure:${NC}"
tree -L 3 "$OUTPUT_DIR" 2>/dev/null || find "$OUTPUT_DIR" -type f | head -20
echo ""

# Show file sizes
echo -e "${CYAN}Quality Sizes:${NC}"
for quality in "${SUCCESSFUL_QUALITIES[@]}"; do
    local quality_dir="$HLS_DIR/$quality"
    if [ -d "$quality_dir" ]; then
        local quality_size=$(du -sh "$quality_dir" | cut -f1)
        local segment_count=$(ls "$quality_dir"/*.ts 2>/dev/null | wc -l | tr -d ' ')
        echo -e "  ${quality}: ${quality_size} ($segment_count segments)"
    fi
done
echo ""

print_success "Ready to register in UI!"
echo ""

exit 0
