use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    ServiceBuilderExt,
    compression::CompressionLayer,
    decompression::RequestDecompressionLayer,
    normalize_path::NormalizePathLayer,
    request_id::MakeRequestUuid,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::info;

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, registry, util::SubscriberInitExt};

fn setup_tracing_subscriber() {
    registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn create_router() -> Router {
    Router::new().route("/", get(|| async { "Hello" })).layer(
        ServiceBuilder::new()
            .layer(NormalizePathLayer::trim_trailing_slash())
            .layer(RequestDecompressionLayer::new())
            .layer(CompressionLayer::new())
            .set_x_request_id(MakeRequestUuid::default())
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_response(DefaultOnResponse::new().include_headers(true)),
            )
            .propagate_x_request_id(),
    )
}

#[tokio::main]
async fn main() {
    setup_tracing_subscriber();

    let app = create_router();
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Starting server on port 3000...");
    axum::serve(listener, app).await.unwrap();
}
