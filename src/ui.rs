use gio::prelude::*;
use gtk::prelude::*;
use harvest::Harvest;
use std::convert::TryInto;
use std::env::args;
use std::rc::Rc;

pub struct Ui {
    main_window: gtk::ApplicationWindow,
    api: Harvest,
    start_button: gtk::Button,
}

fn left_aligned_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label
}

pub fn main_window() {
    let application =
        gtk::Application::new(Some("nl.frankgroeneveld.harvest"), Default::default()).unwrap();

    application.connect_activate(|app| {
        let ui = Rc::new(Ui::new(app));
        Ui::load_time_entries(&ui);
        Ui::connect_main_window_events(&ui);
    });

    application.run(&args().collect::<Vec<_>>());
}

impl Ui {
    pub fn new(application: &gtk::Application) -> Ui {
        let window = gtk::ApplicationWindow::new(application);
        let container = gtk::HeaderBar::new();

        container.set_title(Some("Harvest"));
        container.set_show_close_button(true);

        window.set_title("Harvest");
        window.set_titlebar(Some(&container));
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(350, 70);

        window.add_events(gdk::EventMask::KEY_PRESS_MASK);

        let button = gtk::Button::new_with_label("Start");
        container.pack_start(&button);

        Ui { main_window: window, api: Harvest::new(), start_button: button }
    }

    pub fn connect_main_window_events(ui: &Rc<Ui>) {
        let key_press_event_ui_ref = Rc::clone(&ui);
        ui.main_window.connect_key_press_event(move |_window, event| {
            if event.get_keyval() == 65474 {
                /* F5 key pressed */
                Ui::load_time_entries(&key_press_event_ui_ref);
                Inhibit(true)
            } else {
                Inhibit(false)
            }
        });

        let button_ui_ref = Rc::clone(&ui);
        ui.start_button.connect_clicked(move |_| {
            let popup = build_popup(harvest::Timer {
                id: None,
                project_id: 0,
                task_id: 0,
                spent_date: None,
                notes: None,
                hours: None,
                is_running: false,
            });
            button_ui_ref.main_window.get_application().unwrap().add_window(&popup);
            popup.set_transient_for(Some(&button_ui_ref.main_window));
            popup.show_all();
            let delete_event_ref = Rc::clone(&button_ui_ref);
            popup.connect_delete_event(move |_, _| {
                Ui::load_time_entries(&delete_event_ref);
                Inhibit(false)
            });
        });
    }

    fn load_time_entries(ui: &Rc<Ui>) {
        let user = ui.api.current_user();
        let time_entries = ui.api.time_entries_today(user);
        let rows = gtk::Box::new(
            gtk::Orientation::Vertical,
            time_entries.len().try_into().unwrap(),
        );

        for time_entry in time_entries {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 2);
            let data = gtk::Box::new(gtk::Orientation::Vertical, 3);
            let project_client = format!(
                "<b>{}</b> ({})",
                &name_and_code(&time_entry.project),
                &time_entry.client.name
            );
            let project_label = left_aligned_label(&project_client);
            project_label.set_use_markup(true);
            data.pack_start(&project_label, true, false, 0);
            let task_notes = format!(
                "{} - {}",
                &time_entry.task.name,
                &time_entry.notes.as_ref().unwrap().to_string()
            );
            data.pack_start(
                &left_aligned_label(&task_notes),
                true,
                false,
                0,
            );
            row.pack_start(&data, true, true, 0);
            row.pack_start(
                &left_aligned_label(&harvest::f32_to_duration_str(time_entry.hours)),
                false,
                false,
                10,
            );
            let button = gtk::Button::new();
            let rc = Rc::new(time_entry);
            let time_entry_clone = Rc::clone(&rc);
            let button_ui_ref = Rc::clone(&ui);
            if time_entry_clone.is_running {
                button.set_label("Stop");
                button.connect_clicked(move |_| {
                    button_ui_ref.api.stop_timer(&time_entry_clone);
                    Ui::load_time_entries(&button_ui_ref);
                });
                button.get_style_context().add_class("suggested-action");
            } else {
                button.set_label("Start");
                button.connect_clicked(move |_| {
                    button_ui_ref.api.restart_timer(&time_entry_clone);
                    Ui::load_time_entries(&button_ui_ref);
                });
            };

            row.pack_start(&button, false, false, 0);
            let edit_button = gtk::Button::new_with_label("Edit");
            let window_clone2 = ui.main_window.clone();
            let edit_button_ui_ref = Rc::clone(&ui);
            edit_button.connect_clicked(move |_| {
                let notes = match rc.notes.as_ref() {
                    Some(n) => Some(n.to_string()),
                    None => None,
                };
                let popup = build_popup(harvest::Timer {
                    id: Some(rc.id),
                    project_id: rc.project.id,
                    task_id: rc.task.id,
                    spent_date: Some(rc.spent_date.clone()),
                    notes: notes,
                    hours: Some(rc.hours),
                    is_running: rc.is_running,
                });
                window_clone2.get_application().unwrap().add_window(&popup);
                popup.set_transient_for(Some(&window_clone2));
                popup.show_all();
                let delete_event_ui_ref = Rc::clone(&edit_button_ui_ref);
                popup.connect_delete_event(move |_, _| {
                    Ui::load_time_entries(&delete_event_ui_ref);
                    Inhibit(false)
                });
            });
            row.pack_start(&edit_button, false, false, 0);
            rows.pack_end(&row, true, false, 5);
        }

