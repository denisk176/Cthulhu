use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse};
use include_dir::{include_dir, Dir};
use serde::Serialize;
use tera::{Tera, Value};
use tera_template_macro::TeraTemplate;
use crate::switch::PortStatus;
use crate::web::manager::{PortManager, PortManagerEntry};

fn csscolor_filter(input: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let status: PortStatus = serde_json::from_value(input.clone())?;
    let color = status.get_css_backgroundcolor();
    Ok(serde_json::to_value(color)?)
}

fn make_tera() -> Tera {
    // let mut t = Tera::new("src/web/templates/**/*").expect("Failed to create Tera instance");
    let mut t = Tera::default();
    for f in TEMPLATE_DIR.files() {
        t.add_raw_template(f.path().to_str().unwrap(), f.contents_utf8().unwrap()).expect("Failed to add template");
    }
    t.register_filter("csscolor", csscolor_filter);
    t
}

static TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/web/templates");


pub static TERA: LazyLock<RwLock<Tera>> = LazyLock::new(|| {
    RwLock::new(make_tera())
});
#[derive(TeraTemplate, Serialize)]
#[template(path="index.html")]
pub struct IndexPageTemplate {
    ports: Vec<PortManagerEntry>,
}
#[derive(TeraTemplate, Serialize)]
#[template(path="port.html")]
pub struct PortPageTemplate {
    port: PortManagerEntry,
    log: String,
}
pub async fn index(State(state): State<PortManager>) -> impl IntoResponse {
    let mut context = IndexPageTemplate {
        ports: state.get_ports().await,
    };

    context.ports.sort_by(|a, b| a.label.cmp(&b.label));

    Html(context.render(TERA.read().unwrap().clone()))
}

pub async fn logs_page(State(state): State<PortManager>, Path(port_label): Path<String>) -> impl IntoResponse {
    let ports = state.get_ports().await;
    let port = ports.into_iter().find(|p| p.label == port_label).unwrap();
    let b = PathBuf::new().join("logs/").join(format!("{}-{}.log", port.label, port.job_started.format("%Y-%m-%d-%H-%M-%S")));
    let context = PortPageTemplate {
        port,
        log: strip_ansi_escapes::strip_str(tokio::fs::read_to_string(&b).await.unwrap()),
    };

    Html(context.render(TERA.read().unwrap().clone()))
}
