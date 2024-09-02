use actix_web::{web, App, HttpServer, Responder};
use opentelemetry::global;
use opentelemetry::trace::{Tracer, TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::runtime::Tokio;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};
use tracing_opentelemetry::OpenTelemetryLayer;

async fn index() -> impl Responder {
    let tracer = global::tracer("actix-otel-rs");
    let _span = tracer.start("processing request");

    info!("Processing the request");

    let meter = global::meter("actix-otel-rs");
    let counter = meter.u64_counter("requests_processed").init();

    counter.add(1, &[]);

    "Hello, OpenTelemetry with Actix-web!".to_string()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Set up OpenTelemetry Tracing with batch processing
    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter().tonic().with_endpoint("http://otel-collector:4317"),
        )
        .install_batch(Tokio)
        .expect("Error initializing tracer provider.");

    // Get a tracer from the provider
    let tracer = tracer_provider.tracer("actix-otel-rs");

    // Set up tracing with OpenTelemetry layer
    let otel_layer = OpenTelemetryLayer::new(tracer);

    Registry::default()
        .with(otel_layer)
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Set the global tracer provider
    global::set_tracer_provider(tracer_provider);

    // Start the server
    HttpServer::new(|| {
        App::new()
            .wrap(actix_web_opentelemetry::RequestTracing::new())
            .route("/", web::get().to(index))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}