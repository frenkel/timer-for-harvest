use crate::ui;
use std::sync::mpsc;
use std::thread;
use timer_for_harvest::*;

pub enum Signal {
    RetrieveTimeEntries,
    NewTimeEntry,
    EditTimeEntry(u32),
    RestartTimeEntry(u32),
    StopTimeEntry(u32),
    DeleteTimeEntry(u32),
    PrevDate,
    NextDate,
    LoadTasksForProject(u32),
    StartTimer(u32, u32, String, f32),
}

pub struct App {
    to_ui: glib::Sender<ui::Signal>,
    shown_date: chrono::NaiveDate,
    api: Harvest,
    user: User,
    project_assignments: Vec<ProjectAssignment>,
    time_entries: Vec<TimeEntry>,
}

impl App {
    pub fn new(to_ui: glib::Sender<ui::Signal>) -> App {
        let now = chrono::Local::today().naive_local();
        let api = Harvest::new();
        let user = api.current_user();
        let mut project_assignments = api.active_project_assignments();
        project_assignments.sort_by(|a, b| {
            a.project
                .name
                .to_lowercase()
                .cmp(&b.project.name.to_lowercase())
        });

        App {
            to_ui: to_ui,
            shown_date: now,
            api: api,
            user: user,
            project_assignments: project_assignments,
            time_entries: vec![],
        }
    }

    pub fn handle_ui_signals(mut app: App, from_ui: mpsc::Receiver<Signal>) {
        thread::spawn(move || {
            for signal in from_ui {
                match signal {
                    Signal::RetrieveTimeEntries => {
                        app.retrieve_time_entries();
                    }
                    Signal::NewTimeEntry => {
                        app.to_ui
                            .send(ui::Signal::OpenPopup(app.project_assignments.to_vec()))
                            .expect("Sending message to ui thread");
                    }
                    Signal::EditTimeEntry(id) => {
                        app.edit_time_entry(id);
                    }
                    Signal::RestartTimeEntry(id) => {
                        app.restart_timer(id);
                        app.retrieve_time_entries();
                    }
                    Signal::StopTimeEntry(id) => {
                        app.stop_timer(id);
                        app.retrieve_time_entries();
                    }
                    Signal::DeleteTimeEntry(id) => {
                        app.api.delete_timer(id);
                        app.retrieve_time_entries();
                    }
                    Signal::PrevDate => {
                        app.shown_date = app.shown_date.pred();
                        app.retrieve_time_entries();
                    }
                    Signal::NextDate => {
                        app.shown_date = app.shown_date.succ();
                        app.retrieve_time_entries();
                    }
                    Signal::LoadTasksForProject(id) => {
                        app.retrieve_tasks_for_project(id);
                    }
                    Signal::StartTimer(project_id, task_id, notes, hours) => {
                        app.start_timer(project_id, task_id, &notes, hours);
                    }
                }
            }
        });
    }

    fn format_and_send_title(&self) {
        let title = format!("Harvest - {}", self.shown_date.format("%a %-d %b"));
        self.to_ui
            .send(ui::Signal::SetTitle(title))
            .expect("Sending message to ui thread");
    }

    fn retrieve_time_entries(&mut self) {
        self.to_ui
            .send(ui::Signal::SetTitle("Loading...".to_string()))
            .expect("Sending message to ui thread");
        self.time_entries = self.api.time_entries_for(
            &self.user,
            self.shown_date.to_string(),
            self.shown_date.to_string(),
        );

        self.to_ui
            .send(ui::Signal::SetTimeEntries(self.time_entries.clone()))
            .expect("Sending message to ui thread");
        self.format_and_send_title();
    }

    fn restart_timer(&self, id: u32) {
        self.to_ui
            .send(ui::Signal::SetTitle("Loading...".to_string()))
            .expect("Sending message to ui thread");
        self.api.restart_timer(id);
    }

    fn stop_timer(&self, id: u32) {
        self.to_ui
            .send(ui::Signal::SetTitle("Loading...".to_string()))
            .expect("Sending message to ui thread");
        self.api.stop_timer(id);
    }

    fn retrieve_tasks_for_project(&self, id: u32) {
        for project_assignment in &self.project_assignments {
            if id == project_assignment.project.id {
                self.to_ui
                    .send(ui::Signal::TaskAssignments(
                        project_assignment.task_assignments.clone(),
                    ))
                    .expect("Sending message to ui thread");
                break;
            }
        }
    }

    fn start_timer(&self, project_id: u32, task_id: u32, notes: &String, hours: f32) {
        self.api.start_timer(project_id, task_id, notes, hours);
    }

    fn edit_time_entry(&self, id: u32) {
        for time_entry in self.time_entries.clone() {
            if time_entry.id == id {
                self.to_ui
                    .send(ui::Signal::OpenPopupWithTimeEntry(
                        self.project_assignments.to_vec(),
                        time_entry,
                    ))
                    .expect("Sending message to ui thread");
            }
        }
    }
}
