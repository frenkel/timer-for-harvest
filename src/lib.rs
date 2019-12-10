use chrono::Local;
use serde;
use serde_json;
use std::fs::File;
use std::io::Read;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Harvest {
    token: String,
    account_id: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub id: u32,
    pub name: String,
    pub code: String,
    pub client: Option<Client>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProjectAssignment {
    pub id: u32,
    pub project: Project,
    pub task_assignments: Vec<TaskAssignment>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Client {
    pub id: u32,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: u32,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TaskAssignment {
    pub id: u32,
    pub task: Task,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TimeEntry {
    pub id: u32,
    pub project: Project,
    pub client: Client,
    pub hours: f32,
    pub user: User,
    pub spent_date: String,
    pub task: Task,
    pub notes: Option<String>,
    pub is_running: bool,
}

/* a partially filled TimeEntry with id's instead of objects (Project etc) */
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Timer {
    pub id: Option<u32>,
    pub project_id: u32,
    pub task_id: u32,
    pub spent_date: Option<String>,
    pub notes: Option<String>,
    pub hours: Option<f32>,
    pub is_running: bool,
}

/* a partially filled TimeEntry with id's instead of objects (Project etc) */
#[derive(serde::Serialize, serde::Deserialize)]
pub struct TimerWithoutHours {
    pub id: Option<u32>,
    pub project_id: u32,
    pub task_id: u32,
    pub spent_date: Option<String>,
    pub notes: Option<String>,
    pub is_running: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProjectPage {
    pub projects: Vec<Project>,
    pub per_page: u32,
    pub total_pages: u32,
    pub total_entries: u32,
    pub page: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProjectAssignmentPage {
    pub project_assignments: Vec<ProjectAssignment>,
    pub per_page: u32,
    pub total_pages: u32,
    pub total_entries: u32,
    pub page: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TimeEntryPage {
    pub time_entries: Vec<TimeEntry>,
    pub per_page: u32,
    pub total_pages: u32,
    pub total_entries: u32,
    pub page: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TaskAssignmentPage {
    pub task_assignments: Vec<TaskAssignment>,
    pub per_page: u32,
    pub total_pages: u32,
    pub total_entries: u32,
    pub page: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: u32,
}

impl Harvest {
    pub fn new() -> Harvest {
        let mut file = File::open("config.json").unwrap();
        let mut content = String::new();

        file.read_to_string(&mut content).unwrap();

        serde_json::from_str(&content).unwrap()
    }

    pub fn active_project_assignments(&self) -> Vec<ProjectAssignment> {
        let mut project_assignments: Vec<ProjectAssignment> = vec![];
        let mut current_page = 1;

        loop {
            let url = format!(
                "https://api.harvestapp.com/v2/users/me/project_assignments?page={}",
                current_page
            );
            let mut res = self.api_get_request(&url);
            let body = &res.text().unwrap();
            let page: ProjectAssignmentPage = serde_json::from_str(body).unwrap();

            for project_assignment in page.project_assignments {
                project_assignments.push(project_assignment);
            }

            if current_page == page.total_pages {
                break;
            } else {
                current_page += 1;
            }
        }

        project_assignments
    }

    pub fn time_entries_today(&self, user: User) -> Vec<TimeEntry> {
        let now = Local::now().format("%Y-%m-%d");
        let url = format!(
            "https://api.harvestapp.com/v2/time_entries?user_id={}&from={}&to={}",
            user.id, now, now
        );
        let mut res = self.api_get_request(&url);
        let body = &res.text().unwrap();
        let page: TimeEntryPage = serde_json::from_str(body).unwrap();

        page.time_entries
    }

    pub fn current_user(&self) -> User {
        let url = "https://api.harvestapp.com/v2/users/me";
        let mut res = self.api_get_request(&url);
        let body = &res.text().unwrap();
        serde_json::from_str(body).unwrap()
    }

    pub fn start_timer(
        &self,
        project: &Project,
        task: &Task,
        notes: &str,
        hours: f32,
    ) -> TimeEntry {
        let url = "https://api.harvestapp.com/v2/time_entries";
        let now = Local::now().format("%Y-%m-%d");
        let mut timer = Timer {
            id: None,
            project_id: project.id,
            task_id: task.id,
            spent_date: Some(now.to_string()),
            notes: None,
            hours: None,
            is_running: true,
        };
        if notes.len() > 0 {
            timer.notes = Some(notes.to_string());
        }
        if hours > 0.0 {
            timer.hours = Some(hours);
        }

        let mut res = self.api_post_request(&url, &timer);
        let body = &res.text().unwrap();
        serde_json::from_str(body).unwrap()
    }

    pub fn restart_timer(&self, time_entry: &TimeEntry) -> TimeEntry {
        let url = format!(
            "https://api.harvestapp.com/v2/time_entries/{}/restart",
            time_entry.id
        );

        let mut res = self.api_patch_request(&url, &());
        let body = &res.text().unwrap();
        serde_json::from_str(body).unwrap()
    }

    pub fn stop_timer(&self, time_entry: &TimeEntry) -> TimeEntry {
        let url = format!(
            "https://api.harvestapp.com/v2/time_entries/{}/stop",
            time_entry.id
        );

        let mut res = self.api_patch_request(&url, &());
        let body = &res.text().unwrap();
        serde_json::from_str(body).unwrap()
    }

    pub fn update_timer(&self, timer: &Timer) -> TimeEntry {
        let url = format!(
            "https://api.harvestapp.com/v2/time_entries/{}",
            timer.id.unwrap()
        );

        /* TODO how not to sent hours when is_running in a better way? */
        if timer.is_running {
            let t2 = TimerWithoutHours {
                id: timer.id,
                project_id: timer.project_id,
                task_id: timer.task_id,
                notes: Some(timer.notes.as_ref().unwrap().to_string()),
                is_running: true,
                spent_date: Some(timer.spent_date.as_ref().unwrap().to_string()),
            };

            let mut res = self.api_patch_request(&url, &t2);
            let body = &res.text().unwrap();
            serde_json::from_str(body).unwrap()
        } else {
            let mut res = self.api_patch_request(&url, &timer);
            let body = &res.text().unwrap();
            serde_json::from_str(body).unwrap()
        }
    }

    fn api_get_request(&self, url: &str) -> reqwest::Response {
        let client = reqwest::Client::new();

        client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Harvest-Account-Id", format!("{}", self.account_id))
            .header("User-Agent", "Harvest Linux (TODO)")
            .send()
            .unwrap()
    }

    fn api_post_request<T: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        json: &T,
    ) -> reqwest::Response {
        let client = reqwest::Client::new();

        client
            .post(url)
            .json(&json)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Harvest-Account-Id", format!("{}", self.account_id))
            .header("User-Agent", "Harvest Linux (TODO)")
            .send()
            .unwrap()
    }

    fn api_patch_request<T: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        json: &T,
    ) -> reqwest::Response {
        let client = reqwest::Client::new();

        client
            .patch(url)
            .json(&json)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Harvest-Account-Id", format!("{}", self.account_id))
            .header("User-Agent", "Harvest Linux (TODO)")
            .send()
            .unwrap()
    }
}

/* TODO move to TimeEntry */
pub fn duration_str_to_f32(duration: &str) -> f32 {
    if duration.len() > 0 {
        let mut parts = duration.split(":");
        let hours: f32 = match parts.next() {
            None => { 0.0 }
            Some(h) => {
                match h.parse() {
                    Ok(p) => { p }
                    Err(_) => { 0.0 }
                }
            }
        };
        let minutes: f32 = match parts.next() {
            None => { 0.0 }
            Some(m) => {
                match m.parse() {
                    Ok(p) => { p }
                    Err(_) => { 0.0 }
                }
            }
        };
        hours + minutes / 60.0
    } else {
        0.0
    }
}

/* TODO move to TimeEntry */
pub fn f32_to_duration_str(duration: f32) -> String {
    let minutes = duration % 1.0;
    let hours = duration - minutes;

    format!("{:.0}:{:0>2.0}", hours, minutes * 60.0)
}
