#!/bin/bash

#===============================================================================
# build.sh - Server Deployment Build Script
#===============================================================================
#
# This script handles the complete build process for deploying the media server.
# It builds CSS, compiles Rust, and verifies everything is ready for deployment.
#
# Usage:
#   ./scripts/admin/build.sh [options]
#
# Options:
#   --dev          Build for development (faster, not optimized)
#   --release      Build for production (default, optimized)
#   --css-only     Only build CSS, skip Rust compilation
#   --rust-only    Only build Rust, skip CSS build
#   --clean        Clean build artifacts before building
#   --help         Show this help message
#
# Examples:
#   ./scripts/admin/build.sh                    # Full production build
#   ./scripts/admin/build.sh --dev              # Development build
#   ./scripts/admin/build.sh --css-only         # Only rebuild CSS
#   ./scripts/admin/build.sh --clean --release  # Clean build for production
#
#===============================================================================

set -e  # Exit on error

#===============================================================================
# Configuration
#===============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default options
BUILD_MODE="release"
BUILD_CSS=true
BUILD_RUST=true
CLEAN_BUILD=false

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

show_help() {
    cat << EOF
Media Server Build Script

Usage: $0 [options]

Options:
  --dev          Build for development (faster, not optimized)
  --release      Build for production (default, optimized)
  --css-only     Only build CSS, skip Rust compilation
  --rust-only    Only build Rust, skip CSS build
  --clean        Clean build artifacts before building
  --help         Show this help message

Examples:
  $0                           # Full production build
  $0 --dev                     # Development build
  $0 --css-only                # Only rebuild CSS
  $0 --clean --release         # Clean build for production

Important:
  - CSS MUST be built on every deployment (not in git)
  - Production builds use --release for optimization
  - Development builds are faster but larger

For more information, see DEPLOYMENT.md
EOF
}

#===============================================================================
# Parse Arguments
#===============================================================================

while [[ $# -gt 0 ]]; do
    case $1 in
        --dev)
            BUILD_MODE="dev"
            shift
            ;;
        --release)
            BUILD_MODE="release"
            shift
            ;;
        --css-only)
            BUILD_RUST=false
            shift
            ;;
        --rust-only)
            BUILD_CSS=false
            shift
            ;;
        --clean)
            CLEAN_BUILD=true
            shift
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

#===============================================================================
# Main Build Process
#===============================================================================

cd "$PROJECT_ROOT"

print_header "Media Server Build Script"
echo ""
echo "Build Configuration:"
echo "  • Mode: $BUILD_MODE"
echo "  • Build CSS: $BUILD_CSS"
echo "  • Build Rust: $BUILD_RUST"
echo "  • Clean Build: $CLEAN_BUILD"
echo "  • Project Root: $PROJECT_ROOT"
echo ""

#-------------------------------------------------------------------------------
# Step 1: Clean (if requested)
#-------------------------------------------------------------------------------

if [ "$CLEAN_BUILD" = true ]; then
    print_step "Cleaning build artifacts..."

    # Clean Rust artifacts
    if [ "$BUILD_RUST" = true ]; then
        cargo clean
        print_success "Cleaned Rust build artifacts"
    fi

    # Clean CSS
    if [ "$BUILD_CSS" = true ] && [ -f "static/css/tailwind.css" ]; then
        rm -f static/css/tailwind.css
        print_success "Cleaned CSS artifacts"
    fi

    # Clean node_modules (optional, uncomment if needed)
    # rm -rf node_modules

    echo ""
fi

#-------------------------------------------------------------------------------
# Step 2: Check Prerequisites
#-------------------------------------------------------------------------------

print_step "Checking prerequisites..."

# Check Node.js (if building CSS)
if [ "$BUILD_CSS" = true ]; then
    if ! command -v node &> /dev/null; then
        print_error "Node.js not found. Please install Node.js 18+"
        exit 1
    fi
    NODE_VERSION=$(node --version)
    print_success "Node.js $NODE_VERSION found"

    if ! command -v npm &> /dev/null; then
        print_error "npm not found. Please install npm"
        exit 1
    fi
    NPM_VERSION=$(npm --version)
    print_success "npm $NPM_VERSION found"
fi

# Check Rust (if building Rust)
if [ "$BUILD_RUST" = true ]; then
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust"
        exit 1
    fi
    RUST_VERSION=$(rustc --version)
    print_success "$RUST_VERSION found"
fi

# Check FFmpeg
if command -v ffmpeg &> /dev/null; then
    FFMPEG_VERSION=$(ffmpeg -version | head -n1 | cut -d' ' -f3)
    print_success "FFmpeg $FFMPEG_VERSION found"
