pub mod background;
pub mod popup;
pub mod ui;

use std::env::args;
use timer_for_harvest::Harvest;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();

    if args.len() == 2 && &args[1] == "--version" {
        println!("{}", Harvest::user_agent());
    } else {
        ui::main_window();
    }

    Ok(())
}