        match ui.main_window.get_children().first() {
            Some(child) => {
                if child.is::<gtk::Box>() {
                    ui.main_window.remove(child);
                }
            }
            None => {}
        }
        ui.main_window.add(&rows);
        ui.main_window.show_all();
    }
}

fn build_popup(timer: harvest::Timer) -> gtk::Window {
    let popup = gtk::Window::new(gtk::WindowType::Toplevel);

    popup.set_title("Add time entry");
    popup.set_default_size(400, 200);
    popup.set_modal(true);
    popup.set_type_hint(gdk::WindowTypeHint::Dialog);

    popup.connect_delete_event(|_, _| Inhibit(false));

    let project_store = gtk::ListStore::new(&[
        gtk::Type::String,
        gtk::Type::U32,
        gtk::Type::String,
        gtk::Type::String,
    ]);
    let api = Harvest::new();
    let user = api.current_user();
    let mut projects = api.active_projects(user);
    projects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    for project in &projects {
        project_store.set(
            &project_store.append(),
            &[0, 1, 2, 3],
            &[
                &name_and_code(&project),
                &project.id,
                &project.code,
                &project.name,
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

    let rc = Rc::new(timer);
    let timer_clone = Rc::clone(&rc);

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
                if timer_clone.task_id > 0 {
                    /* when project_id changes, we might not have a task in the dropdown */
                    task_chooser_clone.set_active_iter(iter_from_id(&task_store_clone, timer_clone.task_id).as_ref());
                }
            }
            None => {}
        }
    });
    let timer_clone2 = Rc::clone(&rc);
    if timer_clone2.project_id > 0 {
        /* TODO handle failure */
        project_chooser.set_active_iter(Some(
            &iter_from_id(&project_store, timer_clone2.project_id).unwrap(),
        ));
    }

    let inputs = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    let notes_input = gtk::Entry::new();
    inputs.pack_start(&notes_input, true, true, 0);
    match &timer_clone2.notes {
        Some(n) => notes_input.set_text(&n),
        None => {}
    }

    let hour_input = gtk::Entry::new();
    inputs.pack_start(&hour_input, false, false, 0);
    match timer_clone2.hours {
        Some(h) => hour_input.set_text(&harvest::f32_to_duration_str(h)),
        None => {}
    }
    hour_input.set_editable(!timer_clone2.is_running);

    data.pack_start(&inputs, true, false, 0);

    let start_button = gtk::Button::new();
    data.pack_start(&start_button, false, false, 0);

    let project_chooser_clone2 = project_chooser.clone();
    let task_chooser_clone2 = task_chooser.clone();
    let project_store_clone2 = project_store.clone();
    let task_store_clone2 = task_store.clone();
    let popup_clone = popup.clone();

    if timer_clone2.id == None {
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
                            harvest::duration_str_to_f32(&hour_input.get_text().unwrap()),
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
        start_button.connect_clicked(move |_| match project_chooser_clone2.get_active() {
            Some(index) => {
                match task_chooser_clone2.get_active() {
                    Some(task_index) => {
                        let project = project_from_index(&project_store_clone2, index);
                        /* TODO remove api init here */
                        let api = Harvest::new();
                        let task = task_from_index(&task_store_clone2, task_index);
                        api.update_timer(&harvest::Timer {
                            id: timer_clone2.id,
                            project_id: project.id,
                            task_id: task.id,
                            notes: Some(notes_input.get_text().unwrap().to_string()),
                            hours: Some(harvest::duration_str_to_f32(
                                &hour_input.get_text().unwrap(),
                            )),
                            is_running: timer_clone2.is_running,
                            spent_date: Some(timer_clone2.spent_date.as_ref().unwrap().to_string()),
                        });
                        popup_clone.close();
                    }
                    None => {}
                }
            }
            None => {}
        });
    }

    popup.add(&data);
    popup
}

fn project_from_index(store: &gtk::ListStore, index: u32) -> harvest::Project {
    let iter = &store.get_iter_from_string(&format!("{}", index)).unwrap();
    let id = store.get_value(iter, 1).get::<u32>().unwrap();
    let code = store.get_value(iter, 2).get::<String>().unwrap();
    let name = store.get_value(iter, 3).get::<String>().unwrap();
    harvest::Project {
        id: id,
        client: None,
        name: name,
        code: code,
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

fn name_and_code(project: &harvest::Project) -> String {
    if project.code == "" {
        project.name.clone()
    } else {
        format!("[{}] {}", project.code, project.name)
    }
}
