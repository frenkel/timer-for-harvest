use crate::background;
use crate::popup::Popup;

use gio::prelude::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::env::args;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use timer_for_harvest::*;

pub enum Event {
    RetrieveProjectAssignments,
    RetrieveTimeEntries(String),
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
    prev_button: gtk::Button,
    next_button: gtk::Button,
    time_entries: Rc<RefCell<Vec<TimeEntryRow>>>,
    pub project_assignments: Rc<RefCell<Vec<ProjectAssignment>>>,
    for_date: Rc<RefCell<chrono::NaiveDate>>,
    total_amount_label: gtk::Label,
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
                .send(Event::RetrieveTimeEntries(ui.for_date.borrow().to_string()))
                .expect("Sending message to background thread");
        }
        background::Event::TimerStopped => {
            println!("Timer stopped");
            to_background
                .send(Event::RetrieveTimeEntries(ui.for_date.borrow().to_string()))
                .expect("Sending message to background thread");
        }
        background::Event::TimerRestarted => {
            println!("Timer restarted");
            to_background
                .send(Event::RetrieveTimeEntries(ui.for_date.borrow().to_string()))
                .expect("Sending message to background thread");
        }
        background::Event::TimerUpdated => {
            println!("Timer updated");
            to_background
                .send(Event::RetrieveTimeEntries(ui.for_date.borrow().to_string()))
                .expect("Sending message to background thread");
        }
        background::Event::TimerDeleted => {
            println!("Timer deleted");
            to_background
                .send(Event::RetrieveTimeEntries(ui.for_date.borrow().to_string()))
                .expect("Sending message to background thread");
        }
        background::Event::Loading(id) => {
            println!("Loading");
            ui.main_window
                .get_titlebar()
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
        window.set_border_width(18);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(500, 300);
        window.set_size_request(500, 300);

        window.add_events(gdk::EventMask::KEY_PRESS_MASK);

        let button =
            gtk::Button::new_from_icon_name(Some("list-add-symbolic"), gtk::IconSize::Button);
        button.set_sensitive(false);
        container.pack_start(&button);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        hbox.set_spacing(0);
        hbox.get_style_context().add_class(&gtk::STYLE_CLASS_LINKED);
        let prev_button =
            gtk::Button::new_from_icon_name(Some("go-previous-symbolic"), gtk::IconSize::Button);
        hbox.pack_start(&prev_button, false, false, 0);

        let next_button =
            gtk::Button::new_from_icon_name(Some("go-next-symbolic"), gtk::IconSize::Button);
        hbox.pack_start(&next_button, false, false, 0);
        container.pack_start(&hbox);

        to_background
            .send(Event::RetrieveProjectAssignments)
            .expect("Sending message to background thread");
        let now = chrono::Local::today().naive_local();
        to_background
            .send(Event::RetrieveTimeEntries(now.to_string()))
            .expect("Sending message to background thread");

        let amount_label = left_aligned_label(&"");

        Ui {
            main_window: window,
            to_background: to_background,
            start_button: button,
            prev_button: prev_button,
            next_button: next_button,
            time_entries: Rc::new(RefCell::new(vec![])),
            project_assignments: Rc::new(RefCell::new(vec![])),
            for_date: Rc::new(RefCell::new(now)),
            total_amount_label: amount_label,
        }
    }

    pub fn connect_main_window_signals(ui: &Rc<Ui>) {
        let to_background = ui.to_background.clone();
        let prev_to_background = ui.to_background.clone();
        let next_to_background = ui.to_background.clone();
        let key_press_event_ui_ref = Rc::clone(&ui);
        let start_button_ui_ref = Rc::clone(&ui);
        let next_button_ui_ref = Rc::clone(&ui);
        let prev_button_ui_ref = Rc::clone(&ui);
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
                        .send(Event::RetrieveTimeEntries(
                            key_press_event_ui_ref.for_date.borrow().to_string(),
                        ))
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
            open_popup(&start_button_ui_ref);
        });

        ui.prev_button.connect_clicked(move |_| {
            let new_date = { prev_button_ui_ref.for_date.borrow().pred() };
            prev_button_ui_ref.for_date.replace(new_date);
            prev_to_background
                .send(Event::RetrieveTimeEntries(
                    prev_button_ui_ref.for_date.borrow().to_string(),
                ))
                .expect("Sending message to background thread");
        });

        ui.next_button.connect_clicked(move |_| {
            let new_date = { next_button_ui_ref.for_date.borrow().succ() };
            next_button_ui_ref.for_date.replace(new_date);
            next_to_background
                .send(Event::RetrieveTimeEntries(
                    next_button_ui_ref.for_date.borrow().to_string(),
                ))
                .expect("Sending message to background thread");
        });
    }

    pub fn connect_time_entry_signals(ui: &Rc<Ui>) {
        for time_entry_row in ui.time_entries.borrow().iter() {
            if time_entry_row.time_entry.borrow().is_running {
                let time_entries_ref = Rc::clone(&ui.time_entries);
                let time_entry_ref = Rc::clone(&time_entry_row.time_entry);
                let hours_label_ref = time_entry_row.hours_label.clone();
                let total_amount_label = ui.total_amount_label.clone();

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
                        Ui::update_total_amount(&total_amount_label, total);
                        glib::Continue(true)
                    } else {
                        glib::Continue(false)
                    }
                });
            }

            let to_background_clone = ui.to_background.clone();
            let is_running = time_entry_row.time_entry.borrow().is_running;
            let id = time_entry_row.time_entry.borrow().id;
            time_entry_row
                .start_stop_button
                .connect_clicked(move |_button| {
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
        let total_entries = time_entries.len() as i32;
        let mut row_number = total_entries;
        let grid = gtk::Grid::new();
        grid.set_column_spacing(12);
        grid.set_row_spacing(18);

        /* stop all running gtk timers */
        for old_entry in self.time_entries.borrow().iter() {
            old_entry.time_entry.borrow_mut().is_running = false;
        }
        /* clear old entries */
        self.time_entries.borrow_mut().clear();

        for time_entry in time_entries {
            total_hours += time_entry.hours;

            let notes = match time_entry.notes.as_ref() {
                Some(n) => n
                    .replace("&", "&amp;")
                    .replace("<", "&lt;")
                    .replace(">", "&gt;"),
                None => "".to_string(),
            };
            let project_client = format!(
                "<b>{}</b> ({})\n{} - {}",
                &time_entry.project.name_and_code(),
                &time_entry.client.name,
                &time_entry.task.name,
                &notes
            );
            let project_label = left_aligned_label(&project_client);
            project_label.set_line_wrap(true);
            project_label.set_use_markup(true);
            project_label.set_hexpand(true);
            grid.attach(&project_label, 0, row_number, 1, 1);

            let hours_label = left_aligned_label(&f32_to_duration_str(time_entry.hours));
            grid.attach(&hours_label, 1, row_number, 1, 1);

            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
            hbox.set_spacing(0);
            hbox.get_style_context().add_class(&gtk::STYLE_CLASS_LINKED);

            let button: gtk::Button;
            let rc = Rc::new(RefCell::new(time_entry));
            let time_entry_clone = Rc::clone(&rc);
            if time_entry_clone.borrow().is_running {
                button = gtk::Button::new_from_icon_name(
                    Some("media-playback-stop-symbolic"),
                    gtk::IconSize::Button,
                );
                button
                    .get_style_context()
                    .add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
            } else {
                button = gtk::Button::new_from_icon_name(
                    Some("media-playback-start-symbolic"),
                    gtk::IconSize::Button,
                );
            };
            button.set_valign(gtk::Align::Center);
            hbox.pack_start(&button, false, false, 0);

            let edit_button = gtk::Button::new_from_icon_name(
                Some("document-edit-symbolic"),
                gtk::IconSize::Button,
            );
            edit_button.set_valign(gtk::Align::Center);
            hbox.pack_start(&edit_button, false, false, 0);

            grid.attach(&hbox, 2, row_number, 1, 1);

            row_number -= 1;

            self.time_entries.borrow_mut().push(TimeEntryRow {
                time_entry: rc,
                start_stop_button: button,
                edit_button: edit_button,
                hours_label: hours_label,
            });
        }

        let total_label = left_aligned_label(&"<b>Total</b>");
        total_label.set_use_markup(true);
        grid.attach(&total_label, 0, total_entries + 1, 1, 1);
        Ui::update_total_amount(&self.total_amount_label, total_hours);

        let title = format!("Harvest - {}", self.for_date.borrow().format("%a %-d %b"));
        self.main_window
            .get_titlebar()
            .unwrap()
            .downcast::<gtk::HeaderBar>()
            .unwrap()
            .set_title(Some(&title));

        match self.main_window.get_children().first() {
            Some(child) => {
                if child.is::<gtk::Grid>() {
                    self.main_window.remove(child);
                }
            }
            None => {}
        }

        /* re-use amount label */
        grid.attach(&self.total_amount_label, 1, total_entries + 1, 1, 1);

        self.main_window.add(&grid);
        self.main_window.show_all();
    }

    fn update_total_amount(total_amount_label: &gtk::Label, total: f32) {
        total_amount_label.set_text(&format!("<b>{}</b>", f32_to_duration_str(total)));
        total_amount_label.set_use_markup(true);
    }
}
