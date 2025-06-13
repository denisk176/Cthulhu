use crate::manager::PortManagerEntry;
use crate::web::tera::TERA;
use crate::web::WebState;
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse};
use cthulhu_common::status::JobCommand;
use serde::Serialize;
use tera_template_macro::TeraTemplate;

#[derive(TeraTemplate, Serialize)]
#[template(path="index.html")]
pub struct IndexPageTemplate {
    ports: Vec<PortManagerEntry>,
}

#[derive(TeraTemplate, Serialize)]
#[template(path="index-portstatus.html")]
pub struct IndexPortStatusPageTemplate {
    ports: Vec<PortManagerEntry>,
}
#[derive(TeraTemplate, Serialize)]
#[template(path="port.html")]
pub struct PortPageTemplate {
    port: PortManagerEntry,
}

#[derive(TeraTemplate, Serialize)]
#[template(path="port-header.html")]
pub struct PortHeaderPageTemplate {
    port: PortManagerEntry,
}

#[derive(TeraTemplate, Serialize)]
#[template(path="port-devinfo.html")]
pub struct PortDevInfoPageTemplate {
    port: PortManagerEntry,
}

pub async fn index(State(state): State<WebState>) -> impl IntoResponse {
    let mut context = IndexPageTemplate {
        ports: state.manager.get_ports().await,
    };

    context.ports.sort_by(|a, b| a.label.cmp(&b.label));

    let tera = TERA.read().await.clone();
    Html(context.render(&tera))
}

pub async fn index_portstatus(State(state): State<WebState>) -> impl IntoResponse {
    let mut context = IndexPortStatusPageTemplate {
        ports: state.manager.get_ports().await,
    };

    context.ports.sort_by(|a, b| a.label.cmp(&b.label));

    let tera = TERA.read().await.clone();
    Html(context.render(&tera))
}

pub async fn logs_page(State(state): State<WebState>, Path(port_label): Path<String>) -> impl IntoResponse {
    let port = state.manager.get_port(&port_label).await.unwrap();
    let context = PortPageTemplate {
        port,
    };

    let tera = TERA.read().await.clone();
    Html(context.render(&tera))
}

pub async fn header_page(State(state): State<WebState>, Path(port_label): Path<String>) -> impl IntoResponse {
    let port = state.manager.get_port(&port_label).await.unwrap();
    let context = PortHeaderPageTemplate {
        port,
    };

    let tera = TERA.read().await.clone();
    Html(context.render(&tera))
}

pub async fn devinfo_page(State(state): State<WebState>, Path(port_label): Path<String>) -> impl IntoResponse {
    let port = state.manager.get_port(&port_label).await.unwrap();
    let context = PortDevInfoPageTemplate {
        port,
    };

    let tera = TERA.read().await.clone();
    Html(context.render(&tera))
}

pub async fn abort(State(state): State<WebState>, Path(port_label): Path<String>) -> impl IntoResponse {
    state.mqtt.send_command(&port_label, JobCommand::ResetJob).await.unwrap();
    Html("DONE".to_string())
}
