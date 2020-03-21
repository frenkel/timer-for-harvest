use crate::background;
use crate::popup::Popup;

use gio::prelude::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::convert::TryInto;
use std::env::args;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use timer_for_harvest::*;

pub enum Event {
    RetrieveProjectAssignments,
    RetrieveTimeEntries,
    StartTimer(u32, u32, String, f32),
    StopTimer(u32),
    RestartTimer(u32),
    UpdateTimer(u32, u32, u32, String, f32, bool, String),
    DeleteTimer(u32),
}

pub struct Ui {
    main_window: gtk::ApplicationWindow,
    to_background: mpsc::Sender<Event>,
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

pub fn main_window() {
    let application = gtk::Application::new(
        Some("nl.frankgroeneveld.timer-for-harvest"),
        Default::default(),
    )
    .unwrap();

    application.connect_activate(move |app| {
        let (to_foreground, from_background) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let (to_background, from_foreground) = mpsc::channel();
        let ui_to_background = to_background.clone();

        let ui = Rc::new(Ui::new(ui_to_background, app));
        Ui::connect_main_window_signals(&ui);
        ui.main_window.show_all();

        thread::spawn(move || {
            let api = Harvest::new();
            for event in from_foreground {
                background::handle_event(&api, &to_foreground, event);
            }
        });

        from_background.attach(None, move |event| {
            handle_event(&ui, &to_background, event);
            glib::Continue(true)
        });
    });

    application.run(&args().collect::<Vec<_>>());
}

fn handle_event(ui: &Rc<Ui>, to_background: &mpsc::Sender<Event>, event: background::Event) {
    match event {
        background::Event::RetrievedProjectAssignments(project_assignments) => {
            println!("Processing project assignments");
            for project_assignment in project_assignments {
                ui.project_assignments.borrow_mut().push(project_assignment);
            }
            ui.start_button.set_sensitive(true);
        }
        background::Event::RetrievedTimeEntries(time_entries) => {
            println!("Processing time entries");
            ui.load_time_entries(time_entries);
            Ui::connect_time_entry_signals(&ui);
        }
        background::Event::TimerStarted => {
            println!("Timer started");
            to_background
                .send(Event::RetrieveTimeEntries)
                .expect("Sending message to background thread");
        }
        background::Event::TimerStopped => {
            println!("Timer stopped");
            to_background
                .send(Event::RetrieveTimeEntries)
                .expect("Sending message to background thread");
        }
        background::Event::TimerRestarted => {
            println!("Timer restarted");
            to_background
                .send(Event::RetrieveTimeEntries)
                .expect("Sending message to background thread");
        }
        background::Event::TimerUpdated => {
            println!("Timer updated");
            to_background
                .send(Event::RetrieveTimeEntries)
                .expect("Sending message to background thread");
        }
        background::Event::TimerDeleted => {
            println!("Timer deleted");
            to_background
                .send(Event::RetrieveTimeEntries)
                .expect("Sending message to background thread");
        }
        background::Event::Loading(id) => {
            println!("Loading");
            ui.main_window.get_titlebar()
                .unwrap()
                .downcast::<gtk::HeaderBar>()
                .unwrap()
                .set_title(Some("Loading..."));

            match id {
                Some(id) => {
                    for row in ui.time_entries.borrow().iter() {
                        if id == row.time_entry.borrow().id {
                            row.start_stop_button.set_sensitive(false);
                            row.edit_button.set_sensitive(false);
                        }
                    }
                }
                None => {}
            }
        }
    }
}

impl Ui {
    pub fn new(to_background: mpsc::Sender<Event>, application: &gtk::Application) -> Ui {
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
        button.set_sensitive(false);
        container.pack_start(&button);

        to_background
            .send(Event::RetrieveProjectAssignments)
            .expect("Sending message to background thread");
        to_background
            .send(Event::RetrieveTimeEntries)
            .expect("Sending message to background thread");

        Ui {
            main_window: window,
            to_background: to_background,
            start_button: button,
            time_entries: Rc::new(RefCell::new(vec![])),
            project_assignments: Rc::new(RefCell::new(vec![])),
        }
    }

    pub fn connect_main_window_signals(ui: &Rc<Ui>) {
        let to_background = ui.to_background.clone();
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
                ui_ref.to_background.clone(),
            );
            Popup::connect_signals(&Rc::new(popup), &ui_ref);
        };

        ui.main_window
            .connect_key_press_event(move |_window, event| {
                if event.get_keyval() == gdk::enums::key::F5 {
                    to_background
                        .send(Event::RetrieveTimeEntries)
                        .expect("Sending message to background thread");
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

            let to_background_clone = ui.to_background.clone();
            let is_running = time_entry_row.time_entry.borrow().is_running;
            let id = time_entry_row.time_entry.borrow().id;
            time_entry_row.start_stop_button.connect_clicked(move |button| {
                if is_running {
                    to_background_clone
                        .send(Event::StopTimer(id))
                        .expect("Sending message to background thread");
                } else {
                    to_background_clone
                        .send(Event::RestartTimer(id))
                        .expect("Sending message to background thread");
                }
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
                    ui_ref2.to_background.clone(),
                );
                Popup::connect_signals(&Rc::new(popup), &ui_ref2);
            });
        }
    }

    pub fn load_time_entries(&self, time_entries: Vec<TimeEntry>) {
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
