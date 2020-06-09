use crate::app;
use gio::prelude::*;
use gtk::prelude::*;
use std::sync::mpsc;

pub enum Signal {}

pub struct Ui {
    from_app: glib::Receiver<Signal>,
    to_app: mpsc::Sender<app::Signal>,
}

impl Ui {
    pub fn new(from_app: glib::Receiver<Signal>, to_app: mpsc::Sender<app::Signal>) -> Ui {
        Ui {
            from_app: from_app,
            to_app: to_app,
        }
    }

    pub fn run(ui: Ui) {
        println!("Ui started");
        let application = gtk::Application::new(
            Some("nl.frankgroeneveld.timer-for-harvest"),
            Default::default(),
        )
        .unwrap();
        ui.from_app.attach(None, move |event| glib::Continue(true));
        application.connect_activate(move |app| {
            let window = gtk::ApplicationWindow::new(app);
            window.show_all();
        });

        application.run(&[]);
    }
}
