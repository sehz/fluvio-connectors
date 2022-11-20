use std::sync::Arc;

use clap::Parser;
use futures::StreamExt;

use connector_common::git_hash_version;
use connector_common::metrics::ConnectorMetrics;
use connector_common::monitoring::init_monitoring;
use connector_model_sql::Operation;
use connector_sink_sql::db::Db;
use connector_sink_sql::opt::SqlConnectorOpt;
use fluvio_future::tracing::{debug, info};
use schemars::schema_for;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let metrics = Arc::new(ConnectorMetrics::new());
    init_monitoring(metrics.clone());
    if let Some("metadata") = std::env::args().nth(1).as_deref() {
        let schema = serde_json::json!({
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION"),
            "description": env!("CARGO_PKG_DESCRIPTION"),
            "direction": "Sink",
            "schema": schema_for!(SqlConnectorOpt),
        });
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        return Ok(());
    }
    let raw_opts = SqlConnectorOpt::from_args();
    raw_opts.common.enable_logging();
    info!(
        connector_version = env!("CARGO_PKG_VERSION"),
        git_hash = git_hash_version(),
        "starting JSON SQL sink connector",
    );
    let mut db = Db::connect(raw_opts.database_url.as_str()).await?;
    info!("connected to database {}", db.kind());

    let mut stream = raw_opts.common.create_consumer_stream("sql").await?;
    info!("connected to fluvio stream");

    info!(
        "starting stream processing from {}",
        raw_opts.common.fluvio_topic
    );
    while let Some(Ok(consumer_record)) = stream.next().await {
        let operation: Operation = serde_json::from_slice(consumer_record.as_ref())?;
        debug!("{:?}", operation);
        db.execute(operation).await?;
    }

    Ok(())
}