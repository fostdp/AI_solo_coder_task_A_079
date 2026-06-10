mod config;
mod data_loader;
mod handlers;
mod models;
mod route_analyzer;
mod survival_model;

use axum::{
    Router,
    routing::get,
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tower_http::cors::CorsLayer;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use once_cell::sync::OnceCell;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use metrics::counter;

static PROMETHEUS_HANDLE: OnceCell<PrometheusHandle> = OnceCell::new();

fn init_metrics() {
    let builder = PrometheusBuilder::new();
    match builder.install() {
        Ok(handle) => {
            let _ = PROMETHEUS_HANDLE.set(handle);
            tracing::info!("Prometheus metrics recorder installed");
        }
        Err(e) => {
            tracing::warn!("Failed to install Prometheus recorder: {}", e);
        }
    }
}

async fn metrics_handler() -> impl IntoResponse {
    match PROMETHEUS_HANDLE.get() {
        Some(handle) => {
            let metrics = handle.render();
            (
                StatusCode::OK,
                [("content-type", "text/plain; version=0.0.4")],
                metrics,
            )
                .into_response()
        }
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Metrics exporter not initialized",
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "silkroad_analysis=info,tower_http=info,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_thread_ids(false)
            .with_file(false)
        )
        .init();

    dotenvy::dotenv().ok();

    init_metrics();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/silkroad".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    tracing::info!("Connected to PostgreSQL database");

    let frontend_dir = env::var("FRONTEND_DIR")
        .unwrap_or_else(|_| "../frontend".to_string());

    let app = Router::new()
        .nest("/api", handlers::routes(pool.clone()))
        .route("/metrics", get(metrics_handler))
        .fallback_service(ServeDir::new(&frontend_dir))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new()
            .quality(tower_http::compression::CompressionQuality::Default)
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(tower_http::trace::DefaultMakeSpan::new()
                    .level(tracing::Level::INFO)
                    .include_headers(false),
                )
                .on_response(tower_http::trace::DefaultOnResponse::new()
                    .level(tracing::Level::INFO)
                    .latency_unit(tower_http::LatencyUnit::Millis),
                ),
        );

    let addr = env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("Server running on http://{}", addr);
    tracing::info!("API base: http://{}/api", addr);
    tracing::info!("Metrics endpoint: http://{}/metrics", addr);
    tracing::info!("Frontend served from: {}", frontend_dir);
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    counter!("shutdown_total", "reason" => "signal").increment(1);
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Ctrl+C received, shutting down...");
        },
        _ = terminate => {
            tracing::info!("SIGTERM received, shutting down...");
        },
    }
}
