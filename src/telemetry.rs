use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracer() -> Result<(), Box<dyn std::error::Error>> {
    println!("\u{1f527} Initializing OpenTelemetry...");

    // Get OTLP endpoint from environment
    let otlp_endpoint =
        std::env::var("OTLP_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_string());

    println!("\u{1f4e1} Connecting to OTLP endpoint: {}", otlp_endpoint);

    // Create shared resource - OpenTelemetry 0.31 API
    let resource = opentelemetry_sdk::Resource::builder()
        .with_service_name(
            std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "video-server".to_string()),
        )
        .build();

    // Build trace exporter using OpenTelemetry 0.31 API
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

    // Get tracer from provider
    let tracer = tracer_provider.tracer("video-server");

    println!("\u{2705} Tracer installed successfully");

    // Build log exporter using OpenTelemetry 0.31 API
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

    println!("\u{2705} Logger provider installed successfully");

    // Create the tracing bridge that sends log events to OTLP
    let otel_log_layer = OpenTelemetryTracingBridge::new(&logger_provider);

    // Create OpenTelemetry tracing layer for spans/traces
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Initialize tracing subscriber with all layers
    match tracing_subscriber::registry()
        .with(telemetry_layer) // For traces/spans
        .with(otel_log_layer) // For logs via OTLP
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer()) // Console output
        .try_init()
    {
        Ok(_) => println!("\u{2705} Tracing subscriber initialized"),
        Err(e) => {
            println!("\u{274c} Failed to initialize subscriber: {}", e);
            return Err(Box::new(e));
        }
    }

    println!("\u{2705} OpenTelemetry initialized successfully (traces + logs)");
    Ok(())
}
