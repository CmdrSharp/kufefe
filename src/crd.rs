use crate::traits::{api::ApiResource, expire::Expire};
use crate::CLIENT;
use async_trait::async_trait;
use kube::Resource;
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
}

#[async_trait]
impl Expire for Request {
    /// Gets the expiry of the resource
    async fn get_expiry<T>(&self, resource: T) -> Option<i64>
    where
        T: Resource<DynamicType = ()> + Clone + Send + Sync,
    {
        let api = &self.get_api();
        let name = resource.name_any();

        match api.get(&name).await {
            Ok(request) => {
                let status = if let Some(status) = &request.status {
                    status
                } else {
                    return None;
                };

                status.expires_at
            }
            Err(_) => None,
        }
    }

    /// Don't require a managed_by label as this is a CRD
    fn require_managed_by_label() -> bool {
        false
    }
}

impl ApiResource for Request {
    type ApiType = Request;

    fn get_api(&self) -> Api<Self::ApiType> {
        let client = CLIENT.get().unwrap().clone();
        let api: Api<Request> = Api::all(client);

        api
    }
}
