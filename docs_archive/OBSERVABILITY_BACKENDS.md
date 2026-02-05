# Observability Backends Comparison

This guide compares different observability backends compatible with video-server-rs OpenTelemetry instrumentation.

## Quick Comparison

| Feature | SigNoz + Vector | Jaeger | Grafana Tempo | Zipkin |
|---------|----------------|--------|---------------|--------|
| **Traces** | ✅ Excellent | ✅ Excellent | ✅ Excellent | ✅ Good |
| **Metrics** | ✅ Built-in | ❌ No | ⚠️ Via Prometheus | ❌ No |
| **Logs** | ✅ Built-in | ❌ No | ⚠️ Via Loki | ❌ No |
| **Service Map** | ✅ Yes | ✅ Yes | ✅ Yes | ⚠️ Limited |
| **Alerting** | ✅ Built-in | ❌ No | ✅ Via Grafana | ❌ No |
| **Setup Complexity** | ⚠️ Medium | ✅ Easy | ⚠️ Medium | ✅ Easy |
| **Resource Usage** | 🔶 Medium | 🟢 Low | 🟢 Low | 🟢 Low |
| **Cost** | 🟢 Free (OSS) | 🟢 Free | 🟢 Free | 🟢 Free |
| **Best For** | Production | Development | Existing Grafana | Legacy systems |

## Detailed Comparison

### 1. SigNoz + Vector (⭐ Recommended)

**Pros:**
- ✅ **Complete observability** - traces, metrics, and logs in one platform
- ✅ **Production-ready** - Built for scale with ClickHouse backend
- ✅ **Built-in alerting** - No additional tools needed
- ✅ **Vector integration** - Reliable data pipeline with buffering
- ✅ **Modern UI** - Clean, intuitive interface
- ✅ **Service maps** - Automatic dependency visualization
- ✅ **Query builder** - Easy trace filtering
- ✅ **Dashboards** - Custom metrics dashboards

**Cons:**
- ⚠️ **More components** - SigNoz + Vector + ClickHouse
- ⚠️ **Higher resource usage** - ~2GB RAM minimum
- ⚠️ **Setup complexity** - Requires Docker Compose

**Resource Requirements:**
- CPU: 2+ cores recommended
- RAM: 2-4 GB
- Disk: 10+ GB for trace storage
- Ports: 3301 (UI), 4317/4318 (OTLP), 9000 (ClickHouse)

**When to Use:**
- ✅ Production deployments
- ✅ Need metrics + traces + logs
- ✅ Want built-in alerting
- ✅ Multiple microservices
- ✅ Team collaboration

**Setup Time:** ~15 minutes

**Docker Command:**
```bash
# Clone and start SigNoz
git clone https://github.com/SigNoz/signoz.git
cd signoz/deploy/
docker compose -f docker/clickhouse-setup/docker-compose.yaml up -d

# Add Vector for data pipeline
docker run -d --name vector \
  -p 4317:4317 -p 4318:4318 \
  -v $(pwd)/vector.toml:/etc/vector/vector.toml \
  timberio/vector:latest-alpine
```

---

### 2. Jaeger (✨ Simplest)

**Pros:**
- ✅ **Easy setup** - Single Docker command
- ✅ **Low resource usage** - ~500MB RAM
- ✅ **Fast** - Optimized for trace queries
- ✅ **Battle-tested** - CNCF graduated project
- ✅ **Good UI** - Clean trace visualization
- ✅ **Service dependencies** - Built-in service graph

**Cons:**
- ❌ **Traces only** - No metrics or logs
- ❌ **No alerting** - Need external tools
- ❌ **No dashboards** - Limited to trace search
- ❌ **Storage limits** - In-memory by default

**Resource Requirements:**
- CPU: 1 core
- RAM: 512 MB - 1 GB
- Disk: 1+ GB (if using Cassandra/ES backend)
- Ports: 16686 (UI), 4317/4318 (OTLP)

**When to Use:**
- ✅ Development/testing
- ✅ Learning distributed tracing
- ✅ Quick trace debugging
- ✅ Limited resources
- ✅ Traces-only requirement

**Setup Time:** ~2 minutes

