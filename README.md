# pgtest - run Spin handlers for Postgres database triggers

This repo explores what it might look like to embed Spin in Postgres to handle database triggers.

The current demo is very limited:

* It handles only INSERT triggers
* It hardwires a specific sample application rather than it being configurable
* It handles only a tiny subset of data types (strings. It handles strings)
* It blocks liberally
* It has been tested only in the transient `pgrx` test environment

## To try it out

* You will need a recent version of Rust (get it from https://rustup.rs/)
* Install `cargo-pgrx` via `cargo install --locked cargo-pgrx` (see https://github.com/tcdi/pgrx#getting-started for more info)
* Run `cargo pgrx init`
* Run `cargo test -- --nocapture`

(You don't need Postgres - `pgrx init` does all the groundwork for you.)

You should (after a decently long wait and some spew) see output like:

```
Postgres trigger running
Loaded Spin app
Postgres trigger running
Postgres trigger running
Postgres trigger running
Postgres trigger running
[test row]  title='MOAR Fox' | desc = 'foxy goodness'
[test row]  title='MOAR Box' | desc = 'a different description'
[test row]  title='MOAR Locks' | desc = 'a different description'
[test2 row] title='test2 title' | desc = 'whee - now a the Sunday Times bestseller'
[test2 row] title='Pride and Penguins' | desc = 'Jane Austen but with waterfowl - now a the Sunday Times bestseller'
test tests::pg_test_insert ... ok
```

This is showing the results of the sample triggers running against the INSERTS (in the `test_insert` test method).

## Sample application

The sample handler is in the `sample-app` directory.  If you want to try this with your own apps, you will need to push your app to a registry (`spin registry push`) and edit the reference in `lib.rs` (`sample_app` function) to match.  The sample handler is pre-pushed to a public registry.
