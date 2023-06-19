pub struct App {
    reference: String,
    state_dir: String,
}

impl App {
    pub fn new(reference: impl Into<String>, state_dir: impl Into<String>) -> Self {
        Self {
            reference: reference.into(),
            state_dir: state_dir.into(),
        }
    }
}

pub struct RunningApp {
    trigger: std::sync::Arc<PgEventTrigger>,
    rt: tokio::runtime::Runtime,
}

impl RunningApp {
    pub fn handle_pg_event(&self, table: &str, row: RowParam<'_>) -> anyhow::Result<Option<RowResult>> {
        // println!("handling a pg-event");
        let trigger = self.trigger.clone();
        self.rt.block_on(async move {
            trigger.handle_pg_event(table, row).await
        })
    }
}

pub async fn run(app: &App) -> anyhow::Result<RunningApp> {
    let working_dir = tempfile::tempdir()?;

    let locked_app = prepare_app_from_oci(&app.reference, working_dir.path()).await?;
    let locked_url = write_locked_app(&locked_app, working_dir.path()).await?;

    let loader = spin_trigger::loader::TriggerLoader::new(working_dir.path(), false);
    let init_data = HostComponentInitData::default();

    let trigger = build_executor(&app, loader, locked_url, init_data).await?;

    // let pg_run_config = spin_trigger::cli::NoArgs;  // thankfully unused in this case but would need to figure out how to get it in if it was a thing
    let rt = tokio::runtime::Runtime::new()?;
    Ok(RunningApp { trigger: std::sync::Arc::new(trigger), rt })
}

// Copied and trimmed down from spin trigger

use spin_app::Loader;
use spin_trigger::{HostComponentInitData, RuntimeConfig, TriggerExecutorBuilder};
use crate::trigger::{RowParam, RowResult};

use super::trigger::PgEventTrigger;

async fn build_executor(
    app: &App,
    loader: impl Loader + Send + Sync + 'static,
    locked_url: String,
    init_data: HostComponentInitData,
) -> Result<PgEventTrigger> {
    let runtime_config = build_runtime_config(&app.state_dir)?;

    let mut builder = TriggerExecutorBuilder::new(loader);
    builder.wasmtime_config_mut().cache_config_load_default()?;

    builder.build(locked_url, runtime_config, init_data).await
}

fn build_runtime_config(state_dir: impl Into<String>) -> Result<RuntimeConfig> {
    let mut config = RuntimeConfig::new(None);
    config.set_state_dir(state_dir);
    Ok(config)
}

// Copied and trimmed down from spin up

use anyhow::{anyhow, Context, Result};
use spin_app::locked::LockedApp;
use spin_oci::OciLoader;
use std::path::Path;
use url::Url;

async fn prepare_app_from_oci(reference: &str, working_dir: &Path) -> Result<LockedApp> {
    let mut client = spin_oci::Client::new(false, None)
        .await
        .context("cannot create registry client")?;

    OciLoader::new(working_dir)
        .load_app(&mut client, reference)
        .await
}

async fn write_locked_app(
    locked_app: &LockedApp,
    working_dir: &Path,
) -> Result<String, anyhow::Error> {
    let locked_path = working_dir.join("spin.lock");
    let locked_app_contents =
        serde_json::to_vec_pretty(&locked_app).context("failed to serialize locked app")?;
    tokio::fs::write(&locked_path, locked_app_contents)
        .await
        .with_context(|| format!("failed to write {:?}", locked_path))?;
    let locked_url = Url::from_file_path(&locked_path)
        .map_err(|_| anyhow!("cannot convert to file URL: {locked_path:?}"))?
        .to_string();

    Ok(locked_url)
}
