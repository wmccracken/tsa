use anyhow::{Context, Result};
use reqwest::Client;

use crate::models::{Device, DevicesResponse, UpdateTagsRequest};

const TAILSCALE_API_BASE: &str = "https://api.tailscale.com/api/v2";

pub struct TailscaleClient {
    client: Client,
    api_key: String,
    tailnet: String,
}

impl TailscaleClient {
    pub fn new(api_key: String, tailnet: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            tailnet,
        }
    }

    pub async fn list_devices(&self) -> Result<Vec<Device>> {
        let url = format!(
            "{}/tailnet/{}/devices?fields=all",
            TAILSCALE_API_BASE, self.tailnet
        );

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.api_key, Option::<&str>::None)
            .send()
            .await
            .context("Failed to send request to Tailscale API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, body);
        }

        let devices_response: DevicesResponse = response
            .json()
            .await
            .context("Failed to parse devices response")?;

        Ok(devices_response.devices)
    }

    pub async fn update_tags(&self, device_id: &str, tags: Vec<String>) -> Result<()> {
        let url = format!("{}/device/{}/tags", TAILSCALE_API_BASE, device_id);

        let request_body = UpdateTagsRequest { tags };

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.api_key, Option::<&str>::None)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Tailscale API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API request failed with status {}: {}", status, body);
        }

        Ok(())
    }
}
