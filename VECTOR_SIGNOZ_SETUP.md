# Vector + SigNoz Setup Guide for video-server-rs

This guide provides detailed setup instructions for integrating your video-server-rs application with Vector and SigNoz for comprehensive observability.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        video-server-rs                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │    Main      │  │    Video     │  │    Image     │              │
│  │   Handlers   │  │   Manager    │  │   Manager    │              │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘              │
│         │                  │                  │                       │
│         └──────────────────┴──────────────────┘                      │
│                            │                                          │
│                   OpenTelemetry SDK                                   │
│                            │                                          │
└────────────────────────────┼──────────────────────────────────────────┘
                             │ OTLP (HTTP/gRPC)
                             ▼
                    ┌─────────────────┐
                    │     Vector      │
                    │  (Data Router)  │
                    │                 │
                    │ • Buffering     │
                    │ • Sampling      │
                    │ • Transform     │
                    │ • Routing       │
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        ▼                    ▼                    ▼
┌───────────────┐    ┌──────────────┐    ┌─────────────┐
│    SigNoz     │    │   File Log   │    │   Backup    │
│               │    │   (Debug)    │    │  (Archive)  │
│ • Traces      │    └──────────────┘    └─────────────┘
│ • Metrics     │
│ • Logs        │
│ • Dashboards  │
└───────────────┘
```

## Prerequisites

- Docker & Docker Compose
- 8GB+ RAM recommended
- Ports available: 3301, 4317, 4318, 8686

## Step 1: Install SigNoz

### Option A: Docker Compose (Recommended)

```bash
# Clone SigNoz repository
git clone -b main https://github.com/SigNoz/signoz.git
cd signoz/deploy/

# Start SigNoz
docker compose -f docker/clickhouse-setup/docker-compose.yaml up -d

# Check status
docker compose -f docker/clickhouse-setup/docker-compose.yaml ps

# View logs
docker compose -f docker/clickhouse-setup/docker-compose.yaml logs -f
```

### Option B: Standalone Docker

```bash
# Create network
docker network create signoz-network

# Run ClickHouse
docker run -d \
  --name signoz-clickhouse \
  --network signoz-network \
  -p 9000:9000 \
  clickhouse/clickhouse-server:latest

# Run SigNoz Query Service
docker run -d \
  --name signoz-query-service \
  --network signoz-network \
  -p 8080:8080 \
  signoz/query-service:latest

# Run SigNoz Frontend
docker run -d \
  --name signoz-frontend \
  --network signoz-network \
  -p 3301:3301 \
  signoz/frontend:latest

# Run SigNoz OtelCollector
docker run -d \
  --name signoz-otel-collector \
  --network signoz-network \
  -p 4317:4317 \
  -p 4318:4318 \
  signoz/otelcollector:latest
```

### Verify SigNoz Installation

```bash
# Check if UI is accessible
curl http://localhost:3301

# Check OTLP endpoint
curl http://localhost:4318/v1/traces -X POST -d '{}'

# Expected: 405 Method Not Allowed or similar (means endpoint is up)
```

## Step 2: Install and Configure Vector

### Create Vector Configuration

Create `vector-config.toml`:

```toml
# ============================================================================
# VECTOR CONFIGURATION FOR video-server-rs + SigNoz
# ============================================================================

# Data directory for Vector state
data_dir = "/var/lib/vector"

# ----------------------------------------------------------------------------
# SOURCES - Receive data from video-server-rs
# ----------------------------------------------------------------------------

# OTLP HTTP Source (Primary)
[sources.otlp_http]
type = "http_server"
address = "0.0.0.0:4318"
path = "/v1/traces"
decoding.codec = "bytes"
headers = ["content-type"]

[sources.otlp_http.encoding]
codec = "bytes"

# OTLP gRPC Source (Alternative)
[sources.otlp_grpc]
type = "http_server"
address = "0.0.0.0:4317"
decoding.codec = "bytes"

# ----------------------------------------------------------------------------
# TRANSFORMS - Process and enrich trace data
# ----------------------------------------------------------------------------

