use crate::ui::Ui;

use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use timer_for_harvest::*;

pub struct Popup {
    window: gtk::Window,
    save_button: gtk::Button,
    delete_button: gtk::Button,
    project_chooser: gtk::ComboBox,
    project_store: gtk::ListStore,
    task_chooser: gtk::ComboBox,
    task_store: gtk::ListStore,
    hour_input: gtk::Entry,
    notes_input: gtk::Entry,
    timer: Timer,
}

impl Popup {
    pub fn new(
        timer: Timer,
        project_assignments: Rc<RefCell<Vec<ProjectAssignment>>>,
        main_window: gtk::ApplicationWindow,
    ) -> Popup {
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

        let project_store = gtk::ListStore::new(&[gtk::Type::String, gtk::Type::U32]);
        for project_assignment in project_assignments.borrow().iter() {
            project_store.set(
                &project_store.append(),
                &[0, 1],
                &[
                    &project_assignment.project.name_and_code(),
                    &project_assignment.project.id,
                ],
            );
        }

        let data = gtk::Box::new(gtk::Orientation::Vertical, 5);

        let project_chooser = gtk::ComboBox::new_with_model_and_entry(&project_store);
        project_chooser.set_entry_text_column(0);

        let project_completer = gtk::EntryCompletion::new();
        project_completer.set_model(Some(&project_store));
        project_completer.set_text_column(0);
        project_completer.set_match_func(Popup::fuzzy_matching);
        let project_chooser_clone2 = project_chooser.clone();
        project_completer.connect_match_selected(move |_completion, _model, iter| {
            project_chooser_clone2.set_active_iter(Some(&iter));
            Inhibit(false)
        });

        project_chooser
            .get_child()
            .unwrap()
            .downcast::<gtk::Entry>()
            .unwrap()
            .set_completion(Some(&project_completer));
        data.pack_start(&project_chooser, true, false, 0);

        let task_store = gtk::ListStore::new(&[gtk::Type::String, gtk::Type::U32]);
        let task_chooser = gtk::ComboBox::new_with_model_and_entry(&task_store);
        task_chooser.set_entry_text_column(0);

        let task_completer = gtk::EntryCompletion::new();
        task_completer.set_model(Some(&task_store));
        task_completer.set_text_column(0);
        task_completer.set_match_func(Popup::fuzzy_matching);
        let task_chooser_clone2 = task_chooser.clone();
        task_completer.connect_match_selected(move |_completion, _model, iter| {
            task_chooser_clone2.set_active_iter(Some(&iter));
            Inhibit(false)
        });

        task_chooser
            .get_child()
            .unwrap()
            .downcast::<gtk::Entry>()
            .unwrap()
            .set_completion(Some(&task_completer));
        data.pack_start(&task_chooser, true, false, 0);

        if timer.project_id > 0 {
            /* TODO handle failure */
            project_chooser.set_active_iter(Some(
                &Popup::iter_from_id(&project_store, timer.project_id).unwrap(),
            ));
        }

        let inputs = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        let notes_input = gtk::Entry::new();
        notes_input
            .set_property("activates-default", &true)
            .expect("could not allow default activation");
        inputs.pack_start(&notes_input, true, true, 0);
        match &timer.notes {
            Some(n) => notes_input.set_text(&n),
            None => {}
        }

        let hour_input = gtk::Entry::new();
        hour_input
            .set_property("activates-default", &true)
            .expect("could not allow default activation");
        inputs.pack_start(&hour_input, false, false, 0);
        match timer.hours {
            Some(h) => hour_input.set_text(&f32_to_duration_str(h)),
            None => {}
        }
        hour_input.set_editable(!timer.is_running);

        data.pack_start(&inputs, true, false, 0);

        let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 2);

        let delete_button = gtk::Button::new();
        delete_button.set_label("Delete");
        delete_button.set_sensitive(timer.id != None);
        buttons.pack_start(&delete_button, true, false, 0);

        let save_button = gtk::Button::new();
        save_button.set_can_default(true);
        buttons.pack_end(&save_button, true, false, 0);

        /* TODO fix alignment of buttons, float left & right */
        data.pack_start(&buttons, true, false, 0);

        if timer.id == None {
            save_button.set_label("Start Timer");
        } else {
            save_button.set_label("Save Timer");
        }

