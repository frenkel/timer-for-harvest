pub mod popup;
pub mod ui;

use std::env::args;
use std::rc::Rc;
use timer_for_harvest::Harvest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();

    if args.len() == 2 && &args[1] == "--version" {
        println!("{}", Harvest::user_agent());
    } else {
        let harvest = Rc::new(Harvest::new());
        ui::main_window(harvest);
    }

    Ok(())
}
