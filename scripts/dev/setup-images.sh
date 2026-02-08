#!/bin/bash

# Setup script for image serving functionality
# This script creates directories and generates sample images

set -e

echo "🖼️  Setting up Image Serving"
echo "=============================="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Create directories
echo "📁 Creating storage directories..."
mkdir -p storage/videos/public
mkdir -p storage/videos/private
mkdir -p storage/images/public
mkdir -p storage/images/private

echo -e "${GREEN}✓ Created storage/videos/public${NC}"
echo -e "${GREEN}✓ Created storage/videos/private${NC}"
echo -e "${GREEN}✓ Created storage/images/public${NC}"
echo -e "${GREEN}✓ Created storage/images/private${NC}"
echo ""

# Check if ImageMagick is installed (for generating sample images)
if command -v convert &> /dev/null; then
    echo "🎨 Generating sample images with ImageMagick..."

    # Generate a sample logo (public)
    if [ ! -f "storage/images/public/logo.png" ]; then
        convert -size 400x400 xc:white \
                -fill "#4CAF50" -draw "circle 200,200 200,50" \
                -fill white -pointsize 60 -gravity center -annotate +0+0 "LOGO" \
                storage/images/public/logo.png 2>/dev/null || true

        if [ -f "storage/images/public/logo.png" ]; then
            echo -e "${GREEN}✓ Generated logo.png${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ logo.png already exists${NC}"
    fi

    # Generate a sample banner (public)
    if [ ! -f "storage/images/public/banner.jpg" ]; then
        convert -size 1200x400 gradient:"#1976D2"-"#64B5F6" \
                -fill white -pointsize 80 -gravity center \
                -annotate +0+0 "Welcome Banner" \
                storage/images/public/banner.jpg 2>/dev/null || true

        if [ -f "storage/images/public/banner.jpg" ]; then
            echo -e "${GREEN}✓ Generated banner.jpg${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ banner.jpg already exists${NC}"
    fi

    # Generate a sample private image
    if [ ! -f "storage/images/private/secret.png" ]; then
        convert -size 600x400 xc:"#FF5722" \
                -fill white -pointsize 60 -gravity center \
                -annotate +0-50 "🔒" \
                -pointsize 40 -annotate +0+50 "Confidential" \
                storage/images/private/secret.png 2>/dev/null || true

        if [ -f "storage/images/private/secret.png" ]; then
            echo -e "${GREEN}✓ Generated secret.png${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ secret.png already exists${NC}"
    fi

    echo ""
else
    echo -e "${YELLOW}⚠ ImageMagick not found. Skipping sample image generation.${NC}"
    echo "  Install ImageMagick: brew install imagemagick (macOS)"
    echo "                      apt install imagemagick (Ubuntu/Debian)"
    echo ""
    echo "  Or manually add your own images:"
    echo "    - storage/images/public/logo.png"
    echo "    - storage/images/public/banner.jpg"
    echo "    - storage/images/private/secret.png"
    echo ""
fi

# Reset database to include images table
echo "🗄️  Database setup..."
if [ -f "media.db" ]; then
    read -p "Database exists. Reset it to add images table? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm media.db
        echo -e "${GREEN}✓ Removed old database${NC}"
        echo "  Database will be recreated on next server start"
    else
        echo -e "${YELLOW}⚠ Kept existing database${NC}"
        echo "  Note: Images table may not exist. Check with:"
        echo "  sqlite3 media.db \"SELECT name FROM sqlite_master WHERE type='table';\""
    fi
else
    echo -e "${GREEN}✓ No existing database (will be created on server start)${NC}"
fi
echo ""

# Display directory structure
echo "📂 Directory structure:"
echo "storage/"
echo "├── videos/"
echo "│   ├── public/      (for video files)"
echo "│   └── private/     (for video files)"
echo "├── images/"
echo "│   ├── public/"
if [ -d "storage/images/public" ]; then
    for file in storage/images/public/*; do
        if [ -f "$file" ]; then
            filename=$(basename "$file")
            size=$(ls -lh "$file" | awk '{print $5}')
            echo "│   │   ├── $filename ($size)"
        fi
    done
fi
echo "│   └── private/"
if [ -d "storage/images/private" ]; then
    for file in storage/images/private/*; do
        if [ -f "$file" ]; then
            filename=$(basename "$file")
            size=$(ls -lh "$file" | awk '{print $5}')
            echo "│       └── $filename ($size)"
        fi
    done
fi
echo ""

echo "=============================="
echo -e "${GREEN}✓ Setup Complete!${NC}"
echo ""
echo "Next Steps:"
echo "1. Start the server:     cargo run"
echo "2. Visit gallery:        http://localhost:3000/images"
echo "3. Login:                http://localhost:3000/login"
echo "4. Upload images:        http://localhost:3000/upload"
echo ""
echo "Testing:"
echo "  Run tests:             ./test-images.sh"
echo "  View database:         sqlite3 media.db 'SELECT * FROM images;'"
echo ""
echo "Documentation:"
echo "  Read guide:            cat IMAGE_SERVING_GUIDE.md"
echo ""
