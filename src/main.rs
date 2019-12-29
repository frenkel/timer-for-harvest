pub mod ui;
use harvest::Harvest;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let harvest = Rc::new(Harvest::new());
    ui::main_window(harvest);

    Ok(())
}
