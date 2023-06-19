use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use spin_core::async_trait;
use spin_trigger::{
    EitherInstance, TriggerAppEngine, TriggerExecutor,
};

wasmtime::component::bindgen!({
    path: "spin-pg-event.wit",
    world: "spin-pg-event",
    async: true
});

pub(crate) type RuntimeData = ();
pub(crate) type _Store = spin_core::Store<RuntimeData>;

pub struct PgEventTrigger {
    engine: TriggerAppEngine<Self>,
    component_tables: HashMap<String, String>,
}

// Application settings (raw serialization format)
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriggerMetadata {
    r#type: String,
}

// Per-component settings (raw serialization format)
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PgEventTriggerConfig {
    component: String,
    table: String,
}

// const TRIGGER_METADATA_KEY: MetadataKey<TriggerMetadata> = MetadataKey::new("trigger");

#[async_trait]
impl TriggerExecutor for PgEventTrigger {
    const TRIGGER_TYPE: &'static str = "pg-event";

    type RuntimeData = RuntimeData;

    type TriggerConfig = PgEventTriggerConfig;

    type RunConfig = spin_trigger::cli::NoArgs;

    async fn new(engine: spin_trigger::TriggerAppEngine<Self>) -> anyhow::Result<Self> {
        let component_tables = engine
            .trigger_configs()
            .map(|(_, config)| (config.table.clone(), config.component.clone()))
            .collect();

        Ok(Self {
            engine,
            component_tables,
        })
    }

    async fn run(self, _config: Self::RunConfig) -> anyhow::Result<()> {
        // // This trigger spawns threads, which Ctrl+C does not kill.  So
        // // for this case we need to detect Ctrl+C and shut those threads
        // // down.  For simplicity, we do this by terminating the process.
        // tokio::spawn(async move {
        //     tokio::signal::ctrl_c().await.unwrap();
        //     std::process::exit(0);
        // });

        // loop { //for line in std::io::stdin().lines() {
        //     let mut line = String::new();
        //     std::io::stdin().read_line(&mut line)?;
        //     match line.split_once(":") {
        //         Some((tbl, text)) => {
        //             let new_text = self.handle_pg_event(tbl, text).await?;
        //             println!("result: {new_text}");
        //         },
        //         None => {
        //             println!("bad command");
        //         }   
        //     }
        // };
        todo!("rework Spin trigger `run` for standalone execution")
    }
}

impl PgEventTrigger {
    pub async fn handle_pg_event(&self, table: &str, row: RowParam<'_>) -> anyhow::Result<Option<RowResult>> {
        match self.component_tables.get(table) {
            Some(c) => {
                let new_row = self.handle_pg_event_core(c, row).await?;
                Ok(new_row)
            },
            None => {
                println!("no event set up for table");
                Ok(None)
            }
        }
    }

    async fn handle_pg_event_core(&self, component_id: &str, row: RowParam<'_>) -> anyhow::Result<Option<RowResult>> {
        // Load the guest...
        let (instance, mut store) = self.engine.prepare_instance(component_id).await?;
        let EitherInstance::Component(instance) = instance else {
            unreachable!()
        };
        let instance = SpinPgEvent::new(&mut store, &instance)?;
        // ...and call the entry point
        instance.call_handle_pg_event(&mut store, row).await
    }
}
