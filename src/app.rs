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
    from_ui: mpsc::Receiver<Signal>,
    to_ui: glib::Sender<ui::Signal>,
}

impl App {
    pub fn new(from_ui: mpsc::Receiver<Signal>, to_ui: glib::Sender<ui::Signal>) -> App {
        App {
            from_ui: from_ui,
            to_ui: to_ui,
        }
    }

    pub fn run(app: App) {
        thread::spawn(move || {
            for signal in app.from_ui {
                match signal {
                    Signal::RetrieveTimeEntries => {},
                    Signal::OpenPopup => {},
                    Signal::PrevDate => {},
                    Signal::NextDate => {},
                }
            }
        });
    }
}
