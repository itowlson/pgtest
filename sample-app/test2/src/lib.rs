wit_bindgen::generate!({
    world: "spin-pg-event",
    path: "../../spin-pg-event.wit"
});

struct MySpinPgEvent;

impl SpinPgEvent for MySpinPgEvent {
    fn handle_pg_event(row: Row) -> Option<Row> {
        if let Some(index) = row.columns.iter().position(|c| c.name == "description") {
            let mut new = row;
            if let DbValue::Str(orig) = &new.columns[index].value {
                new.columns[index].value = DbValue::Str(format!("{orig} - now a the Sunday Times bestseller"));
                return Some(new);
            }
        }
        None
    }
}

export_spin_pg_event!(MySpinPgEvent);
