# Observability Quick Start Guide

This guide will help you quickly set up observability for your video-server-rs application using OpenTelemetry with **Vector + SigNoz** (recommended) or alternative backends.

## Prerequisites

- Docker installed on your system
- video-server-rs application built and ready to run

## Quick Setup - Vector + SigNoz (Recommended)

### Step 1: Start SigNoz

SigNoz provides a complete observability platform with traces, metrics, and logs.

```bash
# Clone SigNoz
git clone -b main https://github.com/SigNoz/signoz.git
cd signoz/deploy/

# Start SigNoz using Docker Compose
docker compose -f docker/clickhouse-setup/docker-compose.yaml up -d
```

**Ports:**
- `3301` - SigNoz UI
- `4317` - OTLP gRPC receiver
- `4318` - OTLP HTTP receiver (used by this app)

### Step 2: Configure Vector (Optional but Recommended)

Vector acts as a data pipeline between your app and SigNoz, providing buffering and transformation.

Create `vector.toml`:

```toml
[sources.otlp_http]
type = "http_server"
address = "0.0.0.0:4318"
path = "/v1/traces"
decoding.codec = "bytes"

[sources.otlp_grpc]
type = "http_server"
address = "0.0.0.0:4317"
decoding.codec = "bytes"

[sinks.signoz]
type = "http"
inputs = ["otlp_http", "otlp_grpc"]
uri = "http://localhost:4318/v1/traces"
encoding.codec = "json"

[sinks.console]
type = "console"
inputs = ["otlp_http", "otlp_grpc"]
encoding.codec = "json"
```

Run Vector:

```bash
docker run -d --name vector \
  -p 4317:4317 \
  -p 4318:4318 \
  -v $(pwd)/vector.toml:/etc/vector/vector.toml \
  timberio/vector:latest-alpine
```

> **Note**: If not using Vector, your app will connect directly to SigNoz on ports 4317/4318.

### Step 3: Configure Environment Variables

Create or update your `.env` file with OTLP settings:

```bash
# Enable OTLP telemetry
ENABLE_OTLP=true

# OTLP endpoint (Vector/Jaeger/SigNoz gRPC port)
OTLP_ENDPOINT=http://localhost:4317

# Log level
RUST_LOG=info
```

Available log levels: `trace`, `debug`, `info`, `warn`, `error`

### Step 4: Start Your Application

```bash
cd video-server-rs_v1
cargo run --release
```

Look for the startup message:
```
📊 OTLP telemetry enabled
📡 Connecting to OTLP endpoint: http://localhost:4317
✅ Tracer installed successfully
✅ Logger provider installed successfully
```

If OTLP is disabled, you'll see:
```
📊 OTLP telemetry disabled (set ENABLE_OTLP=true to enable)
```

### Step 5: Generate Traffic

Interact with your application:

```bash
# Health check
curl http://localhost:3000/health

# Access the homepage
curl http://localhost:3000/

# List videos (requires authentication)
curl http://localhost:3000/videos

# Upload an image
curl -X POST http://localhost:3000/api/images/upload \
  -F "file=@/path/to/image.jpg" \
  -F "title=Test Image" \
  -F "is_public=true"
```

Or use your browser to navigate through the application.

### Step 6: View Traces in SigNoz

1. Open SigNoz UI: http://localhost:3301
2. Navigate to **Services** tab
3. Click on `axum-server` service
4. Explore traces, metrics, and service maps!

**SigNoz Features:**
- 📊 Service dependency graphs
- 📈 RED metrics (Rate, Errors, Duration)
- 🔍 Advanced trace filtering
- 📉 Custom dashboards
- 🚨 Alerting capabilities

---

## Alternative Setup: Jaeger (Simpler, Traces Only)

If you prefer a simpler setup with just traces (no metrics/logs), use Jaeger:

### Start Jaeger

```bash
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest
```

### View Traces

1. Open Jaeger UI: http://localhost:16686
2. Select **Service**: `axum-server`
3. Click **Find Traces**
4. Explore your traces!

