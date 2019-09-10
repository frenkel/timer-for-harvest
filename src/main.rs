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
    let list_store = gtk::ListStore::new(&[gtk::Type::String]);

    list_store.set(&list_store.append(), &[0], &[&"Test".to_string()]);
    let combo_box = gtk::ComboBox::new_with_model(&list_store);
    let cell = gtk::CellRendererText::new();
    combo_box.pack_start(&cell, true);
    combo_box.add_attribute(&cell, "text", 0);
    button.connect_clicked(|_| {
        print_time_entries();
    });

    // window.add(&button);
    window.add(&combo_box);

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