# Add deployment metadata
[transforms.enrich]
type = "remap"
inputs = ["otlp_http", "otlp_grpc"]
source = '''
  .deployment_env = "production"
  .service_version = "v1"
  .datacenter = "us-east-1"
'''

# Sample traces (optional - for high-volume production)
[transforms.sample]
type = "sample"
inputs = ["enrich"]
rate = 100  # Keep 100% for development, reduce to 10-50 for production

# Filter out health check noise (optional)
[transforms.filter_noise]
type = "filter"
inputs = ["sample"]
condition = '''
  !contains(string!(.message) ?? "", "health_check")
'''

# ----------------------------------------------------------------------------
# SINKS - Send data to multiple destinations
# ----------------------------------------------------------------------------

# Primary: Send to SigNoz
[sinks.signoz]
type = "http"
inputs = ["filter_noise"]
uri = "http://localhost:4318/v1/traces"
encoding.codec = "json"
compression = "gzip"
batch.max_bytes = 1048576
batch.timeout_secs = 5

[sinks.signoz.request]
headers.Content-Type = "application/json"

# Secondary: Console output for debugging
[sinks.console_debug]
type = "console"
inputs = ["filter_noise"]
encoding.codec = "json"
target = "stdout"

# Tertiary: File backup for audit/recovery
[sinks.file_backup]
type = "file"
inputs = ["filter_noise"]
path = "/var/log/vector/traces-%Y-%m-%d.log"
encoding.codec = "json"
compression = "gzip"

# Quaternary: Metrics about trace processing
[sinks.internal_metrics]
type = "prometheus_exporter"
inputs = ["filter_noise"]
address = "0.0.0.0:9090"
default_namespace = "vector"

# ----------------------------------------------------------------------------
# BUFFER CONFIGURATION
# ----------------------------------------------------------------------------

# Configure disk-based buffering for reliability
[sinks.signoz.buffer]
type = "disk"
max_size = 268435488  # 256 MB
when_full = "block"
```

### Start Vector with Docker

```bash
# Create Vector data directory
mkdir -p ./vector-data
mkdir -p ./vector-logs

# Run Vector
docker run -d \
  --name vector \
  --network host \
  -v $(pwd)/vector-config.toml:/etc/vector/vector.toml:ro \
  -v $(pwd)/vector-data:/var/lib/vector \
  -v $(pwd)/vector-logs:/var/log/vector \
  timberio/vector:0.34.0-alpine \
  --config /etc/vector/vector.toml

# Check Vector logs
docker logs -f vector

# Expected output:
# INFO vector::app: Log level is enabled. level="info"
# INFO vector::topology: Running healthchecks.
# INFO vector: Vector has started.
```

### Verify Vector is Working

```bash
# Check Vector metrics endpoint
curl http://localhost:9090/metrics

# Send test trace
curl -X POST http://localhost:4318/v1/traces \
  -H "Content-Type: application/json" \
  -d '{
    "resourceSpans": [{
      "resource": {
        "attributes": [{
          "key": "service.name",
          "value": {"stringValue": "test-service"}
        }]
      },
      "scopeSpans": [{
        "spans": [{
          "traceId": "0102030405060708090a0b0c0d0e0f10",
          "spanId": "0102030405060708",
          "name": "test-span",
          "kind": 1,
          "startTimeUnixNano": "1000000000",
          "endTimeUnixNano": "2000000000"
        }]
      }]
    }]
  }'

# Check Vector processed it
docker logs vector | grep "test-span"
```

## Step 3: Configure video-server-rs

The application is already configured to send traces to `http://localhost:4318`. No changes needed!

Verify in `src/main.rs`:

```rust
fn init_tracer() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = "http://localhost:4318"; // ✓ Correct for Vector/SigNoz
    
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_endpoint(endpoint)
        .with_timeout(std::time::Duration::from_secs(5));
    // ...
}
```

## Step 4: Start Everything

```bash
# 1. Ensure SigNoz is running
curl http://localhost:3301 && echo "SigNoz OK"

# 2. Ensure Vector is running
curl http://localhost:9090/metrics && echo "Vector OK"

# 3. Start video-server-rs
cd video-server-rs_v1
cargo run --release

# Expected output:
# ✓ Connected to OTLP endpoint: http://localhost:4318
# 🚀 Initializing Modular Video Server...
```