**Docker Command:**
```bash
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest
```

---

### 3. Grafana Tempo + Grafana

**Pros:**
- ✅ **Grafana integration** - Perfect if already using Grafana
- ✅ **Cost-effective storage** - Object storage backends (S3, GCS)
- ✅ **Scalable** - Designed for massive scale
- ✅ **TraceQL** - Powerful query language
- ✅ **Correlate with metrics** - Link traces to Prometheus metrics
- ✅ **Low resource usage** - Efficient design

**Cons:**
- ⚠️ **Requires Grafana** - Extra component to manage
- ⚠️ **Configuration complexity** - More setup required
- ❌ **No standalone UI** - Must use Grafana
- ⚠️ **Storage setup** - Need S3/GCS or local storage

**Resource Requirements:**
- CPU: 1-2 cores
- RAM: 1-2 GB
- Disk: Depends on backend (S3 recommended)
- Ports: 3200 (Tempo), 3000 (Grafana), 4317/4318 (OTLP)

**When to Use:**
- ✅ Already using Grafana
- ✅ Need metrics + traces correlation
- ✅ Object storage available (S3, GCS)
- ✅ Large-scale deployments
- ✅ Cost-sensitive storage

**Setup Time:** ~10 minutes

**Docker Command:**
```bash
# Run Tempo
docker run -d --name tempo \
  -p 3200:3200 -p 4317:4317 -p 4318:4318 \
  grafana/tempo:latest

# Run Grafana
docker run -d --name grafana \
  -p 3000:3000 \
  grafana/grafana:latest
```

---

### 4. Zipkin

**Pros:**
- ✅ **Lightweight** - Minimal resource usage
- ✅ **Easy setup** - Single container
- ✅ **Mature** - Long-established project
- ✅ **Simple UI** - Easy to understand

**Cons:**
- ❌ **Limited features** - Basic trace visualization
- ❌ **Aging UI** - Less modern than alternatives
- ❌ **No metrics/logs** - Traces only
- ❌ **Limited filtering** - Basic search capabilities

**Resource Requirements:**
- CPU: 1 core
- RAM: 512 MB
- Disk: 1+ GB
- Ports: 9411 (UI + API), 4317/4318 (OTLP)

**When to Use:**
- ✅ Legacy systems migration
- ✅ Minimal resource environments
- ✅ Simple trace viewing needs
- ⚠️ Not recommended for new projects

**Setup Time:** ~3 minutes

**Docker Command:**
```bash
docker run -d --name zipkin \
  -p 9411:9411 \
  openzipkin/zipkin:latest
```

---

## Feature Matrix

### Trace Visualization

| Feature | SigNoz | Jaeger | Tempo | Zipkin |
|---------|--------|--------|-------|--------|
| Span tree view | ✅ | ✅ | ✅ | ✅ |
| Timeline view | ✅ | ✅ | ✅ | ✅ |
| Flamegraph | ✅ | ✅ | ✅ | ❌ |
| Trace comparison | ✅ | ❌ | ⚠️ | ❌ |
| Search filters | ✅ Advanced | ✅ Good | ✅ TraceQL | ⚠️ Basic |

### Storage Backends

| Backend | SigNoz | Jaeger | Tempo | Zipkin |
|---------|--------|--------|-------|--------|
| In-memory | ❌ | ✅ | ❌ | ✅ |
| ClickHouse | ✅ | ❌ | ❌ | ❌ |
| Cassandra | ❌ | ✅ | ❌ | ✅ |
| Elasticsearch | ❌ | ✅ | ❌ | ✅ |
| S3/GCS | ❌ | ❌ | ✅ | ❌ |
| MySQL | ❌ | ❌ | ❌ | ✅ |

### Data Pipeline

| Feature | SigNoz + Vector | Jaeger | Tempo | Zipkin |
|---------|----------------|--------|-------|--------|
| Buffering | ✅ Vector | ⚠️ Limited | ⚠️ Limited | ⚠️ Limited |
| Sampling | ✅ Vector | ✅ Built-in | ✅ Built-in | ✅ Built-in |
| Filtering | ✅ Vector | ⚠️ Limited | ⚠️ Limited | ❌ |
| Transform | ✅ Vector | ❌ | ❌ | ❌ |
| Multi-sink | ✅ Vector | ❌ | ❌ | ❌ |

