use crate::app;
use gtk::prelude::*;
use std::sync::mpsc;
use timer_for_harvest::*;

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
    project_chooser: gtk::ComboBox,
    pub task_chooser: gtk::ComboBox,
    to_app: mpsc::Sender<app::Signal>,
    delete_button: gtk::Button,
    save_button: gtk::Button,
    notes_input: gtk::Entry,
    hours_input: gtk::Entry,
}

impl Popup {
    pub fn new(
        application: &gtk::Application,
        project_assignments: Vec<ProjectAssignment>,
        to_app: mpsc::Sender<app::Signal>,
    ) -> Popup {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);

        window.set_title("Add time entry");
        window.set_default_size(400, 200);
        window.set_modal(true);
        window.set_type_hint(gdk::WindowTypeHint::Dialog);
        window.set_border_width(18);

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

        let delete_button = gtk::Button::new_with_label("Delete");
        let save_button = gtk::Button::new_with_label("Start Timer");
        save_button.set_can_default(true);
        let notes_input = gtk::Entry::new();
        notes_input
            .set_property("activates-default", &true)
            .expect("could not allow default activation");
        let hours_input = gtk::Entry::new();
        hours_input
            .set_property("activates-default", &true)
            .expect("could not allow default activation");

        let popup = Popup {
            window: window,
            project_chooser: Popup::project_chooser(project_assignments),
            task_chooser: Popup::task_chooser(),
            to_app: to_app,
            delete_button: delete_button,
            save_button: save_button,
            notes_input: notes_input,
            hours_input: hours_input,
        };
        popup.add_widgets();
        popup.connect_signals();
        popup
    }

    fn project_chooser(project_assignments: Vec<ProjectAssignment>) -> gtk::ComboBox {
        let project_store = gtk::ListStore::new(&[gtk::Type::String, gtk::Type::U32]);
        for project_assignment in project_assignments {
            project_store.set(
                &project_store.append(),
                &[0, 1],
                &[
                    &project_assignment.project.name_and_code(),
                    &project_assignment.project.id,
                ],
            );
        }
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

        project_chooser
    }

    fn task_chooser() -> gtk::ComboBox {
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
        task_chooser
    }

    fn add_widgets(&self) {
        let grid = gtk::Grid::new();
        grid.set_column_spacing(12);
        grid.set_row_spacing(18);

        self.window.add(&grid);

        let to_app = self.to_app.clone();
        self.project_chooser
            .connect_changed(move |project_chooser| match project_chooser.get_active() {
                Some(index) => {
                    let project_id = Popup::id_from_combo_box(&project_chooser, index);
                    to_app
                        .send(app::Signal::LoadTasksForProject(project_id))
                        .expect("Sending message to application thread");
                }
                None => {}
            });
        grid.attach(&self.project_chooser, 0, 0, 4, 1);

        grid.attach(&self.task_chooser, 0, 1, 4, 1);

        grid.attach(&self.notes_input, 0, 2, 2, 1);

        grid.attach(&self.hours_input, 2, 2, 2, 1);

        self.delete_button.set_sensitive(false);
        grid.attach(&self.delete_button, 0, 3, 2, 1);

        grid.attach(&self.save_button, 2, 3, 2, 1);

        grid.show_all();
    }

    fn connect_signals(&self) {
        let to_app = self.to_app.clone();
        let project_chooser = self.project_chooser.clone();
        let task_chooser = self.task_chooser.clone();
        let window = self.window.clone();
        let notes_input = self.notes_input.clone();
        let hours_input = self.hours_input.clone();
        self.save_button.connect_clicked(move |button| {
            button.set_sensitive(false);
            let project_id = match project_chooser.get_active() {
                Some(index) => Popup::id_from_combo_box(&project_chooser, index),
                None => 0,
            };
            let task_id = match task_chooser.get_active() {
                Some(index) => Popup::id_from_combo_box(&task_chooser, index),
                None => 0,
            };
            if project_id > 0 && task_id > 0 {
                to_app
                    .send(app::Signal::StartTimer(
                        project_id,
                        task_id,
                        notes_input.get_text().unwrap().to_string(),
                        duration_str_to_f32(&hours_input.get_text().unwrap()),
                    ))
                    .expect("Sending message to background thread");
                to_app
                    .send(app::Signal::RetrieveTimeEntries)
                    .expect("Sending message to background thread");
                window.close();
            } else {
                button.set_sensitive(true);
            }
        });
    }

    pub fn populate(&self, task_assignments: Vec<TaskAssignment>, time_entry: TimeEntry) {
        self.save_button.set_label("Save Timer");
        self.hours_input.set_editable(!time_entry.is_running);
        self.project_chooser.set_active_iter(Some(
            &Popup::iter_from_id(&self.project_chooser, time_entry.project.id).unwrap(),
        ));
        let task_store = self
            .task_chooser
            .get_model()
            .unwrap()
            .downcast::<gtk::ListStore>()
            .unwrap();
        task_store.clear();
        for task_assignment in task_assignments {
            task_store.set(
                &task_store.append(),
                &[0, 1],
                &[&task_assignment.task.name, &task_assignment.task.id],
            );
        }
        /*self.task_chooser.set_active_iter(Some(
            &Popup::iter_from_id(&self.task_chooser, time_entry.task.id).unwrap(),
        ));*/
        self.delete_button.set_sensitive(true);
        let to_app = self.to_app.clone();
        let window = self.window.clone();
        self.delete_button.connect_clicked(move |button| {
            button.set_sensitive(false);

            to_app.send(app::Signal::DeleteTimeEntry(time_entry.id))
            .expect("Sending message to application thread");
            window.close();
        });
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

    fn id_from_combo_box(combo_box: &gtk::ComboBox, index: u32) -> u32 {
        let model = combo_box.get_model().unwrap();

        let iter = model.get_iter_from_string(&format!("{}", index)).unwrap();
        model.get_value(&iter, 1).get::<u32>().unwrap()
    }

    fn iter_from_id(combo_box: &gtk::ComboBox, id: u32) -> Option<gtk::TreeIter> {
        let store = combo_box.get_model().unwrap();
        let iter = store.get_iter_first().unwrap();
        loop {
            if store.get_value(&iter, 1).get::<u32>().unwrap() == id {
                return Some(iter);
            }
            if !store.iter_next(&iter) {
                break;
            }
        }
        None
    }
}
