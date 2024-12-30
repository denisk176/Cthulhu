use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use tokio::join;
use tower_http::services::ServeDir;
use tracing::info;
use crate::config::Config;
use crate::switch::{PortCommand, SpawnedPort};
use crate::web::manager::PortManager;
use crate::web::pages::{index, logs_page};
use axum::response::Html;

mod manager;
mod pages;

pub async fn abort(State(state): State<PortManager>, Path(port_label): Path<String>) -> impl IntoResponse {
    state.send_command(&port_label, PortCommand::ResetJob).await;
    Html("DONE".to_string())
}

pub async fn web_main(config: Config, ports: Vec<SpawnedPort>) -> color_eyre::Result<()> {
    let (manager, port_jobs) = PortManager::spawn(ports).await?;

    let app = Router::new()
        .route("/", get(index))
        .route("/port/:port_label", get(logs_page))
        .route("/port/:port_label/abort", get(abort))
        .nest_service("/assets", ServeDir::new("src/web/assets"))
        .with_state(manager.clone());

    info!("Starting web...");
    let listener = tokio::net::TcpListener::bind(&config.listen_address).await?;
    let f = axum::serve(listener, app);
    let (a, _) = join!(f, port_jobs);
    a?;
    Ok(())
}