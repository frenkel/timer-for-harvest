use crate::popup::Popup;

use gio::prelude::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::convert::TryInto;
use std::env::args;
use std::rc::Rc;
use timer_for_harvest::*;

pub struct Ui {
    main_window: gtk::ApplicationWindow,
    pub api: Rc<Harvest>,
    start_button: gtk::Button,
    time_entries: Rc<RefCell<Vec<TimeEntryRow>>>,
    pub project_assignments: Rc<RefCell<Vec<ProjectAssignment>>>,
}

struct TimeEntryRow {
    time_entry: Rc<RefCell<TimeEntry>>,
    start_stop_button: gtk::Button,
    edit_button: gtk::Button,
    hours_label: gtk::Label,
}

fn left_aligned_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label
}

pub fn main_window(harvest: Rc<Harvest>) {
    let application = gtk::Application::new(
        Some("nl.frankgroeneveld.timer-for-harvest"),
        Default::default(),
    )
    .unwrap();

    application.connect_activate(move |app| {
        let ui = Rc::new(Ui::new(Rc::clone(&harvest), app));
        Ui::connect_main_window_signals(&ui);
        ui.load_time_entries();
        Ui::connect_time_entry_signals(&ui);
    });

    application.run(&args().collect::<Vec<_>>());
}

impl Ui {
    pub fn new(harvest: Rc<Harvest>, application: &gtk::Application) -> Ui {
        let window = gtk::ApplicationWindow::new(application);
        let container = gtk::HeaderBar::new();

        container.set_title(Some("Harvest"));
        container.set_show_close_button(true);

        window.set_title("Harvest");
        window.set_titlebar(Some(&container));
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(500, 300);
        window.set_size_request(500, 300);

        window.add_events(gdk::EventMask::KEY_PRESS_MASK);

        let button = gtk::Button::new_with_label("Start");
        container.pack_start(&button);

        let mut project_assignments = harvest.active_project_assignments();
        project_assignments.sort_by(|a, b| {
            a.project
                .name
                .to_lowercase()
                .cmp(&b.project.name.to_lowercase())
        });

        Ui {
            main_window: window,
            api: harvest,
            start_button: button,
            time_entries: Rc::new(RefCell::new(vec![])),
            project_assignments: Rc::new(RefCell::new(project_assignments)),
        }
    }

    pub fn connect_main_window_signals(ui: &Rc<Ui>) {
        let key_press_event_ui_ref = Rc::clone(&ui);
        let button_ui_ref = Rc::clone(&ui);
        let open_popup = move |ui_ref: &Rc<Ui>| {
            let project_assignments_ref = Rc::clone(&ui_ref.project_assignments);
            let popup = Popup::new(
                Timer {
                    id: None,
                    project_id: 0,
                    task_id: 0,
                    spent_date: None,
                    notes: None,
                    hours: None,
                    is_running: false,
                },
                project_assignments_ref,
                ui_ref.main_window.clone(),
            );
            Popup::connect_signals(&Rc::new(popup), &ui_ref);
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
                let header_bar_ref = ui
                    .main_window
                    .get_titlebar()
                    .unwrap()
                    .downcast::<gtk::HeaderBar>()
                    .unwrap()
                    .clone();

                gtk::timeout_add_seconds(60, move || {
                    let mut mut_time_entry_ref = time_entry_ref.borrow_mut();
                    if mut_time_entry_ref.is_running {
                        mut_time_entry_ref.hours += 1.0 / 60.0;

                        hours_label_ref.set_text(&f32_to_duration_str(mut_time_entry_ref.hours));

                        let mut total = 0.0;
                        for time_entry_row in time_entries_ref.borrow().iter() {
                            if time_entry_row.time_entry.try_borrow().is_ok() {
                                total += time_entry_row.time_entry.borrow().hours;
                            } else {
                                total += mut_time_entry_ref.hours;
                            }
                        }
                        let title = format!("Harvest - {}", f32_to_duration_str(total));
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
                let project_assignments_ref = Rc::clone(&ui_ref2.project_assignments);
                let time_entry_ref3 = time_entry_ref2.borrow();
                let notes = match time_entry_ref3.notes.as_ref() {
                    Some(n) => Some(n.to_string()),
                    None => None,
                };
                let popup = Popup::new(
                    Timer {
                        id: Some(time_entry_ref3.id),
                        project_id: time_entry_ref3.project.id,
                        task_id: time_entry_ref3.task.id,
                        spent_date: Some(time_entry_ref3.spent_date.clone()),
                        notes: notes,
                        hours: Some(time_entry_ref3.hours),
                        is_running: time_entry_ref3.is_running,
                    },
                    project_assignments_ref,
                    ui_ref2.main_window.clone(),
                );
                Popup::connect_signals(&Rc::new(popup), &ui_ref2);
            });
        }
    }

    pub fn load_time_entries(&self) {
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
            let data = gtk::Box::new(gtk::Orientation::Vertical, 2);
            let project_client = format!(
                "<b>{}</b> ({})",
                &time_entry.project.name_and_code(),
                &time_entry.client.name
            );
            let project_label = left_aligned_label(&project_client);
            project_label.set_line_wrap(true);
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
            let hours_label = left_aligned_label(&f32_to_duration_str(time_entry.hours));
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
            let prevent_vexpand = gtk::Box::new(gtk::Orientation::Vertical, 0);
            prevent_vexpand.set_valign(gtk::Align::Center);
            prevent_vexpand.pack_start(&button, false, false, 0);
            row.pack_start(&prevent_vexpand, false, false, 0);
            let edit_button = gtk::Button::new_with_label("Edit");
            let prevent_vexpand = gtk::Box::new(gtk::Orientation::Vertical, 0);
            prevent_vexpand.set_valign(gtk::Align::Center);
            prevent_vexpand.pack_start(&edit_button, false, false, 0);
            row.pack_start(&prevent_vexpand, false, false, 0);
            rows.pack_end(&row, true, false, 0);
            self.time_entries.borrow_mut().push(TimeEntryRow {
                time_entry: rc,
                start_stop_button: button,
                edit_button: edit_button,
                hours_label: hours_label,
            });
        }

        let title = format!("Harvest - {}", f32_to_duration_str(total_hours));
        self.main_window
            .get_titlebar()
            .unwrap()
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
}