---

## Use Case Recommendations

### Development & Testing
**Recommended: Jaeger**
- Fast setup
- Low resources
- Good enough for debugging

```bash
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 16686:16686 -p 4318:4318 \
  jaegertracing/all-in-one:latest
```

### Production (Small/Medium)
**Recommended: SigNoz + Vector**
- Complete observability
- Built-in alerting
- Modern UI
- Good for teams

```bash
git clone https://github.com/SigNoz/signoz.git
cd signoz/deploy/
docker compose -f docker/clickhouse-setup/docker-compose.yaml up -d
```

### Production (Large Scale)
**Recommended: Grafana Tempo + Grafana**
- Object storage (S3/GCS)
- Cost-effective at scale
- Integrates with existing Grafana

### Existing Grafana Users
**Recommended: Grafana Tempo**
- Native integration
- Unified dashboard
- Correlate traces with metrics

### Migration from Legacy
**Consider: Zipkin**
- If already using Zipkin instrumentation
- Otherwise, migrate to SigNoz or Jaeger

---

## Cost Analysis (Storage)

### 1 million spans/day for 30 days:

| Backend | Estimated Storage | Cost (S3) | Notes |
|---------|------------------|-----------|-------|
| SigNoz (ClickHouse) | ~50 GB | $1.15/month | Compressed, columnar |
| Jaeger (Cassandra) | ~80 GB | $1.84/month | Higher overhead |
| Tempo (S3) | ~30 GB | $0.69/month | Highly compressed |
| Zipkin (MySQL) | ~100 GB | $2.30/month | Less efficient |

*Costs based on AWS S3 Standard pricing. Actual costs vary by compression and retention.*

---

## Migration Guide

### From No Observability → SigNoz
1. Deploy SigNoz (15 min)
2. Deploy Vector (5 min)
3. Application already instrumented! ✅
4. View traces immediately

### From Jaeger → SigNoz
1. Deploy SigNoz
2. Point app to SigNoz endpoint (change port if needed)
3. Keep Jaeger running for historical traces
4. Migrate when comfortable

### From Zipkin → SigNoz
1. Deploy SigNoz
2. Update OTLP endpoint in app
3. Both can run simultaneously
4. Decommission Zipkin when ready

---

## Quick Decision Tree

```
Do you need metrics + logs + traces?
├─ YES → SigNoz
└─ NO (traces only)
    ├─ Already using Grafana? 
    │  ├─ YES → Grafana Tempo
    │  └─ NO
    │      ├─ Development/Testing? 
    │      │  ├─ YES → Jaeger
    │      │  └─ NO → SigNoz (future-proof)
    │      └─ Large scale (millions/day)?
    │          ├─ YES → Grafana Tempo (S3 backend)
    │          └─ NO → SigNoz or Jaeger
```

---

## Summary

### Best Choice for video-server-rs

**🥇 SigNoz + Vector** (Primary Recommendation)
- Complete observability platform
- Production-ready
- Best ROI for effort

**🥈 Jaeger** (Development Alternative)
- Quick setup for testing
- Upgrade to SigNoz for production

**🥉 Grafana Tempo** (If using Grafana already)
- Perfect integration
- Cost-effective storage

### Why Vector?

Vector acts as a reliable data pipeline:
- **Buffering** - Prevents data loss during outages
- **Transformation** - Enrich traces with metadata
- **Routing** - Send to multiple backends
- **Sampling** - Reduce volume in production

Even with other backends, Vector is recommended for production.

---

## Additional Resources

- **SigNoz**: https://signoz.io/docs/
- **Vector**: https://vector.dev/docs/
- **Jaeger**: https://www.jaegertracing.io/docs/
- **Grafana Tempo**: https://grafana.com/docs/tempo/
- **OpenTelemetry**: https://opentelemetry.io/docs/

---

**Questions?** See [OBSERVABILITY_QUICKSTART.md](OBSERVABILITY_QUICKSTART.md) for setup instructions.