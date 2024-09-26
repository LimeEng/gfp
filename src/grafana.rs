use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct Grafana {
    http: reqwest::Client,
    domain: String,
    username: String,
    password: String,
}

impl Grafana {
    #[must_use]
    pub fn new(domain: String, username: String, password: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            domain,
            username,
            password,
        }
    }

    pub async fn public_url_of_dashboard(&self, uid: &str) -> Result<String, Error> {
        let access_token = match self.make_dashboard_public(uid).await {
            Ok(value) => Ok(value.access_token),
            Err(_) => self
                .get_public_dashboard(uid)
                .await
                .map(|value| value.access_token),
        }?;
        tracing::debug!("Received accessToken: {access_token}");
        let url = format!("{}/public-dashboards/{access_token}", self.domain);
        Ok(url)

        // let status = self.get_public_dashboard(uid).await?;

        // if status.is_enabled {
        //     let url = format!("{}/public-dashboards/{}", self.domain, status.access_token);
        //     Ok(url)
        // } else {
        //     let response = self.make_dashboard_public(uid).await?;
        //     let url = format!(
        //         "{}/public-dashboards/{}",
        //         self.domain, response.access_token
        //     );
        //     Ok(url)
        // }
    }

    async fn get_public_dashboard(&self, uid: &str) -> Result<PublicDashboardStatus, Error> {
        let url = format!(
            "{}/api/dashboards/uid/{}/public-dashboards",
            self.domain, uid
        );
        tracing::debug!("About to call {url}");
        let response = self
            .http
            .get(url)
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .send()
            .await;
        // .map_err(|_| Error::Network)?;

        tracing::debug!("Got response: {response:?}");
        let response = response.map_err(|_| Error::Network)?;

        if response.status() == StatusCode::OK {
            let status = response
                .json::<PublicDashboardStatus>()
                .await
                .map_err(|_| Error::Api(None))?;
            tracing::debug!("Status code 200");
            Ok(status)
        } else {
            tracing::debug!("Status code != 200");
            Err(Error::Api(response.json::<ErrorResponse>().await.ok()))
        }
    }

    async fn make_dashboard_public(&self, uid: &str) -> Result<PublicDashboardCreated, Error> {
        let url = format!(
            "{}/api/dashboards/uid/{}/public-dashboards",
            self.domain, uid
        );
        tracing::debug!("About to call {url}");
        let body = HashMap::from([("isEnabled", true)]);
        let response = self
            .http
            .post(url)
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .json(&body)
            .send()
            .await;
        // .map_err(|_| Error::Network)?;

        tracing::debug!("Got response: {response:?}");
        let response = response.map_err(|_| Error::Network)?;

        if response.status() == StatusCode::OK {
            let response = response
                .json::<PublicDashboardCreated>()
                .await
                .map_err(|_| Error::Api(None))?;
            tracing::debug!("Status code 200");
            Ok(response)
        } else {
            tracing::debug!("Status code != 200");
            Err(Error::Api(response.json::<ErrorResponse>().await.ok()))
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicDashboardCreated {
    pub access_token: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicDashboardStatus {
    pub access_token: String,
    pub is_enabled: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ErrorResponse {
    pub status_code: u16,
    pub message_id: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub enum Error {
    Network,
    Api(Option<ErrorResponse>),
}
