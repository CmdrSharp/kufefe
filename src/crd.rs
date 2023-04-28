use crate::delete::DeleteOpt;
use crate::CLIENT;
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
    pub generated_name: String,
    pub kubeconfig: Option<String>,
    pub ready: bool,
    pub expires_at: Option<i64>,
}

impl RequestStatus {
    /// Update status of the CRD
    pub async fn update(
        &self,
        resource: &Request,
        name: &str,
    ) -> Result<(), kube::Error> {
        let client = CLIENT.get().unwrap().clone();
        let api: Api<Request> = Api::all(client);

        let mut status = api.get_status(&resource.name_any()).await?;
        status.status = Some(RequestStatus {
            generated_name: name.to_string(),
            kubeconfig: self.kubeconfig.clone(),
            ready: self.ready,
            expires_at: self.expires_at,
        });

        api.replace_status(
            &resource.name_any(),
            &PostParams::default(),
            serde_json::to_vec(&status).expect("Failed to serialize status"),
        )
        .await?;

        Ok(())
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
    fn is_expired(&self, request: &Request) -> bool {
        if let Some(status) = &request.status {
            if let Some(expires_at) = status.expires_at {
                if expires_at < chrono::Utc::now().timestamp() {
                    return true;
                }
            }
        }

        false
    }

    // Get the API for the CRD
    pub fn get_api(&self) -> Api<Self> {
        let client = CLIENT.get().unwrap().clone();
        let api: Api<Request> = Api::all(client);

        api
    }
}
