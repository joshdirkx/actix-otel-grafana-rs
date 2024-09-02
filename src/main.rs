use actix_web::{web, App, HttpServer, Responder};
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::runtime::Tokio;
use tracing::{info, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use tracing_opentelemetry::OpenTelemetryLayer;

async fn index() -> impl Responder {
    let span = tracing::info_span!("processing request");
    let _guard = span.enter();

    info!("Processing the request");

    let meter = global::meter("actix-otel-rs");
    let counter = meter.u64_counter("requests_processed").init();

    counter.add(1, &[]);

    "Hello, OpenTelemetry with Actix-web!".to_string()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up OpenTelemetry Tracing
    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://otel-collector:4317"),
        )
        .install_batch(Tokio)
        .expect("Failed to install OpenTelemetry tracer");

    // Get a tracer from the provider
    let tracer = tracer_provider.tracer("actix-otel-rs");

    // Set up tracing subscriber with OpenTelemetry
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    let subscriber = Registry::default()
        .with(telemetry)
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer());

    // Initialize the tracing subscriber
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set subscriber");

    // Set the global tracer provider
    global::set_tracer_provider(tracer_provider);

    // Start the server
    HttpServer::new(|| {
        App::new()
            .wrap(actix_web_opentelemetry::RequestTracing::new())
            .route("/", web::get().to(index))
    })
    .bind("0.0.0.0:8080")?
    .bind("0.0.0.0:8887")?  // Metrics endpoint
    .run()
    .await
}