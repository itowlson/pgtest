mod run;
mod trigger;

use std::sync::OnceLock;

use pgrx::prelude::*;

pgrx::pg_module_magic!();

#[derive(thiserror::Error, Debug)]
enum TriggerError {
    #[error("Null Trigger Tuple found")]
    NullTriggerTuple,
    #[error("PgHeapTuple error: {0}")]
    PgHeapTuple(#[from] pgrx::heap_tuple::PgHeapTupleError),
    #[error("TryFromDatumError error: {0}")]
    TryFromDatum(#[from] pgrx::datum::TryFromDatumError),
    #[error("TryFromInt error: {0}")]
    TryFromInt(#[from] std::num::TryFromIntError),
    #[error("CantTable error: {0}")]
    CantTable(#[from] PgTriggerError),
    #[error("CantApp error: {0}")]
    CantApp(#[from] anyhow::Error),
}

fn sample_app() -> run::App {
    run::App::new(
        "ghcr.io/itowlson/pg-app-example:v2",
        "/home/ivan/testing/pgtest/STATEYWATEY",
    )
}

fn make_running_app(app: &run::App) -> anyhow::Result<run::RunningApp> {
    let rt = tokio::runtime::Runtime::new()?;
    let ra = rt.block_on(async {
        run::run(app).await
    })?;
    println!("Loaded Spin app");
    Ok(ra)
}

static CELL: OnceLock<run::RunningApp> = OnceLock::new();

#[pg_trigger]
fn pass_to_spin_trigger<'a>(trigger: &'a pgrx::PgTrigger<'a>,) -> Result<Option<PgHeapTuple<'a, impl WhoAllocated>>, TriggerError> {
    println!("Postgres trigger running");

    let ra = CELL.get_or_init(|| {
        let ra = make_running_app(&sample_app()).unwrap(); //.map_err(|e| TriggerError::CantApp(e))?;
        ra
    });

    let table = trigger.table_name().map_err(|e| TriggerError::CantTable(e))?;
    let mut new = trigger.new().ok_or(TriggerError::NullTriggerTuple)?.into_owned();
    let col_name = "title";

    if let Some(col_value) = new.get_by_name(col_name)? {
        let new_value = ra.handle_pg_event(&table, col_value)?;
        new.set_by_name(col_name, new_value)?;
    }

    Ok(Some(new))
}

extension_sql!(
    r#"
CREATE TABLE test (
    id serial8 NOT NULL PRIMARY KEY,
    title varchar(50),
    description text,
    payload jsonb
);

CREATE TABLE test2 (
    id serial8 NOT NULL PRIMARY KEY,
    title varchar(50),
    description text
);

CREATE TRIGGER test_trigger BEFORE INSERT ON test FOR EACH ROW EXECUTE PROCEDURE pass_to_spin_trigger();
CREATE TRIGGER test2_trigger BEFORE INSERT ON test2 FOR EACH ROW EXECUTE PROCEDURE pass_to_spin_trigger();
-- INSERT INTO test (title, description, payload) VALUES ('Fox', 'a description', '{"key": "value"}');
"#,
    name = "create_trigger",
    requires = [pass_to_spin_trigger]
);

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_insert() -> Result<(), spi::Error> {
        Spi::run(
            r#"INSERT INTO test (title, description, payload) VALUES ('Fox', 'foxy goodness', '{"key": "value"}')"#,
        )?;
        Spi::run(
            r#"INSERT INTO test (title, description, payload) VALUES ('Box', 'a different description', '{"key": "value"}')"#,
        )?;
        Spi::run(
            r#"INSERT INTO test (title, description, payload) VALUES ('Locks', 'a different description', '{"key": "value"}')"#,
        )?;
        Spi::run(
            r#"INSERT INTO test2 (title, description) VALUES ('test2 title', 'whee')"#,
        )?;
        Spi::run(
            r#"INSERT INTO test2 (title, description) VALUES ('Pride and Penguins', 'Jane Austen but with waterfowl')"#,
        )?;

        Spi::connect(|client| {
            show(&client)
        })?;

        Ok(())

    }

    fn show<'conn>(client: &spi::SpiClient<'conn>) -> Result<(), spi::Error> {
        let res = client.select("SELECT * FROM test", None, None)?;
        for r in res {
            let title: String = r.get_by_name("title")?.unwrap_or_default();
            let desc: String = r.get_by_name("description")?.unwrap_or_default();
            println!("[test row]  title='{title}' | desc = '{desc}'");
        }

        let res = client.select("SELECT * FROM test2", None, None)?;
        for r in res {
            let title: String = r.get_by_name("title")?.unwrap_or_default();
            let desc: String = r.get_by_name("description")?.unwrap_or_default();
            println!("[test2 row] title='{title}' | desc = '{desc}'");
        }

        Ok(())
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
