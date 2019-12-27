use gio::prelude::*;
use gtk::prelude::*;
use harvest::Harvest;
use std::convert::TryInto;
use std::env::args;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Ui {
    main_window: gtk::ApplicationWindow,
    api: Rc<Harvest>,
    start_button: gtk::Button,
    time_entries: Rc<RefCell<Vec<TimeEntryRow>>>
}

struct TimeEntryRow {
    time_entry: Rc<RefCell<harvest::TimeEntry>>,
    start_stop_button: gtk::Button,
    edit_button: gtk::Button,
    hours_label: gtk::Label
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
        Ui::connect_main_window_signals(&ui);
        ui.load_time_entries();
        Ui::connect_time_entry_signals(&ui);
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

        Ui {
            main_window: window,
            api: Rc::new(Harvest::new()),
            start_button: button,
            time_entries: Rc::new(RefCell::new(vec!())),
        }
    }

    pub fn connect_main_window_signals(ui: &Rc<Ui>) {
        let key_press_event_ui_ref = Rc::clone(&ui);
        let button_ui_ref = Rc::clone(&ui);
        let open_popup = move |ui_ref: &Rc<Ui>| {
            let popup = ui_ref.build_popup(harvest::Timer {
                id: None,
                project_id: 0,
                task_id: 0,
                spent_date: None,
                notes: None,
                hours: None,
                is_running: false,
            });
            let delete_event_ref = Rc::clone(&ui_ref);
            popup.connect_delete_event(move |_, _| {
                delete_event_ref.load_time_entries();
                Ui::connect_time_entry_signals(&delete_event_ref);
                Inhibit(false)
            });
        };

        ui.main_window
            .connect_key_press_event(move |_window, event| {
                if event.get_keyval() == gdk::enums::key::F5 {
                    key_press_event_ui_ref.load_time_entries();
                    Ui::connect_time_entry_signals(&key_press_event_ui_ref);
                    Inhibit(true)
                } else if event.get_keyval() == gdk::enums::key::n {
                    open_popup(&key_press_event_ui_ref);
                    Inhibit(true)
                } else {
                    Inhibit(false)
                }
            });

        ui.start_button.connect_clicked(move |_| {
            open_popup(&button_ui_ref);
        });
    }

    pub fn connect_time_entry_signals(ui: &Rc<Ui>) {
        for time_entry_row in ui.time_entries.borrow().iter() {
            if time_entry_row.time_entry.borrow().is_running {
                let time_entries_ref = Rc::clone(&ui.time_entries);
                let time_entry_ref = Rc::clone(&time_entry_row.time_entry);
                let hours_label_ref = time_entry_row.hours_label.clone();
                let header_bar_ref = ui.main_window.get_titlebar().unwrap()
                            .downcast::<gtk::HeaderBar>()
                            .unwrap()
                            .clone();

                gtk::timeout_add_seconds(60, move || {
                    let mut mut_time_entry_ref = time_entry_ref.borrow_mut();
                    if mut_time_entry_ref.is_running {
                        mut_time_entry_ref.hours += 1.0 / 60.0;

                        hours_label_ref.set_text(&harvest::f32_to_duration_str(mut_time_entry_ref.hours));

                        let mut total = 0.0;
                        for time_entry_row in time_entries_ref.borrow().iter() {
                            if time_entry_row.time_entry.try_borrow().is_ok() {
                                total += time_entry_row.time_entry.borrow().hours;
                            } else {
                                total += mut_time_entry_ref.hours;
                            }
                        }
                        let title = format!("Harvest - {}", harvest::f32_to_duration_str(total));
                        header_bar_ref.set_title(Some(&title));

                        glib::Continue(true)
                    } else {
                        glib::Continue(false)
                    }
                });
            }

            let api_ref = Rc::clone(&ui.api);
            let ui_ref = Rc::clone(&ui);
            let time_entry_ref = Rc::clone(&time_entry_row.time_entry);
            time_entry_row.start_stop_button.connect_clicked(move |_| {
                if time_entry_ref.borrow().is_running {
                    api_ref.stop_timer(&time_entry_ref.borrow());
                } else {
                    api_ref.restart_timer(&time_entry_ref.borrow());
                }
                ui_ref.load_time_entries();
                Ui::connect_time_entry_signals(&ui_ref);
            });

            let ui_ref2 = Rc::clone(&ui);
            let time_entry_ref2 = Rc::clone(&time_entry_row.time_entry);
            time_entry_row.edit_button.connect_clicked(move |_| {
                let time_entry_ref3 = time_entry_ref2.borrow();
                let notes = match time_entry_ref3.notes.as_ref() {
                    Some(n) => Some(n.to_string()),
                    None => None,
                };
                let popup = ui_ref2.build_popup(harvest::Timer {
                    id: Some(time_entry_ref3.id),
                    project_id: time_entry_ref3.project.id,
                    task_id: time_entry_ref3.task.id,
                    spent_date: Some(time_entry_ref3.spent_date.clone()),
                    notes: notes,
                    hours: Some(time_entry_ref3.hours),
                    is_running: time_entry_ref3.is_running,
                });
                let delete_event_ui_ref = Rc::clone(&ui_ref2);
                popup.connect_delete_event(move |_, _| {
                    delete_event_ui_ref.load_time_entries();
                    Ui::connect_time_entry_signals(&delete_event_ui_ref);
                    Inhibit(false)
                });
            });
        }
    }

    fn load_time_entries(&self) {
        let user = self.api.current_user();
        let time_entries = self.api.time_entries_today(user);
        let mut total_hours = 0.0;
        let rows = gtk::Box::new(
            gtk::Orientation::Vertical,
            time_entries.len().try_into().unwrap(),
        );

        /* stop all running gtk timers */
        for old_entry in self.time_entries.borrow().iter() {
            old_entry.time_entry.borrow_mut().is_running = false;
        }
        /* clear old entries */
        self.time_entries.borrow_mut().clear();

        for time_entry in time_entries {
            total_hours += time_entry.hours;

            let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            let data = gtk::Box::new(gtk::Orientation::Vertical, 3);
            let project_client = format!(
                "<b>{}</b> ({})",
                &time_entry.project.name_and_code(),
                &time_entry.client.name
            );
            let project_label = left_aligned_label(&project_client);
            project_label.set_use_markup(true);
            data.pack_start(&project_label, true, false, 0);
            let notes = match time_entry.notes.as_ref() {
                Some(n) => n.to_string(),
                None => "".to_string(),
            };
            let task_notes = format!("{} - {}", &time_entry.task.name, &notes);
            let notes_label = left_aligned_label(&task_notes);
            notes_label.set_line_wrap(true);
            data.pack_start(&notes_label, true, false, 0);
            row.pack_start(&data, true, true, 0);
            let hours_label = left_aligned_label(&harvest::f32_to_duration_str(time_entry.hours));
            row.pack_start(&hours_label, false, false, 10);
            let button = gtk::Button::new();
            let rc = Rc::new(RefCell::new(time_entry));
            let time_entry_clone = Rc::clone(&rc);
            if time_entry_clone.borrow().is_running {
                button.set_label("Stop");
                button.get_style_context().add_class("suggested-action");
            } else {
                button.set_label("Start");
            };
            let prevent_vexpand = gtk::Box::new(gtk::Orientation::Vertical, 1);
            prevent_vexpand.set_valign(gtk::Align::Center);
            prevent_vexpand.pack_start(&button, false, false, 0);
            row.pack_start(&prevent_vexpand, false, false, 0);
            let edit_button = gtk::Button::new_with_label("Edit");
            let prevent_vexpand = gtk::Box::new(gtk::Orientation::Vertical, 1);
            prevent_vexpand.set_valign(gtk::Align::Center);
            prevent_vexpand.pack_start(&edit_button, false, false, 0);
            row.pack_start(&prevent_vexpand, false, false, 0);
            rows.pack_start(&row, true, false, 5);
            self.time_entries.borrow_mut().push(
                TimeEntryRow {
                    time_entry: rc,
                    start_stop_button: button,
                    edit_button: edit_button,
                    hours_label: hours_label
                }
            );
        }

        let title = format!("Harvest - {}", harvest::f32_to_duration_str(total_hours));
        self.main_window.get_titlebar().unwrap()
                .downcast::<gtk::HeaderBar>()
                .unwrap()
                .set_title(Some(&title));

        match self.main_window.get_children().first() {
            Some(child) => {
                if child.is::<gtk::Box>() {
                    self.main_window.remove(child);
                }
            }
            None => {}
        }
        self.main_window.add(&rows);
        self.main_window.show_all();
    }

    fn build_popup(&self, timer: harvest::Timer) -> gtk::Window {
        let popup = gtk::Window::new(gtk::WindowType::Toplevel);

        popup.set_title("Add time entry");
        popup.set_default_size(400, 200);
        popup.set_modal(true);
        popup.set_type_hint(gdk::WindowTypeHint::Dialog);

        popup.connect_delete_event(|_, _| Inhibit(false));
        popup.add_events(gdk::EventMask::KEY_PRESS_MASK);
        popup.connect_key_press_event(|window, event| {
            if event.get_keyval() == gdk::enums::key::Escape {
                window.close();
                Inhibit(true)
            } else {
                Inhibit(false)
            }
        });

        let project_store = gtk::ListStore::new(&[gtk::Type::String, gtk::Type::U32]);
        let api = Harvest::new();
        let mut project_assignments = api.active_project_assignments();
        project_assignments.sort_by(|a, b| {
            a.project
                .name
                .to_lowercase()
                .cmp(&b.project.name.to_lowercase())
        });
        for project_assignment in &project_assignments {
            project_store.set(
                &project_store.append(),
                &[0, 1],
                &[
                    &project_assignment.project.name_and_code(),
                    &project_assignment.project.id,
                ],
            );
        }
        let project_assignments = Rc::new(project_assignments);

        let data = gtk::Box::new(gtk::Orientation::Vertical, 5);

        let project_chooser = gtk::ComboBox::new_with_model_and_entry(&project_store);
        project_chooser.set_entry_text_column(0);

        let project_completer = gtk::EntryCompletion::new();
        project_completer.set_model(Some(&project_store));
        project_completer.set_text_column(0);
        project_completer.set_match_func(Ui::fuzzy_matching);
        let project_chooser_clone2 = project_chooser.clone();
        project_completer.connect_match_selected(move |_completion, _model, iter| {
            project_chooser_clone2.set_active_iter(Some(&iter));
            Inhibit(false)
        });

        project_chooser
            .get_child()
            .unwrap()
            .downcast::<gtk::Entry>()
            .unwrap()
            .set_completion(Some(&project_completer));
        data.pack_start(&project_chooser, true, false, 0);

        let task_store = gtk::ListStore::new(&[gtk::Type::String, gtk::Type::U32]);
        let task_chooser = gtk::ComboBox::new_with_model_and_entry(&task_store);
        task_chooser.set_entry_text_column(0);

        let task_completer = gtk::EntryCompletion::new();
        task_completer.set_model(Some(&task_store));
        task_completer.set_text_column(0);
        task_completer.set_match_func(Ui::fuzzy_matching);
        let task_chooser_clone2 = task_chooser.clone();
        task_completer.connect_match_selected(move |_completion, _model, iter| {
            task_chooser_clone2.set_active_iter(Some(&iter));
            Inhibit(false)
        });

        task_chooser
            .get_child()
            .unwrap()
            .downcast::<gtk::Entry>()
            .unwrap()
            .set_completion(Some(&task_completer));
        data.pack_start(&task_chooser, true, false, 0);

        let rc = Rc::new(timer);
        let timer_clone = Rc::clone(&rc);

        let task_store_clone = task_store.clone();
        let project_chooser_clone = project_chooser.clone();
        let project_store_clone = project_store.clone();
        let task_chooser_clone = task_chooser.clone();
        let project_assignments_ref = Rc::clone(&project_assignments);
        project_chooser.connect_changed(move |_| {
            task_store_clone.clear();
            match project_chooser_clone.get_active() {
                Some(index) => {
                    let project_assignment = Ui::project_assignment_from_index(
                        &project_store_clone,
                        index,
                        &project_assignments_ref,
                    );
                    match project_assignment {
                        Some(p) => {
                            Ui::load_tasks(&task_store_clone, p);
                            if timer_clone.task_id > 0 {
                                /* when project_id changes, we might not have a task in the dropdown */
                                task_chooser_clone.set_active_iter(
                                    Ui::iter_from_id(&task_store_clone, timer_clone.task_id).as_ref(),
                                );
                            }
                        }
                        None => {}
                    };
                }
                None => {}
            }
        });
        let timer_clone2 = Rc::clone(&rc);
        if timer_clone2.project_id > 0 {
            /* TODO handle failure */
            project_chooser.set_active_iter(Some(
                &Ui::iter_from_id(&project_store, timer_clone2.project_id).unwrap(),
            ));
        }

        let inputs = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        let notes_input = gtk::Entry::new();
        notes_input
            .set_property("activates-default", &true)
            .expect("could not allow default activation");
        inputs.pack_start(&notes_input, true, true, 0);
        match &timer_clone2.notes {
            Some(n) => notes_input.set_text(&n),
            None => {}
        }

        let hour_input = gtk::Entry::new();
        hour_input
            .set_property("activates-default", &true)
            .expect("could not allow default activation");
        inputs.pack_start(&hour_input, false, false, 0);
        match timer_clone2.hours {
            Some(h) => hour_input.set_text(&harvest::f32_to_duration_str(h)),
            None => {}
        }
        hour_input.set_editable(!timer_clone2.is_running);

        data.pack_start(&inputs, true, false, 0);

        let start_button = gtk::Button::new();
        start_button.set_can_default(true);
        data.pack_start(&start_button, false, false, 0);

        let project_chooser_clone2 = project_chooser.clone();
        let task_chooser_clone2 = task_chooser.clone();
        let project_store_clone2 = project_store.clone();
        let task_store_clone2 = task_store.clone();
        let popup_clone = popup.clone();
        let project_assignments_ref2 = Rc::clone(&project_assignments);

        let api_ref = Rc::clone(&self.api);
        if timer_clone2.id == None {
            start_button.set_label("Start Timer");
            start_button.connect_clicked(move |_| match project_chooser_clone2.get_active() {
                Some(index) => {
                    match task_chooser_clone2.get_active() {
                        Some(task_index) => {
                            let project_assignment = Ui::project_assignment_from_index(
                                &project_store_clone2,
                                index,
                                &project_assignments_ref2,
                            )
                            .expect("project not found");
                            let task = Ui::task_from_index(&task_store_clone2, task_index);
                            api_ref.start_timer(
                                &project_assignment.project,
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
                            let project_assignment = Ui::project_assignment_from_index(
                                &project_store_clone2,
                                index,
                                &project_assignments_ref2,
                            )
                            .expect("project not found");
                            let task = Ui::task_from_index(&task_store_clone2, task_index);
                            api_ref.update_timer(&harvest::Timer {
                                id: timer_clone2.id,
                                project_id: project_assignment.project.id,
                                task_id: task.id,
                                notes: Some(notes_input.get_text().unwrap().to_string()),
                                hours: Some(harvest::duration_str_to_f32(
                                    &hour_input.get_text().unwrap(),
                                )),
                                is_running: timer_clone2.is_running,
                                spent_date: Some(
                                    timer_clone2.spent_date.as_ref().unwrap().to_string(),
                                ),
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
        start_button.grab_default();
        self
            .main_window
            .get_application()
            .unwrap()
            .add_window(&popup);
        popup.set_transient_for(Some(&self.main_window));
        popup.show_all();
        popup
    }

    fn project_assignment_from_index<'a>(
        store: &gtk::ListStore,
        index: u32,
        project_assignments: &'a Vec<harvest::ProjectAssignment>,
    ) -> Option<&'a harvest::ProjectAssignment> {
        let iter = &store.get_iter_from_string(&format!("{}", index)).unwrap();
        let id = store.get_value(iter, 1).get::<u32>().unwrap();
        for project_assignment in project_assignments {
            if project_assignment.project.id == id {
                return Some(project_assignment);
            }
        }
        None
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

    fn load_tasks(store: &gtk::ListStore, project_assignment: &harvest::ProjectAssignment) {
        for task_assignment in &project_assignment.task_assignments {
            store.set(
                &store.append(),
                &[0, 1],
                &[&task_assignment.task.name, &task_assignment.task.id],
            );
        }
    }

    fn fuzzy_matching(completion: &gtk::EntryCompletion, key: &str, iter: &gtk::TreeIter) -> bool {
        let store = completion.get_model().unwrap();
        let column_number = completion.get_text_column();
        let row = store
            .get_value(iter, column_number)
            .get::<String>()
            .unwrap();

        /* key is already lower case */
        if row.to_lowercase().contains(key) {
            true
        } else {
            false
        }
    }
}