---

## Alternative Setup: Grafana + Tempo

For Grafana users who want to combine traces with existing dashboards:

### Start Tempo + Grafana

```bash
# Run Tempo
docker run -d --name tempo \
  -p 3200:3200 \
  -p 4317:4317 \
  -p 4318:4318 \
  grafana/tempo:latest

# Run Grafana
docker run -d --name grafana \
  -p 3001:3000 \
  grafana/grafana

# Configure Tempo as datasource in Grafana at http://localhost:3001
# Datasource URL: http://tempo:3200
```

## Understanding Traces

### Trace Components

- **Trace**: Complete request journey through your system
- **Span**: Single operation within a trace (e.g., handler execution)
- **Tags**: Metadata attached to spans (function name, parameters)

### Example Trace Flow

```
HTTP GET /videos
└── videos_list_handler (span)
    ├── check authentication (nested operation)
    ├── get_videos (database query)
    └── render template
```

### Key Metrics to Monitor

1. **Duration**: How long each handler takes
2. **Error Rate**: Failed requests and their causes
3. **Throughput**: Requests per second
4. **Latency Percentiles**: p50, p95, p99 response times

## Advanced Configuration

### Use gRPC Instead of HTTP

Edit `src/main.rs::init_tracer()`:

```rust
let endpoint = "http://localhost:4317"; // gRPC endpoint

let otlp_exporter = opentelemetry_otlp::new_exporter()
    .tonic() // Use gRPC
    .with_endpoint(endpoint)
    .with_timeout(std::time::Duration::from_secs(5));
```

### Configure Trace Sampling

Reduce trace volume in production by sampling:

```rust
use opentelemetry_sdk::trace::Sampler;

let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(otlp_exporter)
    .with_trace_config(
        sdktrace::config()
            .with_sampler(Sampler::TraceIdRatioBased(0.1)) // 10% sampling
            .with_resource(...)
    )
    .install_batch(runtime::Tokio)?;
```

### Export to Multiple Backends

Use Vector or OpenTelemetry Collector to fan out traces:

```bash
# Run OpenTelemetry Collector
docker run -d --name otel-collector \
  -p 4317:4317 \
  -p 4318:4318 \
  -v $(pwd)/otel-config.yaml:/etc/otel-config.yaml \
  otel/opentelemetry-collector:latest \
  --config=/etc/otel-config.yaml
```

Example `otel-config.yaml`:

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317
      http:
        endpoint: 0.0.0.0:4318

exporters:
  jaeger:
    endpoint: jaeger:14250
  prometheus:
    endpoint: 0.0.0.0:8889
  logging:
    loglevel: debug

service:
  pipelines:
    traces:
      receivers: [otlp]
      exporters: [jaeger, logging]
```

## Production Best Practices

### 1. Use Environment Variables

Configure via `.env` file or environment:

```bash
# Enable OTLP
ENABLE_OTLP=true

# OTLP endpoint
OTLP_ENDPOINT=http://collector:4317

# Log level (use warn or error in production)
RUST_LOG=warn
```

### 2. Enable Trace Sampling

Don't trace every request in high-traffic environments. Use ratio-based sampling (e.g., 1-10%).

### 3. Set Resource Attributes

Add deployment metadata:

```rust
opentelemetry_sdk::Resource::new(vec![
    opentelemetry::KeyValue::new("service.name", "video-server"),
    opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
    opentelemetry::KeyValue::new("deployment.environment", "production"),
    opentelemetry::KeyValue::new("service.instance.id", hostname),
])
```

### 4. Monitor Exporter Health

Check logs for export failures:

```bash
grep "OTLP" application.log
```

### 5. Set Timeouts

Configure timeouts for OTLP exports to prevent blocking:

```rust
.with_timeout(std::time::Duration::from_secs(5))
```

## Troubleshooting

### Issue: No traces appearing in Jaeger

**Solutions:**
1. Verify Jaeger is running: `docker ps | grep jaeger`
2. Check application logs for connection errors
3. Test OTLP endpoint: `curl http://localhost:4318/v1/traces`
4. Ensure `RUST_LOG` is set to at least `info`

