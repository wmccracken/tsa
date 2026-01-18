use anyhow::{Context, Result};
use reqwest::Client;

use crate::models::{
    ContactsResponse, Device, DevicesResponse, RenameDeviceRequest, UpdateTagsRequest, User,
    UsersResponse,
};

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

    pub async fn delete_device(&self, device_id: &str) -> Result<()> {
        let url = format!("{}/device/{}", TAILSCALE_API_BASE, device_id);

        let response = self
            .client
            .delete(&url)
            .basic_auth(&self.api_key, Option::<&str>::None)
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

    pub async fn list_users(&self) -> Result<Vec<User>> {
        let url = format!("{}/tailnet/{}/users", TAILSCALE_API_BASE, self.tailnet);

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

        let users_response: UsersResponse = response
            .json()
            .await
            .context("Failed to parse users response")?;

        Ok(users_response.users)
    }

    pub async fn get_contacts(&self) -> Result<ContactsResponse> {
        let url = format!("{}/tailnet/{}/contacts", TAILSCALE_API_BASE, self.tailnet);

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

        let contacts: ContactsResponse = response
            .json()
            .await
            .context("Failed to parse contacts response")?;

        Ok(contacts)
    }

    pub async fn approve_user(&self, user_id: &str) -> Result<()> {
        let url = format!(
            "{}/tailnet/{}/user/{}/approve",
            TAILSCALE_API_BASE, self.tailnet, user_id
        );

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.api_key, Option::<&str>::None)
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

    pub async fn suspend_user(&self, user_id: &str) -> Result<()> {
        let url = format!(
            "{}/tailnet/{}/user/{}/suspend",
            TAILSCALE_API_BASE, self.tailnet, user_id
        );

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.api_key, Option::<&str>::None)
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

    pub async fn restore_user(&self, user_id: &str) -> Result<()> {
        let url = format!(
            "{}/tailnet/{}/user/{}/restore",
            TAILSCALE_API_BASE, self.tailnet, user_id
        );

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.api_key, Option::<&str>::None)
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

    pub async fn delete_user(&self, user_id: &str) -> Result<()> {
        let url = format!(
            "{}/tailnet/{}/user/{}",
            TAILSCALE_API_BASE, self.tailnet, user_id
        );

        let response = self
            .client
            .delete(&url)
            .basic_auth(&self.api_key, Option::<&str>::None)
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

    pub async fn get_device(&self, device_id: &str) -> Result<Device> {
        let url = format!("{}/device/{}?fields=all", TAILSCALE_API_BASE, device_id);

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

        let device: Device = response
            .json()
            .await
            .context("Failed to parse device response")?;

        Ok(device)
    }

    pub async fn rename_device(&self, device_id: &str, new_name: String) -> Result<()> {
        let url = format!("{}/device/{}/name", TAILSCALE_API_BASE, device_id);

        let request_body = RenameDeviceRequest { name: new_name };

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
