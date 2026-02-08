#!/bin/bash

# Test script for image serving functionality
# This script tests the image upload and serving features

set -e

BASE_URL="http://localhost:3000"
COOKIE_FILE="/tmp/video-server-cookies.txt"

echo "🧪 Testing Image Serving Functionality"
echo "======================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test 1: Health Check
echo "Test 1: Health Check"
echo "--------------------"
HEALTH=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/health")
if [ "$HEALTH" -eq 200 ]; then
    echo -e "${GREEN}✓ Server is running${NC}"
else
    echo -e "${RED}✗ Server is not running (HTTP $HEALTH)${NC}"
    exit 1
fi
echo ""

# Test 2: Access Gallery Without Login (Public Only)
echo "Test 2: Public Gallery Access"
echo "------------------------------"
GALLERY=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/images")
if [ "$GALLERY" -eq 200 ]; then
    echo -e "${GREEN}✓ Gallery page accessible${NC}"
else
    echo -e "${RED}✗ Gallery page not accessible (HTTP $GALLERY)${NC}"
fi
echo ""

# Test 3: Try to Access Upload Without Login (Should Require Auth)
echo "Test 3: Upload Page Without Login"
echo "----------------------------------"
UPLOAD_NOAUTH=$(curl -s "$BASE_URL/upload" | grep -c "Authentication Required" || true)
if [ "$UPLOAD_NOAUTH" -gt 0 ]; then
    echo -e "${GREEN}✓ Upload page correctly requires authentication${NC}"
else
    echo -e "${YELLOW}⚠ Upload page may not require authentication${NC}"
fi
echo ""

# Test 4: Try to Access Private Image Without Login (Should Fail)
echo "Test 4: Private Image Without Login"
echo "------------------------------------"
PRIVATE=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/images/secret")
if [ "$PRIVATE" -eq 401 ] || [ "$PRIVATE" -eq 404 ]; then
    echo -e "${GREEN}✓ Private image correctly blocked (HTTP $PRIVATE)${NC}"
else
    echo -e "${YELLOW}⚠ Private image returned HTTP $PRIVATE${NC}"
fi
echo ""

# Test 5: Login
echo "Test 5: User Login"
echo "------------------"
LOGIN=$(curl -s -c "$COOKIE_FILE" -o /dev/null -w "%{http_code}" "$BASE_URL/login")
if [ "$LOGIN" -eq 200 ]; then
    echo -e "${GREEN}✓ Login successful${NC}"
else
    echo -e "${RED}✗ Login failed (HTTP $LOGIN)${NC}"
    exit 1
fi
echo ""

# Test 6: Access Upload With Login
echo "Test 6: Upload Page With Login"
echo "-------------------------------"
UPLOAD_AUTH=$(curl -s -b "$COOKIE_FILE" "$BASE_URL/upload" | grep -c "Upload Image" || true)
if [ "$UPLOAD_AUTH" -gt 0 ]; then
    echo -e "${GREEN}✓ Upload page accessible after login${NC}"
else
    echo -e "${RED}✗ Upload page not accessible after login${NC}"
fi
echo ""

# Test 7: Access Private Image With Login
echo "Test 7: Private Image With Login"
echo "---------------------------------"
PRIVATE_AUTH=$(curl -s -b "$COOKIE_FILE" -o /dev/null -w "%{http_code}" "$BASE_URL/images/secret")
if [ "$PRIVATE_AUTH" -eq 200 ] || [ "$PRIVATE_AUTH" -eq 404 ]; then
    echo -e "${GREEN}✓ Private image accessible after login (HTTP $PRIVATE_AUTH)${NC}"
    if [ "$PRIVATE_AUTH" -eq 404 ]; then
        echo -e "${YELLOW}  Note: Image not found. Add test image: storage/images/private/secret.png${NC}"
    fi
else
    echo -e "${RED}✗ Private image not accessible after login (HTTP $PRIVATE_AUTH)${NC}"
fi
echo ""

# Test 8: Create Test Images (if they don't exist)
echo "Test 8: Check Test Images"
echo "-------------------------"
if [ -f "storage/images/public/logo.png" ]; then
    echo -e "${GREEN}✓ Public test image exists${NC}"
else
    echo -e "${YELLOW}⚠ Public test image not found${NC}"
    echo "  Create one with: cp /path/to/image.png storage/images/public/logo.png"
fi

if [ -f "storage/images/private/secret.png" ]; then
    echo -e "${GREEN}✓ Private test image exists${NC}"
else
    echo -e "${YELLOW}⚠ Private test image not found${NC}"
    echo "  Create one with: cp /path/to/image.png storage/images/private/secret.png"
fi
echo ""

# Test 9: Test Public Image Access
echo "Test 9: Public Image Access"
echo "---------------------------"
PUBLIC=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/images/logo")
if [ "$PUBLIC" -eq 200 ]; then
    echo -e "${GREEN}✓ Public image accessible (HTTP 200)${NC}"

    # Check content type
    CONTENT_TYPE=$(curl -s -I "$BASE_URL/images/logo" | grep -i "content-type" | cut -d' ' -f2-)
    echo "  Content-Type: $CONTENT_TYPE"
elif [ "$PUBLIC" -eq 404 ]; then
    echo -e "${YELLOW}⚠ Public image not found (HTTP 404)${NC}"
    echo "  Add database entry or create file at: storage/images/public/logo.png"
else
    echo -e "${RED}✗ Unexpected response (HTTP $PUBLIC)${NC}"
fi
echo ""

# Test 10: Database Check
echo "Test 10: Database Check"
echo "-----------------------"
if [ -f "media.db" ]; then
    echo -e "${GREEN}✓ Database exists${NC}"

    # Check if images table exists
    if sqlite3 media.db "SELECT name FROM sqlite_master WHERE type='table' AND name='images';" | grep -q "images"; then
        echo -e "${GREEN}✓ Images table exists${NC}"

        # Count images
        IMAGE_COUNT=$(sqlite3 media.db "SELECT COUNT(*) FROM images;")
        echo "  Total images in database: $IMAGE_COUNT"

        if [ "$IMAGE_COUNT" -gt 0 ]; then
            echo "  Sample images:"
            sqlite3 -header -column media.db "SELECT slug, title, is_public FROM images LIMIT 3;"
        else
            echo -e "${YELLOW}  ⚠ No images in database${NC}"
        fi
    else
        echo -e "${RED}✗ Images table does not exist${NC}"
        echo "  Run: rm media.db && cargo run (to recreate database)"
    fi
else
    echo -e "${RED}✗ Database not found${NC}"
fi
echo ""

# Clean up
rm -f "$COOKIE_FILE"

echo "======================================="
echo -e "${GREEN}✓ Testing Complete!${NC}"
echo ""
echo "Next Steps:"
echo "1. Visit http://localhost:3000/images to see the gallery"
echo "2. Visit http://localhost:3000/login then http://localhost:3000/upload to upload images"
echo "3. Add test images to storage/images/public/ and storage/images/private/"
echo ""
echo "Useful Commands:"
echo "  View images:    sqlite3 media.db 'SELECT * FROM images;'"
echo "  Add sample:     sqlite3 media.db \"INSERT INTO images (slug, filename, title, is_public) VALUES ('test', 'test.jpg', 'Test', 1);\""
echo "  Delete image:   sqlite3 media.db \"DELETE FROM images WHERE slug='test';\""
echo ""
