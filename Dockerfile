# Multi-stage build for video-server-rs
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig \
    nodejs \
    npm

# Set working directory
WORKDIR /app

# Copy package files for CSS build
COPY package.json package-lock.json* ./
COPY tailwind.config.ts postcss.config.cjs ./

# Copy static CSS source
COPY static/css/input.css static/css/

# Install Node dependencies and build CSS
RUN npm install && \
    npm run build:css

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
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    ffmpeg \
    ffmpeg-libs \
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

# Copy static files and templates
COPY --from=builder /app/static /app/static
COPY templates/ /app/templates/

# Create storage directories
RUN mkdir -p /app/storage/images \
             /app/storage/videos \
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
