use crate::ui;
use std::sync::mpsc;
use timer_for_harvest::*;

pub enum Event {
    RetrievedProjectAssignments(Vec<ProjectAssignment>),
    RetrievedTimeEntries(Vec<TimeEntry>),
    TimerStarted,
    TimerStopped,
    TimerRestarted,
    TimerUpdated,
    TimerDeleted,
    Loading(Option<u32>),
    OpenPopup(u32), /* actually sent from ui to itself */
    SetTitle(String),
}

pub fn handle_events(
    api: &Harvest,
    from_foreground: mpsc::Receiver<ui::Event>,
    to_foreground: &glib::Sender<Event>,
) {
    let mut for_date = chrono::Local::today().naive_local();

    for event in from_foreground {
        match event {
            ui::Event::RetrieveProjectAssignments => {
                to_foreground
                    .send(Event::Loading(None))
                    .expect("Sending message to foreground");

                println!("Retrieving project assignments");
                let mut project_assignments = api.active_project_assignments();
                project_assignments.sort_by(|a, b| {
                    a.project
                        .name
                        .to_lowercase()
                        .cmp(&b.project.name.to_lowercase())
                });
                to_foreground
                    .send(Event::RetrievedProjectAssignments(project_assignments))
                    .expect("Sending message to foreground");
            }
            ui::Event::RetrieveTimeEntries => {
                retrieve_time_entries(&to_foreground, &api, &for_date);
            }
            ui::Event::StartTimer(project_id, task_id, notes, hours) => {
                to_foreground
                    .send(Event::Loading(None))
                    .expect("Sending message to foreground");

                println!("Starting timer");
                api.start_timer(project_id, task_id, &notes, hours);
                to_foreground
                    .send(Event::TimerStarted)
                    .expect("Sending message to foreground");
            }
            ui::Event::StopTimer(id) => {
                to_foreground
                    .send(Event::Loading(Some(id)))
                    .expect("Sending message to foreground");

                println!("Stopping timer");
                api.stop_timer(id);
                to_foreground
                    .send(Event::TimerStopped)
                    .expect("Sending message to foreground");
            }
            ui::Event::RestartTimer(id) => {
                to_foreground
                    .send(Event::Loading(Some(id)))
                    .expect("Sending message to foreground");

                println!("Restarting timer");
                api.restart_timer(id);
                to_foreground
                    .send(Event::TimerRestarted)
                    .expect("Sending message to foreground");
            }
            ui::Event::UpdateTimer(
                id,
                project_id,
                task_id,
                notes,
                hours,
                is_running,
                spent_date,
            ) => {
                to_foreground
                    .send(Event::Loading(Some(id)))
                    .expect("Sending message to foreground");

                println!("Updating timer");
                api.update_timer(
                    id, project_id, task_id, notes, hours, is_running, spent_date,
                );
                to_foreground
                    .send(Event::TimerUpdated)
                    .expect("Sending message to foreground");
            }
            ui::Event::DeleteTimer(id) => {
                to_foreground
                    .send(Event::Loading(Some(id)))
                    .expect("Sending message to foreground");

                println!("Deleting timer");
                api.delete_timer(id);
                to_foreground
                    .send(Event::TimerDeleted)
                    .expect("Sending message to foreground");
            }
            ui::Event::PreviousDate => {
                for_date = for_date.pred();
                retrieve_time_entries(&to_foreground, &api, &for_date);
            }
            ui::Event::NextDate => {
                for_date = for_date.succ();
                retrieve_time_entries(&to_foreground, &api, &for_date);
            }
        }
    }
}

fn retrieve_time_entries(
    to_foreground: &glib::Sender<Event>,
    api: &Harvest,
    for_date: &chrono::NaiveDate,
) {
    to_foreground
        .send(Event::Loading(None))
        .expect("Sending message to foreground");

    println!("Retrieving time entries for {}", for_date);
    let user = api.current_user();
    to_foreground
        .send(Event::RetrievedTimeEntries(api.time_entries_for(
            user,
            for_date.to_string(),
            for_date.to_string(),
        )))
        .expect("Sending message to foreground");

    let title = format!("Harvest - {}", for_date.format("%a %-d %b"));
    to_foreground
        .send(Event::SetTitle(title))
        .expect("Sending message to foreground");
}
