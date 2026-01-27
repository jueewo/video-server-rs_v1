# OpenTelemetry 0.26 + SigNoz Integration Fix

## Problem Summary

The video server is experiencing gRPC connection issues when sending telemetry data directly to SigNoz. This is due to:

1. **API Changes**: OpenTelemetry Rust SDK 0.26 has different APIs compared to older versions
2. **Version Compatibility**: Need to ensure all OpenTelemetry crates use compatible versions
3. **gRPC Protocol**: Direct gRPC connections require proper configuration

## Current Configuration Issues

The current `Cargo.toml` has version mismatches and the code uses APIs that don't exist in OpenTelemetry 0.26.

## ✅ Solution: Update to OpenTelemetry 0.26

### Step 1: Update Cargo.toml

The `Cargo.toml` already has the correct versions:

```toml
# OpenTelemetry dependencies & Logging - Latest stable versions
# Note: Using 0.26 for compatibility with tracing-opentelemetry
opentelemetry = "0.26"
opentelemetry_sdk = { version = "0.26", features = ["rt-tokio", "logs", "trace"] }
opentelemetry-otlp = { version = "0.26", features = ["grpc-tonic", "logs", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-opentelemetry = "0.26"
opentelemetry-appender-tracing = "0.26"
```

### Step 2: Update src/main.rs - Fix init_tracer() Function

Replace the existing `init_tracer()` function with this working version:

```rust
fn init_tracer() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::trace::TracerProvider;
    
    println!("🔧 Initializing OpenTelemetry...");

    // Get OTLP endpoint from environment
    let otlp_endpoint = std::env::var("OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    println!("📡 Connecting to OTLP endpoint: {}", otlp_endpoint);

    // Create shared resource
    let resource = opentelemetry_sdk::Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", "video-server"),
    ]);

    // Build trace exporter - OpenTelemetry 0.26 API
    let trace_exporter = opentelemetry_otlp::SpanExporter::new(
        opentelemetry_otlp::TonicExporterBuilder::default()
            .with_endpoint(&otlp_endpoint)
            .with_timeout(std::time::Duration::from_secs(10))
            .build_span_exporter()?,
    );

    // Build tracer provider
    let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(trace_exporter, runtime::Tokio)
        .with_config(
            opentelemetry_sdk::trace::Config::default()
                .with_resource(resource.clone())
        )
        .build();

    // Get tracer from provider
    let tracer = tracer_provider.tracer("video-server");

    println!("✅ Tracer installed successfully");

    // Build log exporter - OpenTelemetry 0.26 API
    let log_exporter = opentelemetry_otlp::LogExporter::new(
        opentelemetry_otlp::TonicExporterBuilder::default()
            .with_endpoint(&otlp_endpoint)
            .with_timeout(std::time::Duration::from_secs(10))
            .build_log_exporter()?,
    );

    // Build logger provider
    let logger_provider = opentelemetry_sdk::logs::LoggerProvider::builder()
        .with_config(
            opentelemetry_sdk::logs::Config::default()
                .with_resource(resource.clone())
        )
        .with_batch_exporter(log_exporter, runtime::Tokio)
        .build();

    println!("✅ Logger provider installed successfully");

    // Create the tracing bridge that sends log events to OTLP
    let otel_log_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    // Create OpenTelemetry tracing layer for spans/traces
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Initialize tracing subscriber with all layers
    match tracing_subscriber::registry()
        .with(telemetry_layer)           // For traces/spans
        .with(otel_log_layer)            // For logs via OTLP
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())  // Console output
        .try_init()
    {
        Ok(_) => println!("✅ Tracing subscriber initialized"),
        Err(e) => {
            println!("❌ Failed to initialize subscriber: {}", e);
            return Err(Box::new(e));
        }
    }

    println!("✅ OpenTelemetry initialized successfully (traces + logs)");
    Ok(())
}
```

### Step 3: Update imports in src/main.rs

Ensure these imports are present at the top of the file:

```rust
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::runtime;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// For OTLP logs bridge
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
```

**Remove** the old import if present:
```rust
// DELETE THIS LINE:
use opentelemetry_sdk::{runtime, trace as sdktrace};
```

### Step 4: Rebuild and Test

```bash
cd video-server-rs_v1

# Clean and rebuild
cargo clean
cargo build --release

# Set environment variables
export ENABLE_OTLP=true
export OTLP_ENDPOINT=http://localhost:4317
export RUST_LOG=info

# Run the server
cargo run --release
```

Expected output:
```
🔧 Initializing OpenTelemetry...
📡 Connecting to OTLP endpoint: http://localhost:4317
✅ Tracer installed successfully
✅ Logger provider installed successfully
✅ Tracing subscriber initialized
✅ OpenTelemetry initialized successfully (traces + logs)
📊 OTLP telemetry enabled
```

## SigNoz Setup

### Option 1: Direct to SigNoz (Recommended for Development)

1. Start SigNoz:
```bash
git clone -b main https://github.com/SigNoz/signoz.git
cd signoz/deploy/
docker compose -f docker/clickhouse-setup/docker-compose.yaml up -d
```

2. Configure `.env`:
```bash
ENABLE_OTLP=true
OTLP_ENDPOINT=http://localhost:4317
RUST_LOG=info
```

3. Access SigNoz UI: http://localhost:3301

### Option 2: Via OpenTelemetry Collector (Recommended for Production)

Use the OTel Collector as a buffer between your app and SigNoz:

