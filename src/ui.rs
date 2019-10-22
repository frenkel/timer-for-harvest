use gio::prelude::*;
use gtk::prelude::*;
use harvest::Harvest;
use std::convert::TryInto;
use std::env::args;

fn left_aligned_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label
}
fn load_time_entries() -> gtk::Box {
    let api = Harvest::new();
    /*
        let projects = api.active_projects();
        for project in projects {
            println!("{} {}", project.client.name, project.name);
        }
    */
    let user = api.current_user();
    let time_entries = api.time_entries_today(user);
    let rows = gtk::Box::new(
        gtk::Orientation::Vertical,
        time_entries.len().try_into().unwrap(),
    );

    for time_entry in time_entries {
        println!(
            "{} {}: {} {} at {}",
            time_entry.client.name,
            time_entry.project.name,
            time_entry.hours,
            time_entry.task.name,
            time_entry.spent_date
        );
        let data = gtk::Box::new(gtk::Orientation::Vertical, 3);
        data.pack_start(&left_aligned_label(&time_entry.client.name), true, false, 0);
        data.pack_start(
            &left_aligned_label(&time_entry.project.name),
            true,
            false,
            0,
        );
        data.pack_start(&left_aligned_label(&time_entry.task.name), true, false, 0);
        rows.pack_end(&data, true, false, 0);
    }

    rows
}

pub fn main_window() {
    let application =
        gtk::Application::new(Some("nl.frankgroeneveld.harvest"), Default::default()).unwrap();

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    let container = gtk::HeaderBar::new();

    container.set_title(Some("Harvest"));
    container.set_show_close_button(true);

    window.set_title("Harvest");
    window.set_titlebar(Some(&container));
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(350, 70);

    let button = gtk::Button::new_with_label("Start");
    let application_clone = application.clone();
    let window_clone = window.clone();
    button.connect_clicked(move |_| {
        //print_time_entries();

        let popup = build_popup();
        application_clone.add_window(&popup);
        popup.set_transient_for(Some(&window_clone));
        popup.show_all();

    });

    container.pack_start(&button);

    let time_entries = load_time_entries();
    window.add(&time_entries);

    window.show_all();
}

fn build_popup() -> gtk::Window {
    let popup = gtk::Window::new(gtk::WindowType::Toplevel);

    popup.set_title("Add time entry");
    popup.set_default_size(400, 200);
    popup.set_modal(true);
    popup.set_type_hint(gdk::WindowTypeHint::Dialog);

    popup.connect_delete_event(|_, _| Inhibit(false));

    let list_store = gtk::ListStore::new(&[gtk::Type::String, gtk::Type::U32]);
    let api = Harvest::new();
    let projects = api.active_projects();
    for project in &projects {
        list_store.set(
            &list_store.append(),
            &[0, 1],
            &[
                &format!("{}\n{}", project.client.as_ref().unwrap().name, &project.name),
                &project.id,
            ],
        );
    }

    let data = gtk::Box::new(gtk::Orientation::Vertical, 3);

    let project_chooser = gtk::ComboBox::new_with_model(&list_store);
    let cell = gtk::CellRendererText::new();
    project_chooser.pack_start(&cell, true);
    project_chooser.add_attribute(&cell, "text", 0);
    data.pack_start(&project_chooser, true, false, 0);

    let hour_input = gtk::Entry::new();
    data.pack_start(&hour_input, true, false, 0);

    let start_button = gtk::Button::new_with_label("Start Timer");
    data.pack_start(&start_button, true, false, 0);

    let project_chooser_clone = project_chooser.clone();

    start_button.connect_clicked(move |_| {
        match project_chooser_clone.get_active() {
            Some(size) => {
                println!("{}", projects[size as usize].name);
            }
            None => {
                println!("Nothing selected");
            }
        }
    });

    popup.add(&data);
    popup
}
