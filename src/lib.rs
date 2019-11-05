use chrono::Local;
use serde;
use serde_json;
use std::collections::HashMap;
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
    pub client: Option<Client>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProjectAssignment {
    pub id: u32,
    pub project: Project,
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
    pub project: Project,
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

    pub fn active_projects(&self, user: User) -> Vec<Project> {
        let mut projects: Vec<Project> = vec![];
        let mut current_page = 1;

        loop {
            let url = format!(
                "https://api.harvestapp.com/v2/users/{}/project_assignments?page={}",
                user.id,
                current_page
            );
            let mut res = self.api_get_request(&url);
            let body = &res.text().unwrap();
            let page: ProjectAssignmentPage = serde_json::from_str(body).unwrap();
            for project_assignment in page.project_assignments {
                projects.push(project_assignment.project);
            }
            if current_page == page.total_pages {
                break;
            } else {
                current_page += 1;
            }
        }

        projects
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

    pub fn project_task_assignments(&self, project: &Project) -> Vec<TaskAssignment> {
        let url = format!(
            "https://api.harvestapp.com/v2/projects/{}/task_assignments",
            project.id
        );
        let mut res = self.api_get_request(&url);
        let body = &res.text().unwrap();
        let page: TaskAssignmentPage = serde_json::from_str(body).unwrap();

        page.task_assignments
    }

    pub fn start_timer(&self, project: &Project, task: &Task) -> TimeEntry {
        let url = "https://api.harvestapp.com/v2/time_entries";
        let mut map = HashMap::new();
        let now = Local::now().format("%Y-%m-%d");
        map.insert("project_id", format!("{}", project.id));
        map.insert("task_id", format!("{}", task.id));
        map.insert("spent_date", now.to_string());

        let mut res = self.api_post_request(&url, &map);
        let body = &res.text().unwrap();
        serde_json::from_str(body).unwrap()
    }

    pub fn restart_timer(&self, time_entry: &TimeEntry) -> TimeEntry {
        let url = format!(
            "https://api.harvestapp.com/v2/time_entries/{}/restart",
            time_entry.id
        );

        let mut res = self.api_patch_request(&url);
        let body = &res.text().unwrap();
        serde_json::from_str(body).unwrap()
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

    fn api_post_request(&self, url: &str, map: &HashMap<&str, String>) -> reqwest::Response {
        let client = reqwest::Client::new();

        client
            .post(url)
            .json(&map)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Harvest-Account-Id", format!("{}", self.account_id))
            .header("User-Agent", "Harvest Linux (TODO)")
            .send()
            .unwrap()
    }

    fn api_patch_request(&self, url: &str) -> reqwest::Response {
        let client = reqwest::Client::new();

        client
            .patch(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Harvest-Account-Id", format!("{}", self.account_id))
            .header("User-Agent", "Harvest Linux (TODO)")
            .send()
            .unwrap()
    }
}
