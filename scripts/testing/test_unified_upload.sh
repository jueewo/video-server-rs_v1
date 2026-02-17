#!/bin/bash
# Test script for unified media upload

echo "🧪 Testing Unified Media Upload System"
echo "========================================"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
BASE_URL="${BASE_URL:-http://localhost:3000}"
COOKIES_FILE="/tmp/media_test_cookies.txt"

echo "📍 Server: $BASE_URL"
echo ""

# Step 1: Login (if authentication required)
echo "1️⃣  Checking authentication..."
if [ -n "$USERNAME" ] && [ -n "$PASSWORD" ]; then
    echo "   Logging in as $USERNAME..."
    curl -s -c "$COOKIES_FILE" -X POST "$BASE_URL/login" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}" > /dev/null

    if [ $? -eq 0 ]; then
        echo -e "   ${GREEN}✓${NC} Login successful"
    else
        echo -e "   ${RED}✗${NC} Login failed"
        exit 1
    fi
else
    echo -e "   ${YELLOW}⚠${NC}  Skipping authentication (set USERNAME and PASSWORD env vars if needed)"
    touch "$COOKIES_FILE"
fi
echo ""

# Step 2: Create a test image
echo "2️⃣  Creating test image..."
TEST_IMAGE="/tmp/test_upload_image.png"

# Create a simple PNG using ImageMagick (if available) or just touch a file
if command -v convert &> /dev/null; then
    convert -size 800x600 xc:blue -pointsize 40 -fill white \
        -gravity center -annotate +0+0 'Test Upload Image' "$TEST_IMAGE" 2>/dev/null
    if [ $? -eq 0 ]; then
        echo -e "   ${GREEN}✓${NC} Test image created with ImageMagick"
    else
        echo -e "   ${YELLOW}⚠${NC}  ImageMagick failed, creating placeholder"
        echo "fake image data" > "$TEST_IMAGE"
    fi
else
    echo -e "   ${YELLOW}⚠${NC}  ImageMagick not found, creating placeholder"
    # Create a minimal valid PNG
    echo -ne '\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\x00\x01\x00\x00\x05\x00\x01\r\n-\xb4\x00\x00\x00\x00IEND\xaeB`\x82' > "$TEST_IMAGE"
fi
echo ""

# Step 3: Upload via unified endpoint
echo "3️⃣  Uploading image via /api/media/upload..."
UPLOAD_RESPONSE=$(curl -s -b "$COOKIES_FILE" -X POST "$BASE_URL/api/media/upload" \
    -F "media_type=image" \
    -F "title=Test Upload $(date +%s)" \
    -F "description=Automated test upload" \
    -F "is_public=1" \
    -F "category=test" \
    -F "tags=automation,test,demo" \
    -F "file=@$TEST_IMAGE")

echo "$UPLOAD_RESPONSE" | jq '.' 2>/dev/null || echo "$UPLOAD_RESPONSE"

# Check if upload was successful
if echo "$UPLOAD_RESPONSE" | grep -q '"success":true'; then
    echo -e "${GREEN}✓${NC} Upload successful!"

    # Extract slug from response
    SLUG=$(echo "$UPLOAD_RESPONSE" | jq -r '.slug' 2>/dev/null)
    WEBP_URL=$(echo "$UPLOAD_RESPONSE" | jq -r '.webp_url' 2>/dev/null)
    THUMB_URL=$(echo "$UPLOAD_RESPONSE" | jq -r '.thumbnail_url' 2>/dev/null)

    echo ""
    echo "   📝 Slug: $SLUG"
    echo "   🖼️  WebP URL: $WEBP_URL"
    echo "   🔍 Thumbnail URL: $THUMB_URL"
    echo ""

    # Step 4: Test serving endpoints
    echo "4️⃣  Testing image serving endpoints..."

    # Test WebP
    echo -n "   Testing WebP serving... "
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/images/$SLUG")
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓${NC} GET /images/$SLUG → 200 OK"
    else
        echo -e "${RED}✗${NC} GET /images/$SLUG → $HTTP_CODE"
    fi

    # Test original
    echo -n "   Testing original serving... "
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/images/$SLUG/original")
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓${NC} GET /images/$SLUG/original → 200 OK"
    else
        echo -e "${RED}✗${NC} GET /images/$SLUG/original → $HTTP_CODE"
    fi

    # Test thumbnail
    echo -n "   Testing thumbnail serving... "
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/images/$SLUG/thumb")
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓${NC} GET /images/$SLUG/thumb → 200 OK"
    else
        echo -e "${RED}✗${NC} GET /images/$SLUG/thumb → $HTTP_CODE"
    fi

    # Test WebP explicit
    echo -n "   Testing .webp extension... "
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/images/$SLUG.webp")
    if [ "$HTTP_CODE" = "200" ]; then
        echo -e "${GREEN}✓${NC} GET /images/$SLUG.webp → 200 OK"
    else
        echo -e "${RED}✗${NC} GET /images/$SLUG.webp → $HTTP_CODE"
    fi

    echo ""
    echo "5️⃣  Checking database..."
    echo "   Run this to verify:"
    echo "   sqlite3 media.db \"SELECT slug, media_type, title, webp_url, thumbnail_url FROM media_items WHERE slug='$SLUG';\""

else
    echo -e "${RED}✗${NC} Upload failed!"
    echo ""
    echo "Response:"
    echo "$UPLOAD_RESPONSE"
fi

echo ""
echo "🧹 Cleanup..."
rm -f "$TEST_IMAGE" "$COOKIES_FILE"
echo -e "${GREEN}✓${NC} Test complete!"
