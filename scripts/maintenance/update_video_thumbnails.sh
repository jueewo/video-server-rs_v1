#!/bin/bash

# Update Video Thumbnails in media_items Table
# This script updates the thumbnail_url for all existing videos in the media_items table
# to point to their /hls/{slug}/thumbnail.webp endpoint

DB_FILE="media.db"

echo "🔍 Checking for videos without thumbnail URLs..."

# Count videos without thumbnails
COUNT=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM media_items WHERE media_type='video' AND (thumbnail_url IS NULL OR thumbnail_url = '');")

echo "Found $COUNT videos without thumbnail URLs"

if [ "$COUNT" -eq 0 ]; then
    echo "✅ All videos already have thumbnail URLs!"
    exit 0
fi

echo ""
echo "📝 Updating thumbnail URLs..."

# Update all videos to use the HLS thumbnail endpoint
sqlite3 "$DB_FILE" <<EOF
UPDATE media_items
SET thumbnail_url = '/hls/' || slug || '/thumbnail.webp',
    preview_url = '/hls/' || slug || '/master.m3u8'
WHERE media_type = 'video'
AND (thumbnail_url IS NULL OR thumbnail_url = '');
EOF

# Verify the update
UPDATED=$(sqlite3 "$DB_FILE" "SELECT COUNT(*) FROM media_items WHERE media_type='video' AND thumbnail_url IS NOT NULL AND thumbnail_url != '';")

echo ""
echo "✅ Updated $UPDATED video records"
echo ""
echo "Sample of updated videos:"
sqlite3 "$DB_FILE" "SELECT slug, thumbnail_url FROM media_items WHERE media_type='video' LIMIT 5;"

echo ""
echo "🎉 Done! Video thumbnails have been updated in the media_items table."