1. Create `otel-collector-config.yaml`:
```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

processors:
  batch:
    timeout: 10s
    send_batch_size: 1024

exporters:
  otlp:
    endpoint: signoz-otel-collector:4317
    tls:
      insecure: true

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch]
      exporters: [otlp]
    logs:
      receivers: [otlp]
      processors: [batch]
      exporters: [otlp]
```

2. Run OTel Collector:
```bash
docker run -d --name otel-collector \
  -p 4317:4317 \
  -p 4318:4318 \
  -v $(pwd)/otel-collector-config.yaml:/etc/otel-collector-config.yaml \
  otel/opentelemetry-collector:latest \
  --config=/etc/otel-collector-config.yaml
```

## Verification

### 1. Check Logs
Look for successful initialization:
```bash
✅ Tracer installed successfully
✅ Logger provider installed successfully
✅ Tracing subscriber initialized
```

### 2. Generate Traffic
```bash
# Health check
curl http://localhost:3000/health

# List videos
curl http://localhost:3000/videos

# Create access code (requires auth)
curl -X POST http://localhost:3000/api/access-codes \
  -H "Content-Type: application/json" \
  -d '{
    "code": "test123",
    "description": "Test code",
    "media_items": []
  }'
```

### 3. View in SigNoz
1. Open http://localhost:3301
2. Navigate to **Services** → **video-server**
3. You should see:
   - Traces for HTTP requests
   - Logs with structured fields (user_id, access_code, etc.)
   - Service metrics

## Troubleshooting

### Issue: "Failed to install tracer" Error

**Cause**: Cannot connect to OTLP endpoint

**Solution**:
1. Verify SigNoz is running: `docker ps | grep signoz`
2. Check endpoint is correct: `curl http://localhost:4317` (should connect)
3. Check firewall/network settings

### Issue: "method `new_exporter` not found"

**Cause**: Using old API with OpenTelemetry 0.26

**Solution**: Use the new API shown in this document

### Issue: Version conflict errors

**Cause**: Multiple versions of OpenTelemetry in dependency tree

**Solution**:
```bash
cargo clean
cargo update
cargo build
```

### Issue: No traces appearing in SigNoz

**Cause**: Multiple possibilities

**Solutions**:
1. Check OTLP is enabled: `export ENABLE_OTLP=true`
2. Verify endpoint: Check SigNoz is listening on 4317
3. Check logs: Look for export errors
4. Test with curl:
```bash
# This should work if SigNoz is running
curl -v http://localhost:4317
```

## Key Differences: OpenTelemetry 0.24 → 0.26

| Feature | 0.24 API | 0.26 API |
|---------|----------|----------|
| **Exporter** | `new_exporter().tonic()` | `SpanExporter::new(TonicExporterBuilder)` |
| **Pipeline** | `new_pipeline().tracing()` | Manual `TracerProvider::builder()` |
| **Tracer** | Returned from pipeline | `tracer_provider.tracer()` |
| **Config** | `sdktrace::config()` | `Config::default()` |
| **Resource** | Via `with_resource()` | Via `with_config()` |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ENABLE_OTLP` | `false` | Enable/disable OpenTelemetry export |
| `OTLP_ENDPOINT` | `http://localhost:4317` | gRPC endpoint for OTLP |
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |

## Performance Tuning

### Reduce Overhead
```bash
# Production settings
export RUST_LOG=warn  # Less verbose logging
export OTEL_BSP_MAX_QUEUE_SIZE=2048
export OTEL_BSP_MAX_EXPORT_BATCH_SIZE=512
```

### Sampling (for high-traffic apps)
Modify `init_tracer()` to add sampling:
```rust
use opentelemetry_sdk::trace::Sampler;

let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
    .with_batch_exporter(trace_exporter, runtime::Tokio)
    .with_config(
        opentelemetry_sdk::trace::Config::default()
            .with_sampler(Sampler::TraceIdRatioBased(0.1))  // Sample 10%
            .with_resource(resource.clone())
    )
    .build();
```

## Alternative: HTTP Instead of gRPC

If gRPC continues to have issues, use HTTP transport:

### Update Cargo.toml
```toml
opentelemetry-otlp = { version = "0.26", features = ["http-proto", "logs", "trace"] }
```

### Update init_tracer()
Replace `TonicExporterBuilder` with `HttpExporterBuilder`:
```rust
use opentelemetry_otlp::HttpExporterBuilder;

let trace_exporter = opentelemetry_otlp::SpanExporter::new(
    HttpExporterBuilder::default()
        .with_endpoint("http://localhost:4318/v1/traces")
        .with_timeout(std::time::Duration::from_secs(10))
        .build_span_exporter()?,
);
```

Update endpoint:
```bash
export OTLP_ENDPOINT=http://localhost:4318
```

## Summary

✅ **What Changed**:
- Updated to OpenTelemetry 0.26 API
- Fixed gRPC exporter configuration
- Added proper resource configuration
- Ensured version compatibility across all OTel crates

✅ **What You Get**:
- Working traces in SigNoz
- Structured logs with all custom fields (user_id, access_code, etc.)
- Service metrics and dependency maps
- Both gRPC and HTTP transport options

✅ **Next Steps**:
1. Apply the `init_tracer()` fix from this document
2. Rebuild: `cargo clean && cargo build --release`
3. Test: Start server and check SigNoz UI
4. Monitor: Watch for successful trace/log export

For questions or issues, check the troubleshooting section or SigNoz documentation at https://signoz.io/docs/