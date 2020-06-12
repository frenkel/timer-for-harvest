use crate::ui;
use std::sync::mpsc;
use std::thread;
use timer_for_harvest::*;

pub enum Signal {
    RetrieveTimeEntries,
    OpenPopup,
    PrevDate,
    NextDate,
}

pub struct App {
    to_ui: glib::Sender<ui::Signal>,
    shown_date: chrono::NaiveDate,
    api: Harvest,
    user: User,
}

impl App {
    pub fn new(to_ui: glib::Sender<ui::Signal>) -> App {
        let now = chrono::Local::today().naive_local();
        let api = Harvest::new();
        let user = api.current_user();

        App {
            to_ui: to_ui,
            shown_date: now,
            api: api,
            user: user,
        }
    }

    pub fn handle_ui_signals(mut app: App, from_ui: mpsc::Receiver<Signal>) {
        thread::spawn(move || {
            for signal in from_ui {
                match signal {
                    Signal::RetrieveTimeEntries => {
                        app.to_ui.send(ui::Signal::SetTitle("Loading...".to_string()))
                            .expect("Sending message to ui thread");
                        app.api.time_entries_for(
                            &app.user,
                            app.shown_date.to_string(),
                            app.shown_date.to_string(),
                        );
                    },
                    Signal::OpenPopup => {},
                    Signal::PrevDate => {
                        app.shown_date = app.shown_date.pred();
                        app.format_and_send_title();
                    },
                    Signal::NextDate => {
                        app.shown_date = app.shown_date.succ();
                        app.format_and_send_title();
                    },
                }
            }
        });
    }

    fn format_and_send_title(&self) {
        let title = format!("Harvest - {}", self.shown_date.format("%a %-d %b"));
        self.to_ui.send(ui::Signal::SetTitle(title))
            .expect("Sending message to ui thread");
    }
}
