mod app;
mod popup;
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

        let app = App::new(to_ui);
        let ui = Ui::new(to_app);

        App::handle_ui_signals(app, from_ui);
        Ui::handle_app_signals(ui, from_app);
    }

    Ok(())
}
