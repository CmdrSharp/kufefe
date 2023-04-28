use crate::crd::Request;
use crate::resources::{
    rolebinding::RoleBinding, serviceaccount::ServiceAccount, token::Token,
};
use crate::traits::api::ApiResource;
use crate::traits::expire::Expire;
use kube::{Client, Config};
use std::env;
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::OnceCell;

mod crd;
mod kubeconfig;
mod meta;
mod resources;
mod traits;
mod watcher;

static CLIENT: OnceCell<Client> = OnceCell::const_new();
static NAMESPACE: OnceCell<String> = OnceCell::const_new();
static CLUSTER_URL: OnceCell<String> = OnceCell::const_new();

#[tokio::main]
async fn main() {
    // Setup Tracing
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "kufefe=info");
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    // Set up statics
    let config = Config::infer().await.expect("Failed to infer config");

    CLUSTER_URL
        .set(env::var("CLUSTER_URL").unwrap_or(config.cluster_url.to_string()))
        .expect("Failed to set CLUSTER_URL");

    NAMESPACE
        .set(env::var("NAMESPACE").unwrap_or_else(|_| "default".to_string()))
        .expect("Failed to set NAMESPACE");

    if CLIENT.set(Client::try_default().await.unwrap()).is_err() {
        tracing::error!("Failed to set CLIENT");
        std::process::exit(2);
    }

    // Thread for handling signals
    tokio::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let mut sigint = signal(SignalKind::interrupt()).unwrap();

        loop {
            select! {
                _ = sigterm.recv() => {
                    tracing::info!("SIGTERM received, exiting");
                    std::process::exit(0);
                }
                _ = sigint.recv() => {
                    tracing::info!("SIGINT received, exiting");
                    std::process::exit(0);
                }
            }
        }
    });

    // Bootstrap Controller for CRD's
    watcher::watch().await;

    // Set up API's
    let crd = Request::mock();
    let sa = ServiceAccount::new();
    let rb = RoleBinding::new();
    let tk = Token::new();

    // Scan for expired resources
    tracing::info!("Starting watcher for expired resources");

    loop {
        crd.scan(crd.get_api()).await;
        sa.scan(sa.get_api()).await;
        rb.scan(rb.get_api()).await;
        tk.scan(tk.get_api()).await;

        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
