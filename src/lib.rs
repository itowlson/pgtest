use std::sync::RwLock;

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
}

struct ArseBiscuit {
    value: RwLock<i32>,
}

lazy_static::lazy_static! {
    static ref PLOP: ArseBiscuit = ArseBiscuit { value: RwLock::new(123) };
}

#[pg_trigger]
fn triggly_wiggly<'a>(trigger: &'a pgrx::PgTrigger<'a>,) -> Result<Option<PgHeapTuple<'a, impl WhoAllocated>>, TriggerError> {
    println!("I AM TRIGGLY WIGGLY");
    {
        let mut wr = PLOP.value.write().unwrap();
        *wr = *wr + 1;
    }
    {
        let rd = PLOP.value.read().unwrap();
        println!("PLOPPLY BOPPLY {}", *rd);
    }

    let mut new = trigger.new().ok_or(TriggerError::NullTriggerTuple)?.into_owned();
    let col_name = "title";

    if new.get_by_name(col_name)? == Some("Fox") {
        new.set_by_name(col_name, "Bear")?;
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

CREATE TRIGGER test_trigger BEFORE INSERT ON test FOR EACH ROW EXECUTE PROCEDURE triggly_wiggly();
-- INSERT INTO test (title, description, payload) VALUES ('Fox', 'a description', '{"key": "value"}');
"#,
    name = "create_trigger",
    requires = [triggly_wiggly]
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
            r#"INSERT INTO test (title, description, payload) VALUES ('a different title 2', 'a different description', '{"key": "value"}')"#,
        )?;
        Spi::run(
            r#"INSERT INTO test (title, description, payload) VALUES ('a different title 3', 'a different description', '{"key": "value"}')"#,
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
            println!("ROW! title='{title}' | desc = '{desc}'");
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
