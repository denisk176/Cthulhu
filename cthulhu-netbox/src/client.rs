use std::collections::BTreeMap;
use color_eyre::eyre::eyre;
use reqwest::{header, Client, Url};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::warn;

pub struct NetboxClient {
    http_client: Client,
    base_url: String,
}

impl NetboxClient {
    pub fn with_url_and_token(url: &str, token: &str) -> color_eyre::Result<Self> {
        let mut headers = HeaderMap::new();

        headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&format!("Token {}", token))?);

        let http_client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            http_client,
            base_url: url.to_string(),
        })
    }

    pub async fn get_device_id_by_serial(&self, sn: &str) -> color_eyre::Result<NetboxDeviceID> {
        let u = format!("{}/api/dcim/devices/", self.base_url);
        let mut retries = 0;
        loop {
            let u = Url::parse_with_params(&u, &[("serial", sn)])?;
            let resp = self.http_client.get(u).send().await?;
            if resp.status().is_server_error() && retries < 3 {
                retries = retries + 1;
                warn!("Error fetching device {sn}, retry...");
                continue;
            }
            let resp = resp.error_for_status()?;
            let resp: NetboxPagedAPIResponse<NetboxDeviceAPIResponse> = resp.json().await?;
            if let Some(d) = resp.results.first() {
                return Ok(d.id)
            } else {
                return Err(eyre!("unable to fetch device by serial"))
            }
        }
    }

    pub async fn set_device_status(&self, id: NetboxDeviceID, status: &str) -> color_eyre::Result<()> {
        let u = format!("{}/api/dcim/devices/{}/", self.base_url, id);
        let body = json!({ "status": status });
        let mut retries = 0;
        loop {
            let resp = self.http_client.patch(&u)
                .json(&body)
                .send().await?;
            if resp.status().is_server_error() && retries < 3 {
                retries = retries + 1;
                warn!("Error updating device {id}, retry...");
            } else {
                let _resp = resp.error_for_status()?;
                break;
            }
        }
        Ok(())
    }

    pub async fn add_device_journal_entry(&self, device_id: NetboxDeviceID, kind: NetboxJournalEntryKind, comment: &str) -> color_eyre::Result<()> {
        let body = NetboxJournalEntryAPIResponse {
            assigned_object_type: "dcim.device".to_string(),
            assigned_object_id: device_id as i64,
            kind,
            comments: comment.to_string(),
            custom_fields: Default::default(),
        };
        let u = format!("{}/api/extras/journal-entries/", self.base_url);
        let mut retries = 0;
        loop {
            let resp = self.http_client.post(&u)
                .json(&body)
                .send().await?;
            if resp.status().is_server_error() && retries < 3 {
                retries = retries + 1;
                warn!("Error updating device {id}, retry...");
            } else {
                let _resp = resp.error_for_status()?;
                break;
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetboxJournalEntryKind {
    Info,
    Success,
    Warning,
    Danger,
}

pub type NetboxDeviceID = usize;

#[derive(Debug, Clone, Deserialize)]
struct NetboxPagedAPIResponse<D> {
    results: Vec<D>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct NetboxDeviceAPIResponse {
    name: String,
    id: NetboxDeviceID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct NetboxJournalEntryAPIResponse {
    assigned_object_type: String,
    assigned_object_id: i64,
    kind: NetboxJournalEntryKind,
    comments: String,
    custom_fields: BTreeMap<String, String>,
}