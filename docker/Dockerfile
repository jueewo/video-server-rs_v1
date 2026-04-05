# Multi-stage build for video-server-rs

# Pull bun binary from official image
FROM oven/bun:1-alpine AS bun

FROM rust:1.85-alpine AS builder

# Copy bun binary from official image
COPY --from=bun /usr/local/bin/bun /usr/local/bin/bun

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig

# Set working directory
WORKDIR /app

# Copy package files for CSS build
COPY package.json bun.lock* ./
COPY tailwind.config.ts postcss.config.cjs ./

# Copy static CSS source
COPY static/css/input.css static/css/

# Install Node dependencies and build CSS
RUN bun install && \
    bun run build:css

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Copy source code
COPY src/ ./src/
COPY templates/ ./templates/
COPY migrations/ ./migrations/

# Build Rust application
RUN cargo build --release

# ============================================================================
# Final stage - Runtime image
# ============================================================================
FROM alpine:3.21

# Install runtime dependencies
RUN apk add --no-cache \
    ffmpeg \
    ffmpeg-libs \
    ghostscript \
    libwebp-tools \
    sqlite \
    sqlite-libs \
    ca-certificates \
    tzdata \
    curl \
    wget

# Create app user
RUN addgroup -g 1000 mediaserver && \
    adduser -D -u 1000 -G mediaserver mediaserver

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/video-server-rs /app/video-server-rs

# Copy static files, templates, docs, and migrations
COPY --from=builder /app/static /app/static
COPY templates/ /app/templates/
COPY docs/ /app/docs/
COPY migrations/ /app/migrations/

# Copy site generator static files (used by SITE_COMPONENTS_DIR)
COPY generator/static_files/ /app/generator/static_files/

# Create storage directories (vault-based nested layout)
RUN mkdir -p /app/storage/vaults \
             /app/storage/temp && \
    chown -R mediaserver:mediaserver /app

# Switch to non-root user
USER mediaserver

# Expose port for HTTP server
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Start the Rust server
CMD ["/app/video-server-rs"]
