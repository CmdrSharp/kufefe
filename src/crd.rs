use crate::traits::{api::ApiResource, delete::DeleteOpt, expire::Expire};
use crate::{status_update, CONFIG};
use anyhow::{bail, Result};
use kube::api::{DeleteParams, ListParams};
use kube::{
    api::{Api, PostParams},
    ResourceExt,
};
use kube_derive::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
#[kube(
    group = "kufefe.io",
    version = "v1",
    kind = "Request",
    status = "RequestStatus"
)]
#[serde(rename_all = "camelCase")]
pub struct RequestSpec {
    pub role: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestStatus {
    pub service_account_name: String,
    pub token_name: String,
    pub rolebinding_name: String,
    pub kubeconfig: Option<String>,
    pub ready: bool,
    pub failed: bool,
    pub message: String,
    pub expires_at: Option<i64>,
}

impl RequestStatus {
    /// Creates a new ResourceStatus object but tries to find an existing one
    pub async fn new(resource: &Request) -> Self {
        let api = resource.get_api();

        match api.get_status(&resource.name_any()).await {
            Ok(res) => {
                tracing::debug!("Status found for {}", resource.name_any());
                res.status.unwrap()
            }
            Err(_) => {
                tracing::debug!(
                    "Failed to find status for {}. Falling back to defaults.",
                    resource.name_any()
                );

                let mut res = resource.clone();
                res.status = Some(RequestStatus::default());

                res.status.unwrap()
            }
        }
    }

    /// Update status of the CRD
    pub async fn update(&self, resource: &Request) -> Result<()> {
        let api = resource.get_api();

        let mut status = api.get_status(&resource.name_any()).await?;
        status.status = Some(RequestStatus {
            ..resource.status.clone().unwrap()
        });

        match api
            .replace_status(
                &resource.name_any(),
                &PostParams::default(),
                serde_json::to_vec(&status).expect("Failed to serialize status"),
            )
            .await
        {
            Ok(_) => {
                tracing::info!("Updated status for {}", resource.name_any());
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    "Failed to update status for {}: {}",
                    resource.name_any(),
                    e
                );
                bail!(e)
            }
        }
    }
}

impl Request {
    /// Creates a mock object
    pub fn mock() -> Self {
        Self {
            metadata: Default::default(),
            spec: Default::default(),
            status: Default::default(),
        }
    }

    /// Scan for expired Requests
    pub async fn scan(&self) {
        tracing::info!("Scanning for expired requests");

        let api = &self.get_api().clone();

        match api.list(&ListParams::default()).await {
            Ok(requests) => {
                for request in &requests.items {
                    if self.is_expired(request) {
                        tracing::info!("Deleting expired request {}", request.name_any());

                        if let Err(err) = api
                            .delete_opt(&request.name_any(), &DeleteParams::default())
                            .await
                        {
                            tracing::error!("Failed to delete request: {}", err);
                            continue;
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to list requests: {}", e);
            }
        }
    }

    /// Checks if the object is expired
    pub fn is_expired(&self, request: &Request) -> bool {
        if let Some(status) = &request.status {
            if let Some(expires_at) = status.expires_at {
                if expires_at < chrono::Utc::now().timestamp() {
                    return true;
                }
            }
        }

        false
    }

    /// Updates the status of the CRD
    pub async fn update_status(&mut self) -> Result<&mut Self> {
        if let Some(status) = &self.status {
            if let Err(err) = status.update(self).await {
                bail!("Failed to update status: {}", err);
            }

            return Ok(self);
        }

        bail!("")
    }

    status_update!(ready, ready: bool);

    status_update!(failed, failed: bool);

    status_update!(message, message: String);

    status_update!(
        account_names,
        service_account_name: String,
        token_name: String,
        rolebinding_name: String
    );

    /// Sets the expired at
    pub fn expires_at(&mut self, expires_at: i64) -> &mut Self {
        if let Some(status) = self.status.take() {
            self.status = Some(RequestStatus {
                expires_at: Some(expires_at),
                ..status
            });
        }

        self
    }

    /// Sets the kubeconfig
    pub fn kubeconfig(&mut self, kubeconfig: &str) -> &mut Self {
        if let Some(status) = self.status.take() {
            self.status = Some(RequestStatus {
                kubeconfig: Some(kubeconfig.to_string()),
                ..status
            });
        }

        self
    }
}

impl ApiResource for Request {
    type ApiType = Request;

    fn get_api(&self) -> Api<Self::ApiType> {
        let client = CONFIG.get().unwrap().client();
        let api: Api<Request> = Api::all(client);

        api
    }
}

impl Expire for Request {}
