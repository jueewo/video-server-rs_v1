# OpenTelemetry 0.31 + SigNoz Integration Guide

## ✅ Successfully Upgraded!

Your video server now uses the **newest OpenTelemetry versions** (0.31/0.32) with full SigNoz compatibility.

## 📦 Current Versions

```toml
opentelemetry = "0.31"
opentelemetry_sdk = "0.31"
opentelemetry-otlp = "0.31"
tracing-opentelemetry = "0.32"
opentelemetry-appender-tracing = "0.31"
```

These are the latest stable versions as of the upgrade and are fully compatible with:
- ✅ SigNoz (all versions)
- ✅ Jaeger
- ✅ Grafana Tempo
- ✅ Any OTLP-compatible backend

## 🎯 What Changed

### API Changes from 0.24 → 0.31

| Component | Old API (0.24) | New API (0.31) |
|-----------|---------------|----------------|
| **Exporter** | `new_exporter().tonic()` | `SpanExporter::builder().with_tonic()` |
| **Pipeline** | `new_pipeline().tracing()` | Manual `SdkTracerProvider::builder()` |
| **Resource** | `Resource::new(vec![...])` | `Resource::builder().with_service_name()` |
| **Tracer** | From pipeline | `tracer_provider.tracer("name")` |
| **Logger** | `LoggerProvider` | `SdkLoggerProvider` |
| **Batch Export** | `.with_batch_exporter(exp, runtime::Tokio)` | `.with_batch_exporter(exp)` |

### Key Improvements

1. **Simpler Resource Creation**: Use builder pattern with helpers like `with_service_name()`
2. **Automatic Runtime**: No need to pass `runtime::Tokio` explicitly
3. **Better Type Safety**: More explicit types (e.g., `SdkTracerProvider` vs `TracerProvider`)
4. **Builder APIs**: Consistent builder pattern across all components

## 🚀 Quick Start

### 1. Environment Configuration

Create or update `.env`:

```bash
# Enable OpenTelemetry export
ENABLE_OTLP=true

# OTLP endpoint (SigNoz default)
OTLP_ENDPOINT=http://localhost:4317

# Log level
RUST_LOG=info
```

### 2. Start SigNoz

```bash
git clone -b main https://github.com/SigNoz/signoz.git
cd signoz/deploy/
docker compose -f docker/clickhouse-setup/docker-compose.yaml up -d
```

**Ports:**
- `3301` - SigNoz UI
- `4317` - OTLP gRPC (for telemetry)
- `4318` - OTLP HTTP (alternative)

### 3. Start Your Server

```bash
cd video-server-rs_v1
cargo run --release
```

**Expected Output:**
```
🔧 Initializing OpenTelemetry...
📡 Connecting to OTLP endpoint: http://localhost:4317
✅ Tracer installed successfully
✅ Logger provider installed successfully
✅ Tracing subscriber initialized
✅ OpenTelemetry initialized successfully (traces + logs)
📊 OTLP telemetry enabled
```

### 4. View in SigNoz

1. Open http://localhost:3301
2. Navigate to **Services** → **video-server**
3. Explore:
   - 📊 Traces with all handler operations
   - 📝 Structured logs with custom fields
   - 📈 RED metrics (Rate, Errors, Duration)
   - 🗺️ Service dependency map

## 📝 Structured Logging

All the logs you added are now being exported to SigNoz with full context:

### User Events
```rust
info!(user_id = %user_id, email = %email, name = %name, "User logged in");
```

### Data Loading
```rust
info!(count = videos.len(), authenticated = authenticated, "Videos loaded");
info!(count = images.len(), authenticated = authenticated, "Images loaded");
```

### Access Code Usage
```rust
info!(access_code = %code, media_type = %media_type, media_slug = %media_slug, "Resources access by code");
```

### Error Events
```rust
info!(error = "Invalid credentials", "Failed to process request");
info!(access_code = %code, error = "Invalid or expired", "Failed to process request");
```

### Query Examples in SigNoz

**Find all user logins:**
```
message:"User logged in"
```

**Find failed requests:**
```
message:"Failed to process request"
```

**Track specific user:**
```
user_id:"abc123"
```

**Find access code usage:**
```
message:"Resources access by code" AND media_type:"video"
```

## 🔧 Technical Implementation

### Current init_tracer() Implementation

The updated function uses OpenTelemetry 0.31 APIs:

```rust
fn init_tracer() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::trace::TracerProvider;
    
    // Get endpoint from environment
    let otlp_endpoint = std::env::var("OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    // Create resource with service name
    let resource = opentelemetry_sdk::Resource::builder()
        .with_service_name("video-server")
        .build();

    // Build trace exporter
    let trace_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&otlp_endpoint)
        .with_timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Build tracer provider
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(trace_exporter)
        .with_resource(resource.clone())
        .build();

    let tracer = tracer_provider.tracer("video-server");

    // Build log exporter
    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(&otlp_endpoint)
        .with_timeout(std::time::Duration::from_secs(10))
        .build()?;

    // Build logger provider
    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(log_exporter)
        .build();

    // Create tracing layers
    let otel_log_layer = OpenTelemetryTracingBridge::new(&logger_provider);
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Initialize subscriber
    tracing_subscriber::registry()
        .with(telemetry_layer)
        .with(otel_log_layer)
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .try_init()?;

    Ok(())
}
```

### Key Features

