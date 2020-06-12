use crate::ui;
use std::sync::mpsc;
use std::thread;

pub enum Signal {
    RetrieveTimeEntries,
    OpenPopup,
    PrevDate,
    NextDate,
}

pub struct App {
    to_ui: glib::Sender<ui::Signal>,
}

impl App {
    pub fn new(to_ui: glib::Sender<ui::Signal>) -> App {
        App {
            to_ui: to_ui,
        }
    }

    pub fn handle_ui_signals(app: App, from_ui: mpsc::Receiver<Signal>) {
        thread::spawn(move || {
            for signal in from_ui {
                match signal {
                    Signal::RetrieveTimeEntries => {
                        app.to_ui.send(ui::Signal::SetTitle("Loading...".to_string()))
                            .expect("Sending message to ui thread");
                    },
                    Signal::OpenPopup => {},
                    Signal::PrevDate => {},
                    Signal::NextDate => {},
                }
            }
        });
    }
}
