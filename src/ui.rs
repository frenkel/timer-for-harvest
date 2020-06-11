use crate::app;
use gio::prelude::*;
use gtk::prelude::*;
use std::sync::mpsc;

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

pub enum Signal {}

pub struct Ui {
    from_app: glib::Receiver<Signal>,
    to_app: mpsc::Sender<app::Signal>,
    application: gtk::Application,
}

impl Ui {
    pub fn new(from_app: glib::Receiver<Signal>, to_app: mpsc::Sender<app::Signal>) -> Ui {
        let application = gtk::Application::new(
            Some("nl.frankgroeneveld.timer-for-harvest"),
            Default::default(),
        )
        .unwrap();
        Ui {
            from_app: from_app,
            to_app: to_app,
            application: application,
        }
    }

    pub fn run(ui: Ui) {
        let to_app = ui.to_app.clone();
        ui.application.connect_activate(move |application| {
            let window = Ui::main_window(application, &to_app);
        });

        ui.from_app.attach(None, move |event| glib::Continue(true));
        ui.application.run(&[]);
    }

    pub fn main_window(
        application: &gtk::Application,
        to_app: &mpsc::Sender<app::Signal>,
    ) -> gtk::ApplicationWindow {
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
        window.connect_key_press_event(clone!(to_app => move |_window, event| {
            if event.get_keyval() == gdk::enums::key::F5 {
                to_app.send(app::Signal::RetrieveTimeEntries)
                    .expect("Sending message to application thread");
                Inhibit(true)
            } else if event.get_keyval() == gdk::enums::key::n {
                to_app.send(app::Signal::OpenPopup)
                    .expect("Sending message to application thread");
                Inhibit(true)
            } else {
                Inhibit(false)
            }
        }));

        let button =
            gtk::Button::new_from_icon_name(Some("list-add-symbolic"), gtk::IconSize::Button);
        button.set_sensitive(false);
        container.pack_start(&button);
        button.connect_clicked(clone!(to_app => move |_button| {
            to_app.send(app::Signal::OpenPopup)
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
        container.pack_start(&hbox);
        next_button.connect_clicked(clone!(to_app => move |_button| {
            to_app.send(app::Signal::NextDate)
                .expect("Sending message to application thread");
        }));

        window.show_all();

        window
    }
}