        window.add(&data);
        save_button.grab_default();
        main_window.get_application().unwrap().add_window(&window);
        window.set_transient_for(Some(&main_window));
        window.show_all();
        Popup {
            window: window,
            save_button: save_button,
            delete_button: delete_button,
            project_chooser: project_chooser,
            project_store: project_store,
            task_chooser: task_chooser,
            task_store: task_store,
            hour_input: hour_input,
            notes_input: notes_input,
            timer: timer,
        }
    }
    pub fn connect_signals(popup: &Rc<Popup>, ui: &Rc<Ui>) {
        let popup_ref = Rc::clone(popup);
        let api_ref = Rc::clone(&ui.api);
        let project_assignments_ref = Rc::clone(&ui.project_assignments);
        popup
            .save_button
            .connect_clicked(move |_| match popup_ref.project_chooser.get_active() {
                Some(index) => match popup_ref.task_chooser.get_active() {
                    Some(task_index) => {
                        let iter = &popup_ref
                            .project_store
                            .get_iter_from_string(&format!("{}", index))
                            .unwrap();
                        let id = popup_ref
                            .project_store
                            .get_value(iter, 1)
                            .get::<u32>()
                            .unwrap();
                        let task = Popup::task_from_index(&popup_ref.task_store, task_index);

                        for project_assignment in project_assignments_ref.borrow().iter() {
                            if project_assignment.project.id == id {
                                if popup_ref.timer.id == None {
                                    api_ref.start_timer(
                                        &project_assignment.project,
                                        &task,
                                        &popup_ref.notes_input.get_text().unwrap(),
                                        duration_str_to_f32(
                                            &popup_ref.hour_input.get_text().unwrap(),
                                        ),
                                    );
                                } else {
                                    api_ref.update_timer(&Timer {
                                        id: popup_ref.timer.id,
                                        project_id: project_assignment.project.id,
                                        task_id: task.id,
                                        notes: Some(
                                            popup_ref.notes_input.get_text().unwrap().to_string(),
                                        ),
                                        hours: Some(duration_str_to_f32(
                                            &popup_ref.hour_input.get_text().unwrap(),
                                        )),
                                        is_running: popup_ref.timer.is_running,
                                        spent_date: Some(
                                            popup_ref
                                                .timer
                                                .spent_date
                                                .as_ref()
                                                .unwrap()
                                                .to_string(),
                                        ),
                                    });
                                }
                            }
                        }

                        popup_ref.window.close();
                    }
                    None => {}
                },
                None => {}
            });
        let popup_ref2 = Rc::clone(popup);
        let project_assignments_ref2 = Rc::clone(&ui.project_assignments);
        let load_task = move |project_chooser: &gtk::ComboBox| {
            popup_ref2.task_store.clear();
            match project_chooser.get_active() {
                Some(index) => {
                    let iter = &popup_ref2
                        .project_store
                        .get_iter_from_string(&format!("{}", index))
                        .unwrap();
                    let id = popup_ref2
                        .project_store
                        .get_value(iter, 1)
                        .get::<u32>()
                        .unwrap();

                    for project_assignment in project_assignments_ref2.borrow().iter() {
                        if project_assignment.project.id == id {
                            Popup::load_tasks(&popup_ref2.task_store, &project_assignment);
                            if popup_ref2.timer.task_id > 0 {
                                /* when project_id changes, we might not have a task in the dropdown */
                                popup_ref2.task_chooser.set_active_iter(
                                    Popup::iter_from_id(
                                        &popup_ref2.task_store,
                                        popup_ref2.timer.task_id,
                                    )
                                    .as_ref(),
                                );
                            }
                            break;
                        }
                    }
                }
                None => {}
            }
        };
        /* trigger loading of task */
        load_task(&popup.project_chooser);
        popup.project_chooser.connect_changed(load_task);

        let ui_ref = Rc::clone(&ui);
        popup.window.connect_delete_event(move |_, _| {
            ui_ref.load_time_entries();
            Ui::connect_time_entry_signals(&ui_ref);
            Inhibit(false)
        });

        let api_ref2 = Rc::clone(&ui.api);
        let popup_ref2 = Rc::clone(&popup);
        popup.delete_button.connect_clicked(move |_| {
            api_ref2.delete_timer(&popup_ref2.timer);
            popup_ref2.window.close();
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

    fn iter_from_id(store: &gtk::ListStore, id: u32) -> Option<gtk::TreeIter> {
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

    fn task_from_index(store: &gtk::ListStore, index: u32) -> Task {
        let iter = &store.get_iter_from_string(&format!("{}", index)).unwrap();
        let id = store.get_value(iter, 1).get::<u32>().unwrap();
        let name = store.get_value(iter, 0).get::<String>().unwrap();
        Task { id: id, name: name }
    }

    fn load_tasks(store: &gtk::ListStore, project_assignment: &ProjectAssignment) {
        for task_assignment in &project_assignment.task_assignments {
            store.set(
                &store.append(),
                &[0, 1],
                &[&task_assignment.task.name, &task_assignment.task.id],
            );
        }
    }
}
