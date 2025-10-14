use axum::{Router, routing::get};
use tokio::net::TcpListener;

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, registry, util::SubscriberInitExt};

fn setup_tracing_subscriber() {
    registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[tokio::main]
async fn main() {
    setup_tracing_subscriber();

    let app = Router::new().route("/", get(|| async { "Hello" }));
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
