use harvest::Harvest;
use gio::prelude::*;
use gtk::prelude::*;
use std::env::args;

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Harvest");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 70);

    let button = gtk::Button::new_with_label("Start");

    button.connect_clicked(|_| {
        print_time_entries();
    });

    window.add(&button);

    window.show_all();
}

fn print_time_entries() {
    let api = Harvest::new();
    /*
        let projects = api.active_projects();
        for project in projects {
            println!("{} {}", project.client.name, project.name);
        }
    */
    let user = api.current_user();
    let time_entries = api.time_entries_today(user);
    for time_entry in time_entries {
        println!(
            "{} {}: {} {} at {}",
            time_entry.client.name,
            time_entry.project.name,
            time_entry.hours,
            time_entry.task.name,
            time_entry.spent_date
        );
    }
}

fn gtk_window() {
    let application =
        gtk::Application::new(Some("nl.frankgroeneveld.harvest"), Default::default())
        .unwrap();

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}

fn main() -> Result<(), Box<std::error::Error>> {
    gtk_window();

    Ok(())
}
