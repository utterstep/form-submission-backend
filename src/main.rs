#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use axum::{routing::post, Router};
use eyre::{Result, WrapErr};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

mod config;
use crate::config::Config;

mod error;

mod handlers;

mod send_mail;

mod state;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // enable tracing
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(2)
                .with_targets(true)
                .with_indent_lines(true)
                .with_bracketed_fields(true)
                .with_thread_names(false)
                .with_thread_ids(false),
        )
        .init();

    let config = Config::from_env()?;
    let app_state = AppState::new(config.clone()).await?;

    let app = Router::new()
        .route("/form/{template_name}/", post(handlers::handle_form))
        .route_layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(config.bind_to())
        .await
        .wrap_err("Failed to bind to address")?;

    axum::serve(listener, app).await?;

    Ok(())
}
