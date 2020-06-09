mod app;
mod ui;

use app::App;
use std::env::args;
use std::sync::mpsc;
use timer_for_harvest::Harvest;
use ui::Ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();

    if args.len() == 2 && &args[1] == "--version" {
        println!("{}", Harvest::user_agent());
    } else {
        let (to_ui, from_app) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let (to_app, from_ui) = mpsc::channel();

        let app = App::new(from_ui, to_ui);
        let ui = Ui::new(from_app, to_app);
        App::run(app);
        Ui::run(ui);
    }

    Ok(())
}
