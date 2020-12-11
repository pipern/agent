use std::env;
use std::ffi::OsString;

use kube::config::Config as KubeConfig;
use kube::config::KubeConfigOptions;
use kubelet::config::Config;
use kubelet::Kubelet;
use log::{info, warn};
use stackable_config::ConfigBuilder;

use crate::agentconfig::AgentConfig;
use crate::provider::StackableProvider;

mod agentconfig;
mod provider;

#[tokio::main(threaded_scheduler)]
async fn main() -> anyhow::Result<()> {
    // Initialize the logger
    env_logger::init();

    let agent_config: AgentConfig =
        ConfigBuilder::build(env::args_os().collect::<Vec<OsString>>(), "CONFIG_FILE")
            .expect("Error initializing Configuration!");

    // Currently the only way to _properly_ configure the Krustlet is via these environment exports,
    // as their config object only offers methods that parse from command line flags (or combinations
    // of those flags with other things).
    // Since we have our own command line flags that are not compatible with the Krustlet's we
    // configure the agent via a file from the environment variable (CONFIG_FILE), extract what
    // is needed for the Krustlet and pass it via environment variables.
    // This is an ugly hack for now, until we've had time to take a proper look at Krustlet's config
    export_env(
        "KRUSTLET_NODE_IP",
        &agent_config.server_ip_address.to_string(),
    );

    // Convert node tags to string in the form of key=value,key=value,...
    // TODO: check for commas in the key value pairs themselves
    let node_labels = agent_config
        .tags
        .iter()
        .map(|(k, v)| format!("{}={}", String::from(k), String::from(v)))
        .collect::<Vec<_>>()
        .join(",");

    export_env("NODE_LABELS", &node_labels);

    if let Some(cert_file_path) = agent_config.server_cert_file {
        export_env("KRUSTLET_CERT_FILE", cert_file_path.to_str().unwrap());
    } else {
        warn!("Not exporting server cert file path, as non was specified that could be converted to a String.");
    }

    if let Some(key_file_path) = agent_config.server_key_file {
        export_env("KRUSTLET_PRIVATE_KEY_FILE", key_file_path.to_str().unwrap());
    } else {
        warn!("Not exporting server key file path, as non was specified that could be converted to a String.");
    }
    info!("args: {:?}", env::args());
    let krustlet_config = Config::new_from_flags(env!("CARGO_PKG_VERSION"));

    let kubeconfig = KubeConfig::from_kubeconfig(&KubeConfigOptions::default())
        .await
        .expect("Failed to create Kubernetes Client!");

    let provider = StackableProvider::new(
        kube::Client::new(kubeconfig.clone()),
        agent_config.parcel_directory.clone(),
        agent_config.config_directory.clone(),
        agent_config.log_directory.clone(),
    )
    .await
    .expect("Error initializing provider.");

    let kubelet = Kubelet::new(provider, kubeconfig, krustlet_config).await?;
    kubelet.start().await
}

fn export_env(var_name: &str, var_value: &str) {
    info!("Exporting {}={}", var_name, var_value);
    std::env::set_var(var_name, var_value);
}