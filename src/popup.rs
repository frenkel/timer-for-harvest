use gtk::prelude::*;

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

pub struct Popup {
    window: gtk::Window,
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

        let popup = Popup { window: window };
        popup.fill_grid();
        popup
    }

    fn fill_grid(&self) {
        let grid = gtk::Grid::new();
        grid.set_column_spacing(12);
        grid.set_row_spacing(18);

        self.window.add(&grid);

        let project_store = gtk::ListStore::new(&[gtk::Type::String, gtk::Type::U32]);
        /*for project_assignment in project_assignments.borrow().iter() {
            project_store.set(
                &project_store.append(),
                &[0, 1],
                &[
                    &project_assignment.project.name_and_code(),
                    &project_assignment.project.id,
                ],
            );
        }*/
        let project_chooser = gtk::ComboBox::new_with_model_and_entry(&project_store);
        project_chooser.set_entry_text_column(0);

        let project_completer = gtk::EntryCompletion::new();
        project_completer.set_model(Some(&project_store));
        project_completer.set_text_column(0);
        project_completer.set_match_func(Popup::fuzzy_matching);
        project_completer.connect_match_selected(
            clone!(project_chooser => move |_completion, _model, iter| {
                project_chooser.set_active_iter(Some(&iter));
                Inhibit(false)
            }),
        );
        project_chooser
            .get_child()
            .unwrap()
            .downcast::<gtk::Entry>()
            .unwrap()
            .set_completion(Some(&project_completer));
        project_chooser.set_hexpand(true);
        grid.attach(&project_chooser, 0, 0, 4, 1);

        let task_store = gtk::ListStore::new(&[gtk::Type::String, gtk::Type::U32]);
        let task_chooser = gtk::ComboBox::new_with_model_and_entry(&task_store);
        task_chooser.set_entry_text_column(0);

        let task_completer = gtk::EntryCompletion::new();
        task_completer.set_model(Some(&task_store));
        task_completer.set_text_column(0);
        task_completer.set_match_func(Popup::fuzzy_matching);
        task_completer.connect_match_selected(
            clone!(task_chooser => move |_completion, _model, iter| {
                task_chooser.set_active_iter(Some(&iter));
                Inhibit(false)
            }),
        );

        task_chooser
            .get_child()
            .unwrap()
            .downcast::<gtk::Entry>()
            .unwrap()
            .set_completion(Some(&task_completer));
        grid.attach(&task_chooser, 0, 1, 4, 1);

        let notes_input = gtk::Entry::new();
        notes_input
            .set_property("activates-default", &true)
            .expect("could not allow default activation");
        grid.attach(&notes_input, 0, 2, 2, 1);

        let hour_input = gtk::Entry::new();
        hour_input
            .set_property("activates-default", &true)
            .expect("could not allow default activation");
        grid.attach(&hour_input, 2, 2, 2, 1);

        let delete_button = gtk::Button::new();
        delete_button.set_label("Delete");
        grid.attach(&delete_button, 0, 3, 2, 1);

        let save_button = gtk::Button::new();
        save_button.set_can_default(true);
        save_button.set_label("Start Timer");
        grid.attach(&save_button, 2, 3, 2, 1);

        grid.show_all();
    }

    fn fuzzy_matching(completion: &gtk::EntryCompletion, key: &str, iter: &gtk::TreeIter) -> bool {
        let store = completion.get_model().unwrap();
        let column_number = completion.get_text_column();
        let row = store
            .get_value(iter, column_number)
            .get::<String>()
            .unwrap();

        /* key is already lower case */
        if row.to_lowercase().contains(key) {
            true
        } else {
            false
        }
    }
}
