use crate::{traits::api::ApiResource, CONFIG};
use kube::api::Api;
use kube_derive::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema, Default)]
#[kube(
    group = "cluster.k8s.io",
    version = "v1alpha1",
    kind = "Cluster",
    status = "ClusterStatus",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct ClusterSpec {
    pub provider_spec: ProviderSpec,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSpec {
    pub value: ProviderSpecValue,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Default)]
pub struct ProviderSpecValue {
    #[serde(rename = "loadBalancerIP")]
    pub load_balancer_ip: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClusterStatus {
    pub api_endpoints: Vec<ClusterStatusApiEndpoints>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ClusterStatusApiEndpoints {
    pub host: String,
    pub port: i32,
}

impl ApiResource for Cluster {
    type ApiType = Cluster;

    fn get_api(&self) -> Api<Self::ApiType> {
        let client = CONFIG.get().unwrap().client();
        let api: Api<Self::ApiType> = Api::all(client);

        api
    }
}
