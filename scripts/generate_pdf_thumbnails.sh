#!/bin/bash
# Generate thumbnails for PDF documents in the media database

DB_PATH="${1:-media.db}"
STORAGE_BASE="${2:-storage}"

echo "🔍 Finding PDFs without thumbnails..."

# Get all PDF documents from database
sqlite3 "$DB_PATH" -separator '|' \
  "SELECT id, slug, filename, vault_id, user_id
   FROM media_items
   WHERE media_type = 'document'
     AND mime_type = 'application/pdf'" | while IFS='|' read -r id slug filename vault_id user_id; do

  echo "📄 Processing: $slug (ID: $id)"

  # Determine PDF file path
  if [ -n "$vault_id" ]; then
    pdf_path="$STORAGE_BASE/vaults/$vault_id/documents/$filename"
    thumb_dir="$STORAGE_BASE/vaults/$vault_id/thumbnails/documents"
  elif [ -n "$user_id" ]; then
    pdf_path="$STORAGE_BASE/users/$user_id/documents/$filename"
    thumb_dir="$STORAGE_BASE/users/$user_id/thumbnails/documents"
  else
    pdf_path="$STORAGE_BASE/documents/$filename"
    thumb_dir="$STORAGE_BASE/thumbnails/documents"
  fi

  # Check if PDF exists
  if [ ! -f "$pdf_path" ]; then
    echo "  ❌ PDF not found: $pdf_path"
    continue
  fi

  # Create thumbnail directory
  mkdir -p "$thumb_dir"

  # Generate thumbnail filename
  thumb_filename="${slug}_thumb.webp"
  thumb_path="$thumb_dir/$thumb_filename"

  # Skip if thumbnail already exists
  if [ -f "$thumb_path" ]; then
    echo "  ✓ Thumbnail already exists"
    continue
  fi

  # Generate thumbnail using Ghostscript (first page only, 400px width)
  echo "  🖼️  Generating thumbnail..."
  temp_png="/tmp/${slug}_thumb.png"

  gs -dSAFER -dBATCH -dNOPAUSE -dQUIET \
     -sDEVICE=png16m \
     -dFirstPage=1 -dLastPage=1 \
     -dGraphicsAlphaBits=4 -dTextAlphaBits=4 \
     -r150 \
     -sOutputFile="$temp_png" \
     "$pdf_path" 2>/dev/null

  if [ $? -ne 0 ]; then
    echo "  ❌ Failed to generate PNG from PDF"
    continue
  fi

  # Convert PNG to WebP and resize to max 400px width
  if command -v convert >/dev/null 2>&1; then
    # ImageMagick available
    convert "$temp_png" -resize 400x400\> -quality 85 "$thumb_path" 2>/dev/null
  elif command -v sips >/dev/null 2>&1; then
    # macOS sips available
    sips -s format png "$temp_png" --resampleWidth 400 --out "$temp_png" >/dev/null 2>&1
    # Convert PNG to WebP using simple method (keep as PNG if cwebp not available)
    if command -v cwebp >/dev/null 2>&1; then
      cwebp -q 85 "$temp_png" -o "$thumb_path" >/dev/null 2>&1
    else
      # Fallback: keep as PNG
      thumb_filename="${slug}_thumb.png"
      thumb_path="$thumb_dir/$thumb_filename"
      mv "$temp_png" "$thumb_path"
    fi
  else
    # No conversion tool, just copy PNG
    thumb_filename="${slug}_thumb.png"
    thumb_path="$thumb_dir/$thumb_filename"
    mv "$temp_png" "$thumb_path"
  fi

  # Clean up temp file
  rm -f "$temp_png"

  if [ -f "$thumb_path" ]; then
    # Update database with thumbnail URL
    thumb_url="/documents/${slug}/thumbnail"
    sqlite3 "$DB_PATH" \
      "UPDATE media_items SET thumbnail_url = '$thumb_url' WHERE id = $id"

    echo "  ✅ Thumbnail created: $thumb_filename"
  else
    echo "  ❌ Failed to create thumbnail"
  fi
done

echo ""
echo "✨ PDF thumbnail generation complete!"
