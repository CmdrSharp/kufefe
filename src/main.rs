use crate::{config::KufefeConfig, crd::Request};
use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::OnceCell;

mod config;
mod crd;
mod kubeconfig;
mod macros;
mod resources;
mod traits;
mod watcher;

static CONFIG: OnceCell<KufefeConfig> = OnceCell::const_new();

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
    CONFIG
        .set(
            KufefeConfig::new()
                .await
                .expect("Failed to generate Kufefe Configuration"),
        )
        .ok();

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

    // Scan for expired resources
    tracing::info!("Starting watcher for expired resources");
    let crd = Request::mock();

    loop {
        crd.scan().await;
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
