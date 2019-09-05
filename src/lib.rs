use std::io::Read;
use std::fs::File;
use serde;
use serde_json;

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
pub struct TimeEntry {
    pub id: u32,
    pub project: Project,
    pub client: Client,
    pub hours: f32,
    pub user: User,
    pub spent_date: String,
    pub task: Task,
    pub notes: String,
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
pub struct TimeEntryPage {
    pub time_entries: Vec<TimeEntry>,
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

    pub fn active_projects(&self) -> Vec<Project> {
        let mut projects: Vec<Project> = vec![];
        let mut current_page = 1;

        loop {
            let url = format!("https://api.harvestapp.com/v2/projects?is_active=true&page={}", current_page);
            let mut res = self.api_get_request(&url);
            let body = &res.text().unwrap();
            let page: ProjectPage = serde_json::from_str(body).unwrap();
            if current_page == page.total_pages {
                projects.extend(page.projects);
                break;
            } else {
                current_page += 1;
                projects.extend(page.projects);
            }
        }

        projects
    }

    pub fn time_entries(&self, user: User) -> Vec<TimeEntry> {
        let url = format!("https://api.harvestapp.com/v2/time_entries?user_id={}", user.id);
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

    fn api_get_request(&self, url: &str) -> reqwest::Response {
        let client = reqwest::Client::new();

        client.get(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Harvest-Account-Id", format!("{}", self.account_id))
            .header("User-Agent", "Harvest Linux (TODO)")
            .send()
            .unwrap()
    }
}