else
    print_warning "FFmpeg not found (required for video transcoding)"
fi

echo ""

#-------------------------------------------------------------------------------
# Step 3: Build CSS
#-------------------------------------------------------------------------------

if [ "$BUILD_CSS" = true ]; then
    print_step "Building CSS..."

    # Install dependencies
    print_step "Installing Node dependencies..."
    npm install
    print_success "Node dependencies installed"

    # Build CSS
    print_step "Compiling Tailwind CSS..."
    npm run build:css

    # Verify CSS was created
    if [ -f "static/css/tailwind.css" ]; then
        CSS_SIZE=$(du -h static/css/tailwind.css | cut -f1)
        print_success "CSS built successfully ($CSS_SIZE)"
    else
        print_error "CSS build failed - tailwind.css not found"
        exit 1
    fi

    echo ""
fi

#-------------------------------------------------------------------------------
# Step 4: Build Rust
#-------------------------------------------------------------------------------

if [ "$BUILD_RUST" = true ]; then
    print_step "Building Rust application..."

    if [ "$BUILD_MODE" = "release" ]; then
        print_step "Compiling in RELEASE mode (optimized, takes longer)..."
        cargo build --release
        BINARY_PATH="target/release/video-server-rs"
    else
        print_step "Compiling in DEV mode (faster, larger binary)..."
        cargo build
        BINARY_PATH="target/debug/video-server-rs"
    fi

    # Verify binary was created
    if [ -f "$BINARY_PATH" ]; then
        BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
        print_success "Rust binary built successfully ($BINARY_SIZE)"
        print_success "Binary location: $BINARY_PATH"
    else
        print_error "Rust build failed - binary not found"
        exit 1
    fi

    echo ""
fi

#-------------------------------------------------------------------------------
# Step 5: Verify Build
#-------------------------------------------------------------------------------

print_step "Verifying build..."

ERRORS=0

# Check CSS exists
if [ "$BUILD_CSS" = true ] || [ ! "$BUILD_RUST" = false ]; then
    if [ ! -f "static/css/tailwind.css" ]; then
        print_error "CSS file missing: static/css/tailwind.css"
        ERRORS=$((ERRORS + 1))
    else
        print_success "CSS file exists"
    fi
fi

# Check Rust binary exists
if [ "$BUILD_RUST" = true ]; then
    if [ "$BUILD_MODE" = "release" ]; then
        if [ ! -f "target/release/video-server-rs" ]; then
            print_error "Release binary missing"
            ERRORS=$((ERRORS + 1))
        else
            print_success "Release binary exists"
        fi
    else
        if [ ! -f "target/debug/video-server-rs" ]; then
            print_error "Debug binary missing"
            ERRORS=$((ERRORS + 1))
        else
            print_success "Debug binary exists"
        fi
    fi
fi

# Check storage directories exist
if [ ! -d "storage" ]; then
    print_warning "Storage directory missing - creating..."
    mkdir -p storage/images
    mkdir -p storage/videos
    mkdir -p storage/temp
    print_success "Storage directories created"
fi

# Check database exists
if [ ! -f "media.db" ] && [ ! -f "media.db" ]; then
    print_warning "Database not found - you may need to run migrations"
fi

echo ""

#-------------------------------------------------------------------------------
# Step 6: Build Summary
#-------------------------------------------------------------------------------

if [ $ERRORS -eq 0 ]; then
    print_header "Build Complete ✓"
    echo ""
    print_success "All checks passed!"
    echo ""

    echo -e "${CYAN}Next Steps:${NC}"

    if [ "$BUILD_MODE" = "release" ]; then
        echo "  1. Run the server:"
        echo "     ./target/release/video-server-rs"
        echo ""
        echo "  2. Or install as systemd service (see DEPLOYMENT.md)"
    else
        echo "  1. Run the server:"
        echo "     cargo run"
        echo ""
        echo "  2. Or run the binary directly:"
        echo "     ./target/debug/video-server-rs"
    fi

    echo ""
    echo "  3. Access the server:"
    echo "     http://localhost:3000"
    echo ""

    if [ "$BUILD_CSS" = true ]; then
        echo -e "${YELLOW}Remember:${NC} CSS must be rebuilt on every deployment!"
        echo "  (static/css/tailwind.css is not in git)"
        echo ""
    fi

    exit 0
else
    print_header "Build Failed ✗"
    echo ""
    print_error "$ERRORS error(s) found during verification"
    echo ""
    echo "Please fix the errors above and try again."
    echo "For help, see DEPLOYMENT.md or run: $0 --help"
    echo ""
    exit 1
fi
