use gio::prelude::*;
use gtk::prelude::*;
use harvest::Harvest;
use std::convert::TryInto;
use std::env::args;
use std::rc::Rc;

fn left_aligned_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label
}
fn load_time_entries(window: &gtk::ApplicationWindow) {
    /* TODO remove api init here */
    let api = Harvest::new();
    let user = api.current_user();
    let time_entries = api.time_entries_today(user);
    let rows = gtk::Box::new(
        gtk::Orientation::Vertical,
        time_entries.len().try_into().unwrap(),
    );

    for time_entry in time_entries {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        let data = gtk::Box::new(gtk::Orientation::Vertical, 3);
        data.pack_start(&left_aligned_label(&time_entry.client.name), true, false, 0);
        data.pack_start(
            &left_aligned_label(&time_entry.project.name),
            true,
            false,
            0,
        );
        data.pack_start(&left_aligned_label(&time_entry.task.name), true, false, 0);
        row.pack_start(&data, true, false, 0);
        let button = gtk::Button::new();
        let window_clone = window.clone();
        let rc = Rc::new(time_entry);
        let time_entry_clone = Rc::clone(&rc);
        if time_entry_clone.is_running {
            button.set_label("Stop");
            button.connect_clicked(move |_| {
                /* TODO remove api init here */
                let api = Harvest::new();
                api.stop_timer(&time_entry_clone);
                load_time_entries(&window_clone.clone());
            });
        } else {
            button.set_label("Start");
            button.connect_clicked(move |_| {
                /* TODO remove api init here */
                let api = Harvest::new();
                api.restart_timer(&time_entry_clone);
                load_time_entries(&window_clone.clone());
            });
        };

        row.pack_start(&button, true, false, 0);
        let edit_button = gtk::Button::new_with_label("Edit");
        let window_clone2 = window.clone();
        edit_button.connect_clicked(move |_| {
            let popup = build_popup(
                Some(rc.project.id),
                Some(rc.task.id),
                &rc.notes.as_ref().unwrap(),
                rc.hours,
                rc.is_running
            );
            window_clone2.get_application().unwrap().add_window(&popup);
            popup.set_transient_for(Some(&window_clone2));
            popup.show_all();
        });
        row.pack_start(&edit_button, true, false, 0);
        rows.pack_end(&row, true, false, 0);
    }

    match window.get_children().first() {
        Some(child) => {
            if child.is::<gtk::Box>() {
                window.remove(child);
            }
        }
        None => {}
    }
    window.add(&rows);
    window.show_all();
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
        let popup = build_popup(None, None, &"", 0.0, false);
        application_clone.add_window(&popup);
        popup.set_transient_for(Some(&window_clone));
        popup.show_all();
        let window_clone2 = window_clone.clone();
        popup.connect_delete_event(move |_, _| {
            load_time_entries(&window_clone2.clone());
            Inhibit(false)
        });
    });

    container.pack_start(&button);

    load_time_entries(&window.clone());
}

/* TODO use only TimeEntry argument */
fn build_popup(
    project_id: Option<u32>,
    task_id: Option<u32>,
    notes: &str,
    hours: f32,
    is_running: bool
) -> gtk::Window {
    let popup = gtk::Window::new(gtk::WindowType::Toplevel);

    popup.set_title("Add time entry");
    popup.set_default_size(400, 200);
    popup.set_modal(true);
    popup.set_type_hint(gdk::WindowTypeHint::Dialog);

    popup.connect_delete_event(|_, _| Inhibit(false));

    let project_store = gtk::ListStore::new(&[gtk::Type::String, gtk::Type::U32]);
    let api = Harvest::new();
    let user = api.current_user();
    let mut projects = api.active_projects(user);
    projects.sort_by(|a, b| a.name.cmp(&b.name));
    for project in &projects {
        project_store.set(
            &project_store.append(),
            &[0, 1],
            &[&project.name, &project.id],
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
    let task_chooser_clone = task_chooser.clone();
    project_chooser.connect_changed(move |_| {
        task_store_clone.clear();
        match project_chooser_clone.get_active() {
            Some(index) => {
                load_tasks(
                    &task_store_clone,
                    project_from_index(&project_store_clone, index),
                );
                match task_id {
                    Some(id) => {
                        /* TODO handle failure */
                        task_chooser_clone
                            .set_active_iter(Some(&iter_from_id(&task_store_clone, id).unwrap()));
                    }
                    None => {}
                }
            }
            None => {}
        }
    });
    match project_id {
        Some(id) => {
            /* TODO handle failure */
            project_chooser.set_active_iter(Some(&iter_from_id(&project_store, id).unwrap()));
        }
        None => {}
    }

    let inputs = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    let notes_input = gtk::Entry::new();
    inputs.pack_start(&notes_input, true, true, 0);
    notes_input.set_text(&notes);

    let hour_input = gtk::Entry::new();
    inputs.pack_start(&hour_input, false, false, 0);
    hour_input.set_text(&f32_to_duration_str(hours));
    hour_input.set_editable(!is_running);

    data.pack_start(&inputs, true, false, 0);

    let start_button = gtk::Button::new();
    data.pack_start(&start_button, false, false, 0);

    let project_chooser_clone2 = project_chooser.clone();
    let task_chooser_clone2 = task_chooser.clone();
    let project_store_clone2 = project_store.clone();
    let task_store_clone2 = task_store.clone();
    let popup_clone = popup.clone();

    if project_id == None {
        start_button.set_label("Start Timer");
        start_button.connect_clicked(move |_| match project_chooser_clone2.get_active() {
            Some(index) => {
                match task_chooser_clone2.get_active() {
                    Some(task_index) => {
                        let project = project_from_index(&project_store_clone2, index);
                        /* TODO remove api init here */
                        let api = Harvest::new();
                        let task = task_from_index(&task_store_clone2, task_index);
                        api.start_timer(
                            &project,
                            &task,
                            &notes_input.get_text().unwrap(),
                            duration_str_to_f32(&hour_input.get_text().unwrap()),
                        );
                        popup_clone.close();
                    }
                    None => {}
                }
            }
            None => {}
        });
    } else {
        start_button.set_label("Save Timer");
    }

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

fn iter_from_id(store: &gtk::ListStore, id: u32) -> Option<gtk::TreeIter> {
    let iter = store.get_iter_first().unwrap();
    loop {
        if store.get_value(&iter, 1).get::<u32>().unwrap() == id {
            return Some(iter);
        }
        if !store.iter_next(&iter) {
            break;
        }
    }
    None
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

/* TODO move to TimeEntry */
fn duration_str_to_f32(duration: &str) -> f32 {
    if duration.len() > 0 {
        let mut parts = duration.split(":");
        /* TODO handle errors */
        let hours: f32 = parts.next().unwrap().parse().unwrap();
        /* TODO handle errors */
        let minutes: f32 = parts.next().unwrap().parse().unwrap();
        hours + minutes / 60.0
    } else {
        0.0
    }
}

/* TODO move to TimeEntry */
fn f32_to_duration_str(duration: f32) -> String {
    let minutes = duration % 1.0;
    let hours = duration - minutes;

    format!("{:.0}:{:0<2.0}", hours, minutes * 60.0)
}