## Step 5: Generate Test Traffic

```bash
# Health check
curl http://localhost:3000/health

# Homepage
curl http://localhost:3000/

# Login page
curl http://localhost:3000/login

# Video list (will show unauthorized)
curl http://localhost:3000/videos

# MediaMTX status
curl http://localhost:3000/api/mediamtx/status
```

## Step 6: View Traces in SigNoz

1. Open SigNoz UI: **http://localhost:3301**
2. Click **"Services"** in left sidebar
3. You should see **"axum-server"** service
4. Click on service name to see:
   - **Overview**: P99, P95, P50 latencies, error rate, throughput
   - **Traces**: Individual request traces
   - **Service Map**: Dependencies between handlers
   - **Database Calls**: If instrumented

### Understanding the SigNoz UI

#### Services Dashboard
- **RED Metrics**: Rate, Errors, Duration for each endpoint
- **Apdex Score**: Application performance index
- **Key Operations**: Slowest and most-called handlers

#### Traces View
- **Trace ID**: Unique identifier for request
- **Span Tree**: Hierarchical view of operations
- **Duration**: Time spent in each handler
- **Tags**: Metadata (user_id, endpoint, etc.)

#### Filtering Traces
```
# Find slow requests
duration > 1s

# Find errors
status = error

# Find specific handler
serviceName = axum-server AND operation = video_player_handler

# Find user activity
tag.user_id = "user123"
```

## Production Configuration

### Vector Production Config

Create `vector-production.toml`:

```toml
data_dir = "/var/lib/vector"

# OTLP Source
[sources.otlp_http]
type = "http_server"
address = "0.0.0.0:4318"
path = "/v1/traces"
decoding.codec = "bytes"

# Add production metadata
[transforms.enrich]
type = "remap"
inputs = ["otlp_http"]
source = '''
  .deployment_env = get_env_var!("DEPLOYMENT_ENV")
  .datacenter = get_env_var!("DATACENTER")
  .hostname = get_hostname!()
  .k8s_pod = get_env_var("K8S_POD_NAME") ?? "unknown"
'''

# Aggressive sampling for production
[transforms.sample]
type = "sample"
inputs = ["enrich"]
rate = 10  # Keep only 10% of traces

# Filter health checks and static assets
[transforms.filter]
type = "filter"
inputs = ["sample"]
condition = '''
  op = string!(.name) ?? ""
  !contains(op, "health_check") &&
  !contains(op, "static") &&
  !starts_with(op, "/storage/")
'''

# Primary sink: SigNoz
[sinks.signoz]
type = "http"
inputs = ["filter"]
uri = "${SIGNOZ_ENDPOINT:-http://signoz:4318}/v1/traces"
encoding.codec = "json"
compression = "gzip"
batch.max_bytes = 10485760  # 10 MB
batch.timeout_secs = 10

[sinks.signoz.buffer]
type = "disk"
max_size = 1073741824  # 1 GB
when_full = "drop_newest"

[sinks.signoz.request]
retry_max_duration_secs = 30
retry_initial_backoff_secs = 1

# Backup: Long-term storage
[sinks.s3_backup]
type = "aws_s3"
inputs = ["filter"]
bucket = "${TRACES_BUCKET}"
key_prefix = "traces/%Y/%m/%d/"
encoding.codec = "json"
compression = "gzip"
batch.max_bytes = 10485760
region = "${AWS_REGION}"

# Alerting: Critical errors only
[sinks.pagerduty]
type = "http"
inputs = ["filter"]
uri = "https://events.pagerduty.com/v2/enqueue"
encoding.codec = "json"

[sinks.pagerduty.transform]
type = "filter"
condition = 'contains(string!(.status) ?? "", "error")'
```

### Environment Variables

Create `.env.production`:

```bash
# Application
RUST_LOG=warn,video_server_rs=info
DEPLOYMENT_ENV=production
DATACENTER=us-east-1

# SigNoz
SIGNOZ_ENDPOINT=http://signoz:4318

# Vector
VECTOR_CONFIG=/etc/vector/vector-production.toml

# AWS (for backups)
AWS_REGION=us-east-1
TRACES_BUCKET=my-app-traces

# Sampling
TRACE_SAMPLE_RATE=0.1  # 10%
```

### Kubernetes Deployment

Create `k8s-deployment.yaml`:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: vector-config
data:
  vector.toml: |
    # (Insert vector-production.toml content here)
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: video-server
spec:
  replicas: 3
  selector:
    matchLabels:
      app: video-server
  template:
    metadata:
      labels:
        app: video-server
    spec:
      containers:
      - name: app
        image: video-server-rs:latest
        ports:
        - containerPort: 3000
        env:
        - name: RUST_LOG
          value: "info"
      - name: vector
        image: timberio/vector:0.34.0-alpine
        ports:
        - containerPort: 4318
        - containerPort: 9090
        volumeMounts:
        - name: vector-config
          mountPath: /etc/vector
        - name: vector-data
          mountPath: /var/lib/vector
      volumes:
      - name: vector-config
        configMap:
          name: vector-config
      - name: vector-data
        emptyDir: {}
```

## Monitoring Vector Itself

### Vector Metrics

```bash
# View Vector's own metrics
curl http://localhost:9090/metrics | grep vector_

# Key metrics:
# - vector_processed_events_total: Events processed
# - vector_buffer_events: Events in buffer
# - vector_component_errors_total: Processing errors
# - vector_component_sent_bytes_total: Data sent to sinks
```

### Vector Health Check

```bash
# Add to vector.toml
[api]
enabled = true
address = "0.0.0.0:8686"

# Check health
curl http://localhost:8686/health
```

## Troubleshooting

### Issue: Traces not appearing in SigNoz

**Check Vector logs:**
```bash
docker logs vector | grep -i error
```

**Verify Vector is receiving data:**
```bash
docker logs vector | grep "processed"
```

**Check SigNoz endpoint:**
```bash
curl http://localhost:4318/v1/traces -X POST -d '{}'
```

**Verify network connectivity:**
```bash
docker exec vector ping -c 3 localhost
```

### Issue: High memory usage

**Solution 1: Reduce buffer size**
```toml
[sinks.signoz.buffer]
max_size = 104857600  # 100 MB instead of 256 MB
```

**Solution 2: Increase sample rate**
```toml
[transforms.sample]
rate = 5  # Keep only 5% instead of 10%
```

**Solution 3: Add memory limits**
```bash
docker run --memory=512m --memory-swap=512m ...
```

### Issue: Vector crashes

**Check disk space:**
```bash
df -h ./vector-data
```

**Check logs for OOM:**
```bash
docker logs vector | grep -i "out of memory"
```

**Reduce batch size:**
```toml
[sinks.signoz]
batch.max_bytes = 1048576  # 1 MB
```

## Performance Tuning

### Optimize for Throughput

```toml
# Larger batches, less frequent
[sinks.signoz]
batch.max_bytes = 10485760  # 10 MB
batch.timeout_secs = 30

# In-memory buffer for speed
[sinks.signoz.buffer]
type = "memory"
max_events = 10000
```

### Optimize for Low Latency

```toml
# Smaller batches, more frequent
[sinks.signoz]
batch.max_bytes = 102400  # 100 KB
batch.timeout_secs = 1

# Disk buffer for reliability
[sinks.signoz.buffer]
type = "disk"
max_size = 268435456  # 256 MB
```

## Resources

- **SigNoz Docs**: https://signoz.io/docs/
- **Vector Docs**: https://vector.dev/docs/
- **OTLP Spec**: https://opentelemetry.io/docs/specs/otlp/
- **Example Dashboards**: https://github.com/SigNoz/dashboards

## Next Steps

1. ✅ Set up custom SigNoz dashboards for your app
2. ✅ Configure alerting rules for critical errors
3. ✅ Enable metrics collection (RED metrics)
4. ✅ Add distributed tracing across microservices
5. ✅ Set up log aggregation in Vector
6. ✅ Configure long-term trace storage (S3/GCS)