### Issue: Application crashes on startup

**Solutions:**
1. Check if OTLP endpoint is reachable
2. The app gracefully falls back to local logging if OTLP fails
3. Review `init_tracer()` error handling

### Issue: High memory usage

**Solutions:**
1. Reduce `RUST_LOG` level to `warn` or `error`
2. Enable trace sampling
3. Increase batch export intervals
4. Skip more parameters in `#[instrument]` attributes

### Issue: Slow performance

**Solutions:**
1. Use async OTLP exporter (already configured)
2. Increase batch size and timeout
3. Use gRPC instead of HTTP for better performance
4. Deploy Vector as a local buffer/proxy
5. Enable trace sampling in production

## Integration with Other Tools

### Vector Data Pipeline

Vector is recommended for production as it provides:
- **Buffering**: Prevents data loss during backend outages
- **Transformation**: Modify traces before export
- **Routing**: Send to multiple backends
- **Sampling**: Reduce trace volume

Example `vector.toml` with sampling:

```toml
[sources.otlp]
type = "http_server"
address = "0.0.0.0:4318"
path = "/v1/traces"

[transforms.sample]
type = "sample"
inputs = ["otlp"]
rate = 10  # Keep 10% of traces

[sinks.signoz]
type = "http"
inputs = ["sample"]
uri = "http://signoz:4318/v1/traces"
encoding.codec = "json"

[sinks.backup_storage]
type = "aws_s3"
inputs = ["sample"]
bucket = "traces-backup"
compression = "gzip"
```

### Prometheus Metrics

Add metrics alongside traces:

```toml
# Cargo.toml
[dependencies]
opentelemetry-prometheus = "0.14"
prometheus = "0.13"
```

### ELK Stack

Use Filebeat or Logstash to ship logs to Elasticsearch:

```bash
# Structured JSON logging
export RUST_LOG_FORMAT=json
```

## Monitoring Dashboards

### SigNoz Built-in Dashboards

SigNoz provides out-of-the-box dashboards:
- **APM**: Service overview with RED metrics
- **Traces**: Detailed trace search and analysis
- **Service Map**: Visualize dependencies
- **Exceptions**: Error tracking and grouping

### Key Queries (Works in SigNoz, Jaeger, Grafana)

1. **Slow Requests**: Duration > 1s
2. **Error Rate**: Status = Error
3. **Top Endpoints**: Group by operation name
4. **Database Latency**: Filter spans containing "query"

### Custom SigNoz Dashboard Panels

Create panels for:
- Request rate (traces/second)
- Average response time by endpoint
- Error rate percentage
- P50/P95/P99 latency
- Top 10 slowest endpoints
- Authentication success/failure rate
- Video streaming performance
- Image upload performance

## Further Reading

- [SigNoz Documentation](https://signoz.io/docs/)
- [Vector Documentation](https://vector.dev/docs/)
- [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
- [Jaeger Documentation](https://www.jaegertracing.io/docs/)
- [Tracing Best Practices](https://docs.rs/tracing/latest/tracing/)
- [Distributed Tracing Guide](https://opentelemetry.io/docs/concepts/signals/traces/)

## Cleanup

### SigNoz

```bash
cd signoz/deploy/
docker compose -f docker/clickhouse-setup/docker-compose.yaml down

# Remove volumes (deletes all data)
docker compose -f docker/clickhouse-setup/docker-compose.yaml down -v
```

### Vector

```bash
docker stop vector
docker rm vector
```

### Jaeger

```bash
docker stop jaeger
docker rm jaeger
```

### Keep Data for Next Session

```bash
# Just stop containers, don't remove
docker stop vector signoz jaeger
# Restart later with: docker start <container_name>
```

---

**Need Help?** Check `INSTRUMENTATION.md` for detailed handler documentation.