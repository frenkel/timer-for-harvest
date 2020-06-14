use gtk::prelude::*;

pub struct Popup {
}

impl Popup {
    pub fn new(application: &gtk::Application) -> Popup {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);

        window.set_title("Add time entry");
        window.set_default_size(400, 200);
        window.set_modal(true);
        window.set_type_hint(gdk::WindowTypeHint::Dialog);
        window.set_border_width(10);

        window.connect_delete_event(|_, _| Inhibit(false));
        window.add_events(gdk::EventMask::KEY_PRESS_MASK);
        window.connect_key_press_event(|window, event| {
            if event.get_keyval() == gdk::enums::key::Escape {
                window.close();
                Inhibit(true)
            } else {
                Inhibit(false)
            }
        });

        window.set_transient_for(Some(&application.get_active_window().unwrap()));
        application.add_window(&window);

        window.show_all();

        Popup { }
    }
}
