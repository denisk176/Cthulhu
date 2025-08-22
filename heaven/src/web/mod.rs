use crate::manager::JobManager;
use crate::mqtt::{BroadcastSender, MQTTSender};
use crate::web::pages::{abort, restart_all};
use crate::web::serial::serial_handler;
use axum::body::Body;
use axum::extract::{Path, Request};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Router, middleware};
use axum::middleware::Next;
use cthulhu_config::heaven::HeavenConfig;
use include_dir::{Dir, include_dir};
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use tracing::info;

mod pages;

mod helpers;
mod serial;

#[derive(Clone)]
struct WebState {
    manager: JobManager,
    mqtt: MQTTSender,
    broadcast: BroadcastSender,
}

pub async fn web_main(
    config: HeavenConfig,
    manager: JobManager,
    mqtt: MQTTSender,
    broadcast: BroadcastSender,
) -> color_eyre::Result<()> {
    let state = WebState {
        manager,
        mqtt,
        broadcast,
    };
    let app = Router::new()
        .route("/", get(pages::index::index))
        .route("/portstatus.html", get(pages::index::port_status))
        .route("/restart", get(restart_all))
        .route("/port/{port_label}/", get(pages::port::port))
        .route("/port/{port_label}/header.html", get(pages::port::header))
        .route("/port/{port_label}/devinfo.html", get(pages::port::footer))
        .route("/port/{port_label}/abort", get(abort))
        .route("/port/{port_label}/serial", get(serial_handler))
        .route("/assets/{*path}", get(static_path))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(set_static_cache_control))
                .layer(CatchPanicLayer::new()),
        )
        .with_state(state);

    info!("Starting web...");
    let listener = tokio::net::TcpListener::bind(&config.web.listen_address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/web/assets");

async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    match STATIC_DIR.get_file(path) {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(Body::from(file.contents()))
            .unwrap(),
    }
}

async fn set_static_cache_control(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    response
}
