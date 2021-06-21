use super::constants;
use super::discover;
use super::pair::Pairing;
use super::error::{Error, Result};

use reqwest::Client;

use std::error;
use std::time::Duration;

/// A Vizio Device
#[derive(Debug, Clone)]
pub struct Device {
    friendly_name: String,
    manufacturer: String,
    model_name: String,
    ip_addr: String,
    port: u16,
    uuid: String,
    auth_token: Option<String>,
    client: Option<Client>,
    pub(crate) pairing: Option<Pairing>,
}

impl Device {
    pub(crate) fn new(
        friendly_name: String,
        manufacturer: String,
        model_name: String,
        ip_addr: String,
        uuid: String
    ) -> Self {
        Self {
            friendly_name,
            manufacturer,
            model_name,
            ip_addr,
            port: 7345,
            uuid,
            auth_token: None,
            client: None,
            pairing: None,
        }
    }

    pub(crate) fn build_client(&mut self) -> Result<()> {
        if self.client.is_none() {
            self.client = Some(
                reqwest::Client::builder()
                .timeout(Duration::from_secs(constants::DEFAULT_TIMEOUT))
                .https_only(true)
                .build()?
            );
        }
        Ok(())
    }

    /// Get device's model name
    pub fn model_name(&self) -> String {
        self.model_name.clone()
    }

    /// Get device's 'friendly' name
    pub fn friendly_name(&self) -> String {
        self.friendly_name.clone()
    }

    /// Get device's local IP
    pub fn ip(&self) -> String {
        self.ip_addr.clone()
    }

    /// Get device's port
    pub fn port(&self) -> u16 {
        // TO-DO: Verify port (dependant on device firmware)
        self.port
    }

    /// Get device's UUID
    pub fn uuid(&self) -> String {
        self.uuid.clone()
    }

    /// Get client's auth token for device
    pub fn auth_token(&self) -> Option<String> {
        self.auth_token.clone()
    }

    /// Set client's auth token for device (if already paired). Returns an error
    /// if connection fails.
    pub fn set_auth_token(&mut self, token: String) -> Result<()> {
        self.auth_token = Some(token);
        // TO-DO: Verify token
        Ok(())
    }
}