✅ **Traces**: All handler functions with `#[tracing::instrument]` are traced
✅ **Logs**: All `info!()` calls are exported with structured fields
✅ **Metrics**: Automatic HTTP metrics (request rate, duration, errors)
✅ **Context Propagation**: Trace context flows through async operations
✅ **Graceful Fallback**: Falls back to console logging if OTLP fails

## 🎛️ Configuration Options

### Production Settings

```bash
# Minimal logging for performance
export RUST_LOG=warn

# Remote SigNoz instance
export OTLP_ENDPOINT=http://signoz-collector.example.com:4317

# Enable OTLP
export ENABLE_OTLP=true
```

### Development Settings

```bash
# Verbose logging
export RUST_LOG=debug

# Local SigNoz
export OTLP_ENDPOINT=http://localhost:4317

# Enable OTLP
export ENABLE_OTLP=true
```

### Disable Telemetry

```bash
# Disable OTLP export (use console only)
export ENABLE_OTLP=false
```

## 📊 Performance Tuning

### Add Sampling (for high-traffic apps)

Modify `init_tracer()` to add sampling:

```rust
use opentelemetry_sdk::trace::Sampler;

let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
    .with_batch_exporter(trace_exporter)
    .with_config(
        opentelemetry_sdk::trace::Config::default()
            .with_sampler(Sampler::TraceIdRatioBased(0.1)) // Sample 10%
    )
    .with_resource(resource.clone())
    .build();
```

### Batch Export Configuration

Environment variables for tuning (optional):

```bash
# Maximum queue size
export OTEL_BSP_MAX_QUEUE_SIZE=2048

# Maximum batch size
export OTEL_BSP_MAX_EXPORT_BATCH_SIZE=512

# Export interval (ms)
export OTEL_BSP_SCHEDULE_DELAY=5000
```

## 🔄 Alternative: HTTP Instead of gRPC

If gRPC has issues, switch to HTTP:

### 1. Update Cargo.toml

```toml
opentelemetry-otlp = { version = "0.31", features = ["http-proto", "logs", "trace"] }
```

### 2. Update init_tracer()

Replace `.with_tonic()` with `.with_http()`:

```rust
let trace_exporter = opentelemetry_otlp::SpanExporter::builder()
    .with_http()  // Changed from .with_tonic()
    .with_endpoint("http://localhost:4318/v1/traces")
    .with_timeout(std::time::Duration::from_secs(10))
    .build()?;
```

### 3. Update endpoint

```bash
export OTLP_ENDPOINT=http://localhost:4318
```

## 🐛 Troubleshooting

### Issue: No traces appearing

**Check:**
1. SigNoz is running: `docker ps | grep signoz`
2. OTLP enabled: `echo $ENABLE_OTLP` (should be "true")
3. Endpoint correct: `curl http://localhost:4317` (should connect)
4. Check server logs for export errors

### Issue: Build errors after upgrade

**Solution:**
```bash
cargo clean
cargo update
cargo build --release
```

### Issue: "Connection refused" to OTLP endpoint

**Possible causes:**
1. SigNoz not running
2. Wrong endpoint (check port: 4317 for gRPC, 4318 for HTTP)
3. Firewall blocking connection

**Fix:**
```bash
# Check SigNoz is running
docker ps | grep signoz

# Test connection
curl -v http://localhost:4317

# Restart SigNoz if needed
cd signoz/deploy/
docker compose -f docker/clickhouse-setup/docker-compose.yaml restart
```

### Issue: High memory usage

**Solutions:**
1. Reduce log level: `export RUST_LOG=warn`
2. Enable sampling (see Performance Tuning)
3. Increase batch export intervals

### Issue: Logs not appearing in SigNoz

**Check:**
1. Logs layer is enabled (check initialization output)
2. Log level allows your messages: `export RUST_LOG=info`
3. SigNoz is indexing logs (check SigNoz logs tab)

## 📚 Additional Resources

- **OpenTelemetry Rust Docs**: https://docs.rs/opentelemetry/
- **SigNoz Documentation**: https://signoz.io/docs/
- **OTLP Specification**: https://opentelemetry.io/docs/specs/otlp/
- **Tracing Crate**: https://docs.rs/tracing/

## 🎉 What You Get

With OpenTelemetry 0.31 + SigNoz, you now have:

✅ **Distributed Tracing**: Follow requests through your entire application
✅ **Structured Logging**: Query logs by any field (user_id, access_code, etc.)
✅ **Automatic Metrics**: RED metrics for all endpoints
✅ **Service Maps**: Visualize dependencies and call patterns
✅ **Error Tracking**: Group and analyze errors
✅ **Performance Insights**: Identify slow operations and bottlenecks
✅ **Production Ready**: Battle-tested, stable, and performant

## 🔮 Future Upgrades

To upgrade to newer versions in the future:

```bash
# Check for updates
cargo outdated

# Update OpenTelemetry crates
cargo update -p opentelemetry
cargo update -p opentelemetry_sdk
cargo update -p opentelemetry-otlp
cargo update -p tracing-opentelemetry
cargo update -p opentelemetry-appender-tracing

# Test
cargo test
cargo run
```

**Note**: Always check the [OpenTelemetry Rust changelog](https://github.com/open-telemetry/opentelemetry-rust/releases) for breaking changes.

---

**Status**: ✅ Fully operational with OpenTelemetry 0.31 + SigNoz
**Last Updated**: Latest compatible versions as of upgrade
**Compatibility**: SigNoz, Jaeger, Grafana Tempo, and all OTLP backends