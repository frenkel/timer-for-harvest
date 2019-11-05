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
    let user = api.current_user();
    let time_entries = api.time_entries_today(user);
    let rows = gtk::Box::new(
        gtk::Orientation::Vertical,
        time_entries.len().try_into().unwrap(),
    );

    for time_entry in time_entries {
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
        let popup = build_popup();
        application_clone.add_window(&popup);
        popup.set_transient_for(Some(&window_clone));
        popup.show_all();
        let window_clone2 = window_clone.clone();
        popup.connect_delete_event(move |_, _| {
            let time_entries = load_time_entries();
            window_clone2.remove(window_clone2.get_children().first().unwrap());
            window_clone2.add(&time_entries);
            window_clone2.show_all();
            Inhibit(false)
        });
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

    let project_store = gtk::ListStore::new(&[gtk::Type::String, gtk::Type::U32]);
    let api = Harvest::new();
    let user = api.current_user();
    let projects = api.active_projects(user);
    for project in &projects {
        project_store.set(
            &project_store.append(),
            &[0, 1],
            &[
                &project.name,
                &project.id,
            ],
        );
    }

    let data = gtk::Box::new(gtk::Orientation::Vertical, 5);

    let project_chooser = gtk::ComboBox::new_with_model(&project_store);
    let cell = gtk::CellRendererText::new();
    project_chooser.pack_start(&cell, true);
    project_chooser.add_attribute(&cell, "text", 0);
    data.pack_start(&project_chooser, true, false, 0);

    let task_store = gtk::ListStore::new(&[gtk::Type::String, gtk::Type::U32]);
    let task_chooser = gtk::ComboBox::new_with_model(&task_store);
    let cell = gtk::CellRendererText::new();
    task_chooser.pack_start(&cell, true);
    task_chooser.add_attribute(&cell, "text", 0);
    data.pack_start(&task_chooser, true, false, 0);

    let task_store_clone = task_store.clone();
    let project_chooser_clone = project_chooser.clone();
    let project_store_clone = project_store.clone();
    project_chooser.connect_changed(move |_| {
        task_store_clone.clear();
        match project_chooser_clone.get_active() {
            Some(index) => {
                load_tasks(&task_store_clone, project_from_index(&project_store_clone, index));
            }
            None => {}
        }
    });

    let inputs = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    let description_input = gtk::Entry::new();
    inputs.pack_start(&description_input, true, true, 0);

    let hour_input = gtk::Entry::new();
    inputs.pack_start(&hour_input, false, false, 0);

    data.pack_start(&inputs, true, false, 0);

    let start_button = gtk::Button::new_with_label("Start Timer");
    data.pack_start(&start_button, false, false, 0);

    let project_chooser_clone2 = project_chooser.clone();
    let task_chooser_clone2 = task_chooser.clone();
    let project_store_clone2 = project_store.clone();
    let task_store_clone2 = task_store.clone();
    let popup_clone = popup.clone();

    start_button.connect_clicked(move |_| match project_chooser_clone2.get_active() {
        Some(index) => {
            match task_chooser_clone2.get_active() {
                Some(task_index) => {
                    let project = project_from_index(&project_store_clone2, index);
                    /* TODO remove api init here */
                    let api = Harvest::new();
                    let task = task_from_index(&task_store_clone2, task_index);
                    api.start_timer(&project, &task);
                    popup_clone.close();
                }
                None => {}
            }
        }
        None => {}
    });

    popup.add(&data);
    popup
}

fn project_from_index(store: &gtk::ListStore, index: u32) -> harvest::Project {
    let iter = &store.get_iter_from_string(&format!("{}", index)).unwrap();
    let id = store.get_value(iter, 1).get::<u32>().unwrap();
    let name = store.get_value(iter, 0).get::<String>().unwrap();
    harvest::Project {
        id: id,
        client: None,
        name: name,
    }
}

fn task_from_index(store: &gtk::ListStore, index: u32) -> harvest::Task {
    let iter = &store.get_iter_from_string(&format!("{}", index)).unwrap();
    let id = store.get_value(iter, 1).get::<u32>().unwrap();
    let name = store.get_value(iter, 0).get::<String>().unwrap();
    harvest::Task { id: id, name: name }
}

fn load_tasks(store: &gtk::ListStore, project: harvest::Project) {
    /* TODO remove api init here */
    let api = Harvest::new();
    for task_assignment in &api.project_task_assignments(&project) {
        store.set(
            &store.append(),
            &[0, 1],
            &[&task_assignment.task.name, &task_assignment.task.id],
        );
    }
}
