use crate::app;
use gio::prelude::*;
use gtk::prelude::*;
use std::sync::mpsc;
use timer_for_harvest::*;
use crate::popup::Popup;

/* handy gtk callback clone macro taken from https://gtk-rs.org/docs-src/tutorial/closures */
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

pub enum Signal {
    SetTitle(String),
    SetTimeEntries(Vec<TimeEntry>),
    OpenPopup,
}

pub struct Ui {
    application: gtk::Application,
    header_bar: gtk::HeaderBar,
    grid: gtk::Grid,
    to_app: mpsc::Sender<app::Signal>,
}

impl Ui {
    pub fn new(to_app: mpsc::Sender<app::Signal>) -> Ui {
        let application = gtk::Application::new(
            Some("nl.frankgroeneveld.timer-for-harvest"),
            Default::default(),
        )
        .unwrap();
        let header_bar = gtk::HeaderBar::new();
        let grid = gtk::Grid::new();
        grid.set_column_spacing(12);
        grid.set_row_spacing(18);

        application.connect_activate(clone!(to_app, header_bar, grid => move |app| {
            Ui::main_window(app, &to_app, &header_bar, &grid);
        }));

        to_app.send(app::Signal::RetrieveTimeEntries)
            .expect("Sending message to application thread");

        Ui {
            application: application,
            header_bar: header_bar,
            grid: grid,
            to_app: to_app,
        }
    }

    pub fn handle_app_signals(ui: Ui, from_app: glib::Receiver<Signal>) {
        let application = ui.application.clone();
        from_app.attach(None, move |signal| {
            match signal {
                Signal::SetTitle(value) => {
                    ui.header_bar.set_title(Some(&value));
                },
                Signal::SetTimeEntries(time_entries) => {
                    ui.set_time_entries(time_entries);
                },
                Signal::OpenPopup => {
                    ui.open_popup();
                }
            }
            glib::Continue(true)
        });
        application.run(&[]);
    }

    pub fn main_window(
        application: &gtk::Application,
        to_app: &mpsc::Sender<app::Signal>,
        header_bar: &gtk::HeaderBar,
        grid: &gtk::Grid,
    ) -> gtk::ApplicationWindow {
        let window = gtk::ApplicationWindow::new(application);

        header_bar.set_title(Some("Harvest"));
        header_bar.set_show_close_button(true);

        window.set_title("Harvest");
        window.set_titlebar(Some(header_bar));
        window.set_border_width(18);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(500, 300);
        window.set_size_request(500, 300);

        window.add_events(gdk::EventMask::KEY_PRESS_MASK);
        window.connect_key_press_event(clone!(to_app => move |_window, event| {
            if event.get_keyval() == gdk::enums::key::F5 {
                to_app.send(app::Signal::RetrieveTimeEntries)
                    .expect("Sending message to application thread");
                Inhibit(true)
            } else if event.get_keyval() == gdk::enums::key::n {
                to_app.send(app::Signal::NewTimeEntry)
                    .expect("Sending message to application thread");
                Inhibit(true)
            } else {
                Inhibit(false)
            }
        }));

        let button =
            gtk::Button::new_from_icon_name(Some("list-add-symbolic"), gtk::IconSize::Button);
        button.set_sensitive(false);
        header_bar.pack_start(&button);
        button.connect_clicked(clone!(to_app => move |_button| {
            to_app.send(app::Signal::NewTimeEntry)
                .expect("Sending message to application thread");
        }));

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        hbox.set_spacing(0);
        hbox.get_style_context().add_class(&gtk::STYLE_CLASS_LINKED);
        let prev_button =
            gtk::Button::new_from_icon_name(Some("go-previous-symbolic"), gtk::IconSize::Button);
        hbox.pack_start(&prev_button, false, false, 0);
        prev_button.connect_clicked(clone!(to_app => move |_button| {
            to_app.send(app::Signal::PrevDate)
                .expect("Sending message to application thread");
        }));

        let next_button =
            gtk::Button::new_from_icon_name(Some("go-next-symbolic"), gtk::IconSize::Button);
        hbox.pack_start(&next_button, false, false, 0);
        header_bar.pack_start(&hbox);
        next_button.connect_clicked(clone!(to_app => move |_button| {
            to_app.send(app::Signal::NextDate)
                .expect("Sending message to application thread");
        }));

        window.add(grid);
        window.show_all();

        window
    }

    pub fn set_time_entries(&self, time_entries: Vec<TimeEntry>) {
        let total_entries = time_entries.len() as i32;
        let mut total_hours = 0.0;
        let mut row_number = total_entries;

        for child in self.grid.get_children() {
            self.grid.remove(&child);
        }

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
            let project_label = gtk::Label::new(Some(&project_client));
            project_label.set_xalign(0.0);
            project_label.set_line_wrap(true);
            project_label.set_use_markup(true);
            project_label.set_hexpand(true);
            self.grid.attach(&project_label, 0, row_number, 1, 1);

            let hours_label = gtk::Label::new(Some(&f32_to_duration_str(time_entry.hours)));
            hours_label.set_xalign(0.0);
            self.grid.attach(&hours_label, 1, row_number, 1, 1);

            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
            hbox.set_spacing(0);
            hbox.get_style_context().add_class(&gtk::STYLE_CLASS_LINKED);

            let button: gtk::Button;
            let to_app = self.to_app.clone();
            let id = time_entry.id;
            if time_entry.is_running {
                button = gtk::Button::new_from_icon_name(
                    Some("media-playback-stop-symbolic"),
                    gtk::IconSize::Button,
                );
                button
                    .get_style_context()
                    .add_class(&gtk::STYLE_CLASS_SUGGESTED_ACTION);
                button.connect_clicked(move |button| {
                    button.set_sensitive(false);
                    to_app.send(app::Signal::StopTimeEntry(id))
                        .expect("Sending message to application thread");
                });
            } else {
                button = gtk::Button::new_from_icon_name(
                    Some("media-playback-start-symbolic"),
                    gtk::IconSize::Button,
                );
                button.connect_clicked(move |button| {
                    button.set_sensitive(false);
                    to_app.send(app::Signal::RestartTimeEntry(id))
                        .expect("Sending message to application thread");
                });
            };
            button.set_valign(gtk::Align::Center);
            hbox.pack_start(&button, false, false, 0);

            let edit_button = gtk::Button::new_from_icon_name(
                Some("document-edit-symbolic"),
                gtk::IconSize::Button,
            );
            edit_button.set_valign(gtk::Align::Center);
            let to_app = self.to_app.clone();
            let id = time_entry.id;
            edit_button.connect_clicked(move |_button| {
                to_app.send(app::Signal::EditTimeEntry(id))
                    .expect("Sending message to application thread");
            });
            hbox.pack_start(&edit_button, false, false, 0);

            self.grid.attach(&hbox, 2, row_number, 1, 1);

            row_number -= 1;
        }

        let total_label = gtk::Label::new(Some(&"<b>Total</b>"));
        total_label.set_xalign(0.0);
        total_label.set_use_markup(true);
        self.grid.attach(&total_label, 0, total_entries + 1, 1, 1);

        let formatted_label = format!("<b>{}</b>", f32_to_duration_str(total_hours));
        let total_amount_label = gtk::Label::new(Some(&formatted_label));
        total_amount_label.set_xalign(0.0);
        total_amount_label.set_use_markup(true);
        self.grid.attach(&total_amount_label, 1, total_entries + 1, 1, 1);

        self.grid.show_all();
    }
    
    fn open_popup(&self) {
        Popup::new(&self.application);
    }
}
