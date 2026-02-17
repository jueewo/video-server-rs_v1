#!/bin/bash

# Test script for Tailwind CSS 4 and DaisyUI 5 upgrade
# Run this after upgrading to verify everything works

set -e  # Exit on error

echo "=========================================="
echo "Testing Tailwind CSS 4 & DaisyUI 5 Upgrade"
echo "=========================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step 1: Clean install
echo "📦 Step 1: Clean install of dependencies"
echo "------------------------------------------"
rm -rf node_modules package-lock.json
npm install

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ npm install successful${NC}"
else
    echo -e "${RED}❌ npm install failed${NC}"
    exit 1
fi
echo ""

# Step 2: Build CSS
echo "🎨 Step 2: Building Tailwind CSS"
echo "------------------------------------------"
npm run build:css

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ CSS build successful${NC}"
else
    echo -e "${RED}❌ CSS build failed${NC}"
    exit 1
fi
echo ""

# Step 3: Verify output
echo "🔍 Step 3: Verifying CSS output"
echo "------------------------------------------"

if [ -f "static/css/tailwind.css" ]; then
    FILE_SIZE=$(wc -c < "static/css/tailwind.css")
    FILE_SIZE_KB=$((FILE_SIZE / 1024))
    echo -e "${GREEN}✅ tailwind.css exists (${FILE_SIZE_KB}KB)${NC}"

    if [ $FILE_SIZE -gt 10000 ]; then
        echo -e "${GREEN}✅ File size looks good${NC}"
    else
        echo -e "${RED}❌ File size too small (${FILE_SIZE_KB}KB)${NC}"
        exit 1
    fi
else
    echo -e "${RED}❌ tailwind.css not found${NC}"
    exit 1
fi
echo ""

# Step 4: Check for v4 markers
echo "🔍 Step 4: Checking Tailwind v4 features"
echo "------------------------------------------"

# Check if @import is used instead of @tailwind
if grep -q "@import \"tailwindcss\"" static/css/input.css; then
    echo -e "${GREEN}✅ Using Tailwind v4 @import syntax${NC}"
else
    echo -e "${YELLOW}⚠️  Not using @import syntax (may be v3)${NC}"
fi

# Check for ES modules in config
if grep -q "export default" tailwind.config.js; then
    echo -e "${GREEN}✅ Using ES modules in config${NC}"
else
    echo -e "${YELLOW}⚠️  Not using ES modules${NC}"
fi

# Check for module type in package.json
if grep -q "\"type\": \"module\"" package.json; then
    echo -e "${GREEN}✅ Module type set in package.json${NC}"
else
    echo -e "${YELLOW}⚠️  Module type not set${NC}"
fi
echo ""

# Step 5: Verify versions
echo "📋 Step 5: Checking installed versions"
echo "------------------------------------------"

TAILWIND_VERSION=$(npm list tailwindcss --depth=0 2>/dev/null | grep tailwindcss | sed 's/.*@//' | sed 's/ .*//')
DAISYUI_VERSION=$(npm list daisyui --depth=0 2>/dev/null | grep daisyui | sed 's/.*@//' | sed 's/ .*//')

echo "Tailwind CSS: $TAILWIND_VERSION"
echo "DaisyUI: $DAISYUI_VERSION"

if [[ "$TAILWIND_VERSION" == 4.* ]]; then
    echo -e "${GREEN}✅ Tailwind CSS 4.x detected${NC}"
else
    echo -e "${YELLOW}⚠️  Expected Tailwind CSS 4.x, got $TAILWIND_VERSION${NC}"
fi

if [[ "$DAISYUI_VERSION" == 5.* ]]; then
    echo -e "${GREEN}✅ DaisyUI 5.x detected${NC}"
else
    echo -e "${YELLOW}⚠️  Expected DaisyUI 5.x, got $DAISYUI_VERSION${NC}"
fi
echo ""

# Step 6: Test Rust build
echo "🦀 Step 6: Testing Rust build"
echo "------------------------------------------"
cargo build --quiet

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Rust build successful${NC}"
else
    echo -e "${RED}❌ Rust build failed${NC}"
    exit 1
fi
echo ""

# Summary
echo "=========================================="
echo "✨ All tests passed!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Start the server: cargo run"
echo "2. Visit: http://localhost:3000"
echo "3. Verify styling looks correct"
echo "4. Test dark mode toggle"
echo ""
echo "If everything looks good, commit the changes:"
echo "  git add ."
echo "  git commit -m 'Upgrade to Tailwind CSS 4 and DaisyUI 5'"
echo ""
