use crate::crd::Request;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::{core::ObjectMeta, ResourceExt};
use rand::distributions::{Alphanumeric, DistString};
use std::env;

/// Generates a unique resource name
pub fn generate_name() -> String {
    format!(
        "kufefe-generated-{}",
        Alphanumeric.sample_string(&mut rand::thread_rng(), 6)
    )
    .to_lowercase()
}

/// Generates expiry timestamp
pub fn generate_expiry() -> i64 {
    let minutes: i64 = env::var("EXPIRE_MINUTES")
        .unwrap_or_else(|_| "60".to_string())
        .parse::<i64>()
        .unwrap_or(60);

    chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(minutes))
        .unwrap()
        .timestamp()
}

/// Creates metadata for Kubernetes resources
pub fn meta(
    mut name: Option<String>,
    namespace: Option<String>,
    expiry: Option<i64>,
    owner: &Request,
) -> ObjectMeta {
    if name.is_none() {
        name = Some(generate_name());
    }

    let mut meta = ObjectMeta {
        name,
        namespace,
        annotations: Some(annotations(expiry)),
        labels: Some(labels()),
        ..ObjectMeta::default()
    };

    if owner.uid().is_some() {
        meta.owner_references = Some(vec![OwnerReference {
            api_version: "kufefe.io/v1".to_string(),
            kind: "Request".to_string(),
            name: owner.name_any(),
            uid: owner.uid().unwrap(),
            controller: None,
            block_owner_deletion: None,
        }]);
    }

    meta
}

/// Gets BTreeMap of annotations
fn annotations(mut expiry: Option<i64>) -> std::collections::BTreeMap<String, String> {
    let mut m = std::collections::BTreeMap::new();

    if expiry.is_none() {
        expiry = Some(generate_expiry());
    }

    m.insert(
        "kufefe.io/expire-by".to_string(),
        expiry.unwrap().to_string(),
    );

    m
}

/// Gets ownership labels
fn labels() -> std::collections::BTreeMap<String, String> {
    let mut m = std::collections::BTreeMap::new();

    m.insert(
        "app.kubernetes.io/managed-by".to_string(),
        "kufefe".to_string(),
    );

    m
}
