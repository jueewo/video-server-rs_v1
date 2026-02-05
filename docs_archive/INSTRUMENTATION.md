# OpenTelemetry Instrumentation Guide

This document describes the OpenTelemetry instrumentation added to the video-server-rs project.

## Overview

All HTTP handlers across the project have been instrumented with `#[tracing::instrument]` attributes to provide distributed tracing capabilities. Traces are exported to an OTLP endpoint (default: `http://localhost:4318` for HTTP or `http://localhost:4317` for gRPC).

## Instrumented Components

### Main Application (`src/main.rs`)

- ✅ `index_handler` - Skip: `session`
- ✅ `demo_handler` - Skip: `params`, `state`
- ✅ `health_check`
- ✅ `webhook_stream_ready`
- ✅ `webhook_stream_ended`

### Video Manager (`crates/video-manager/src/lib.rs`)

- ✅ `validate_stream_handler` - Skip: `params`
- ✅ `authorize_stream_handler` - Skip: `session`
- ✅ `videos_list_handler` - Skip: `session`, `state`
- ✅ `video_player_handler` - Skip: `query`, `session`, `state`
- ✅ `live_test_handler` - Skip: `session`
- ✅ `hls_proxy_handler` - Skip: `query`, `session`, `state`
- ✅ `mediamtx_status` - Skip: `state`

### Image Manager (`crates/image-manager/src/lib.rs`)

- ✅ `upload_page_handler` - Skip: `session`
- ✅ `upload_image_handler` - Skip: `session`, `state`, `multipart`
- ✅ `images_gallery_handler` - Skip: `session`, `state`
- ✅ `serve_image_handler` - Skip: `query`, `session`, `state`

### User Auth (`crates/user-auth/src/lib.rs`)

- ✅ `user_profile_handler` - Skip: `state`, `session`
- ✅ `login_page_handler` - Skip: `state`, `session`
- ✅ `oidc_authorize_handler` - Skip: `state`, `query`, `session`
- ✅ `oidc_callback_handler` - Skip: `state`, `query`, `session`
- ✅ `auth_error_handler` - Skip: `query`
- ✅ `emergency_login_form_handler` - Skip: `_state`, `session`
- ✅ `emergency_login_auth_handler` - Skip: `state`, `session`, `form`
- ✅ `logout_handler` - Skip: `session`

### Access Codes (`crates/access-codes/src/lib.rs`)

- ✅ `create_access_code` - Skip: `session`, `state`, `request`
- ✅ `list_access_codes` - Skip: `session`, `state`
- ✅ `delete_access_code` - Skip: `session`, `state`

## Skipped Parameters

Parameters are skipped from automatic tracing to:
1. **Avoid logging sensitive data** (sessions, passwords, tokens)
2. **Prevent Debug trait requirements** for complex state objects
3. **Reduce trace payload size** for large objects (multipart uploads, query params)

## Configuration

### Environment Variables

```bash
# Trace level (default: info)
RUST_LOG=info

# OTLP endpoint (configured in init_tracer function)
# HTTP: http://localhost:4318
# gRPC: http://localhost:4317
```

### Tracer Initialization

The tracer is initialized in `src/main.rs::init_tracer()`:

```rust
fn init_tracer() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = "http://localhost:4318"; // HTTP endpoint
    
    // Attempts to connect to OTLP endpoint
    // Falls back to local logging if unavailable
    // ...
}
```

## Viewing Traces

### With Jaeger

```bash
# Run Jaeger all-in-one
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest

# View traces at http://localhost:16686
```

### With Grafana Tempo

```bash
# Configure Tempo to receive OTLP traces
# View in Grafana at http://localhost:3001
```

### With Zipkin

```bash
# Run Zipkin
docker run -d -p 9411:9411 openzipkin/zipkin

# Configure OTLP receiver in Zipkin
```

## Trace Spans

Each instrumented handler automatically creates a span with:
- **Function name** as the span name
- **Non-skipped parameters** as span attributes
- **Execution time** as span duration
- **Error status** if the handler returns an error

### Example Trace Hierarchy

```
index_handler
├── videos_list_handler
│   ├── get_videos (DB query)
│   └── Template rendering
└── Response

video_player_handler [slug="demo-video"]
├── check_access_code (DB query)
├── hls_proxy_handler [path="demo-video/index.m3u8"]
│   ├── HTTP request to MediaMTX
│   └── Response proxy
└── Template rendering
```

## Benefits

1. **Request Tracing**: Follow a single request across all handlers
2. **Performance Monitoring**: Identify slow handlers and queries
3. **Error Tracking**: Quickly locate failing operations
4. **Debugging**: Understand request flow through the system
5. **Metrics**: Aggregate span data for handler performance stats

## Custom Spans

To add custom instrumentation within handlers:

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(some_param))]
async fn my_handler() -> Result<Response, Error> {
    info!("Starting operation");
    
    // Create a custom span
    let _guard = tracing::info_span!("database_query").entered();
    let result = query_database().await?;
    drop(_guard);
    
    info!("Operation completed");
    Ok(result)
}
```

## Troubleshooting

### No traces appearing

1. Check OTLP endpoint is reachable
2. Verify `RUST_LOG` environment variable is set
3. Check tracer initialization logs on startup
4. Ensure collector is configured to receive OTLP data

### High overhead

1. Reduce `RUST_LOG` level to `warn` or `error`
2. Use sampling to reduce trace volume
3. Skip additional parameters in instrumentation

### Missing spans

1. Ensure handler has `#[tracing::instrument]` attribute
2. Check that tracing subscriber is initialized before handlers run
3. Verify span is within an async context

## References

- [OpenTelemetry Rust SDK](https://github.com/open-telemetry/opentelemetry-rust)
- [Tracing crate](https://docs.rs/tracing)
- [OTLP Specification](https://opentelemetry.io/docs/specs/otlp/)