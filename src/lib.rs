use chrono::Local;
use dirs;
use hyper;
use serde;
use serde_json::json;
use std::fs::write;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Harvest {
    token: String,
    account_id: u32,
    expires_at: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Project {
    pub id: u32,
    pub name: String,
    pub code: Option<String>,
    pub client: Option<Client>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ProjectAssignment {
    pub id: u32,
    pub project: Project,
    pub task_assignments: Vec<TaskAssignment>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Client {
    pub id: u32,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Task {
    pub id: u32,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct TaskAssignment {
    pub id: u32,
    pub task: Task,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct User {
    pub id: u32,
}

impl Project {
    pub fn name_and_code(&self) -> String {
        if self.code == None || self.code.as_ref().unwrap() == "" {
            self.name.clone()
        } else {
            format!("[{}] {}", self.code.as_ref().unwrap(), self.name)
        }
    }
}

impl Harvest {
    const CLIENT_ID: &'static str = "8ApPquiiqcpFrBt-GX7DhRDN";
    const CONFIG_FILE_NAME: &'static str = "timer-for-harvest.json";

    pub fn new() -> Harvest {
        match Harvest::read_authorization_from_file() {
            Some(harvest) => {
                let unix_timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let one_day = 60 * 60 * 24;

                if harvest.expires_at < unix_timestamp + one_day {
                    Harvest::obtain_new_authorization()
                } else {
                    return harvest;
                }
            }
            None => Harvest::obtain_new_authorization(),
        }
    }

    fn obtain_new_authorization() -> Harvest {
        let listener = TcpListener::bind("127.0.0.1:12345").expect("Port 12345 is already in use");

        Command::new("xdg-open")
            .arg(format!(
                "https://id.getharvest.com/oauth2/authorize?client_id={}&response_type=token",
                Harvest::CLIENT_ID
            ))
            .spawn()
            .expect("Unable to open browser");

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let result = Harvest::authorize_callback(stream);

            let unix_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let expires_in: u64 = result.2.parse().unwrap();

            let harvest = Harvest {
                token: result.0,
                account_id: result.1.parse().unwrap(),
                expires_at: unix_timestamp + expires_in,
            };
            harvest.write_authorization_to_file();
            return harvest;
        }

        panic!("unable to authorize");
    }

    fn read_authorization_from_file() -> Option<Harvest> {
        match File::open(Harvest::config_file_path()) {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                Some(
                    serde_json::from_str(&content)
                        .expect(&format!("Invalid configuration file: {}", content).to_string()),
                )
            }
            Err(_) => None,
        }
    }

    fn write_authorization_to_file(&self) {
        write(Harvest::config_file_path(), json!(self).to_string())
            .expect("unable to save config file");
    }

    fn config_file_path() -> PathBuf {
        let mut path = dirs::config_dir().expect("Unable to find XDG config dir path");
        path.push(Harvest::CONFIG_FILE_NAME);
        path
    }

    fn authorize_callback(mut stream: TcpStream) -> (String, String, String) {
        let mut buffer = [0; 512];
        let mut first_line = "".to_string();

        loop {
            match stream.read(&mut buffer) {
                Ok(n) => {
                    if first_line == "" {
                        let request = String::from_utf8_lossy(&buffer[..]).to_string();
                        first_line = request.lines().next().unwrap().to_string();
                    }
                    if n < buffer.len() {
                        break;
                    }
                }
                Err(_) => {
                    panic!("unable to read request");
                }
            }
        }

        let result = parse_account_details(&first_line);

        let response = "HTTP/1.1 200 OK\r\n\r\n<!DOCTYPE html>
            <html>
                <body>
                    Authorized successfully
                </body>
            </html>\r\n";

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();

        result
    }

    pub fn user_agent() -> String {
        format!(
            "{} {}.{}.{}{} ({})",
            env!("CARGO_PKG_DESCRIPTION"),
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH"),
            option_env!("CARGO_PKG_VERSION_PRE").unwrap_or(""),
            env!("CARGO_PKG_HOMEPAGE")
        )
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
            let page: ProjectAssignmentPage = serde_json::from_str(body).expect(
                &format!("Unexpected project assignment page structure: {}", body).to_string(),
            );

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

    pub fn time_entries_for(&self, user: &User, from: String, till: String) -> Vec<TimeEntry> {
        let url = format!(
            "https://api.harvestapp.com/v2/time_entries?user_id={}&from={}&to={}",
            user.id, from, till
        );
        let mut res = self.api_get_request(&url);
        let body = &res.text().unwrap();
        let page: TimeEntryPage = serde_json::from_str(body)
            .expect(&format!("Unexpected time entry page structure: {}", body).to_string());

        page.time_entries
    }

    pub fn current_user(&self) -> User {
        let url = "https://api.harvestapp.com/v2/users/me";
        let mut res = self.api_get_request(&url);
        let body = &res.text().unwrap();
        serde_json::from_str(body)
            .expect(&format!("Unexpected user structure: {}", body).to_string())
    }

    pub fn start_timer(&self, project_id: u32, task_id: u32, notes: &str, hours: f32) -> TimeEntry {
        let url = "https://api.harvestapp.com/v2/time_entries";
        let now = Local::now().format("%Y-%m-%d");
        let mut timer = Timer {
            id: None,
            project_id: project_id,
            task_id: task_id,
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
        serde_json::from_str(body)
            .expect(&format!("Unexpected timer structure: {}", body).to_string())
    }

    pub fn restart_timer(&self, time_entry_id: u32) -> TimeEntry {
        let url = format!(
            "https://api.harvestapp.com/v2/time_entries/{}/restart",
            time_entry_id
        );

        let mut res = self.api_patch_request(&url, &());
        let body = &res.text().unwrap();
        serde_json::from_str(body)
            .expect(&format!("Unexpected timer structure: {}", body).to_string())
    }

    pub fn stop_timer(&self, time_entry_id: u32) -> TimeEntry {
        let url = format!(
            "https://api.harvestapp.com/v2/time_entries/{}/stop",
            time_entry_id
        );

        let mut res = self.api_patch_request(&url, &());
        let body = &res.text().unwrap();
        serde_json::from_str(body)
            .expect(&format!("Unexpected timer structure: {}", body).to_string())
    }

    pub fn update_timer(
        &self,
        id: u32,
        project_id: u32,
        task_id: u32,
        notes: String,
        hours: f32,
        is_running: bool,
        spent_date: String,
    ) -> TimeEntry {
        let url = format!("https://api.harvestapp.com/v2/time_entries/{}", id);

        /* TODO how not to sent hours when is_running in a better way? */
        if is_running {
            let t2 = TimerWithoutHours {
                id: Some(id),
                project_id: project_id,
                task_id: task_id,
                notes: Some(notes),
                is_running: is_running,
                spent_date: Some(spent_date),
            };

            let mut res = self.api_patch_request(&url, &t2);
            let body = &res.text().unwrap();
            serde_json::from_str(body)
                .expect(&format!("Unexpected time entry structure: {}", body).to_string())
        } else {
            let timer = Timer {
                id: Some(id),
                project_id: project_id,
                task_id: task_id,
                notes: Some(notes),
                is_running: is_running,
                hours: Some(hours),
                spent_date: Some(spent_date),
            };
            let mut res = self.api_patch_request(&url, &timer);
            let body = &res.text().unwrap();
            serde_json::from_str(body)
                .expect(&format!("Unexpected time entry structure: {}", body).to_string())
        }
    }

    pub fn delete_timer(&self, timer_id: u32) -> TimeEntry {
        let url = format!("https://api.harvestapp.com/v2/time_entries/{}", timer_id);

        let mut res = self.api_delete_request(&url);
        let body = &res.text().unwrap();
        serde_json::from_str(body)
            .expect(&format!("Unexpected timer structure: {}", body).to_string())
    }

    fn api_get_request(&self, url: &str) -> reqwest::Response {
        let client = reqwest::Client::new();

        client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Harvest-Account-Id", format!("{}", self.account_id))
            .header("User-Agent", Harvest::user_agent())
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
            .header("User-Agent", Harvest::user_agent())
            .send()
            .unwrap()
    }

    fn api_delete_request(&self, url: &str) -> reqwest::Response {
        let client = reqwest::Client::new();

        client
            .delete(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Harvest-Account-Id", format!("{}", self.account_id))
            .header("User-Agent", Harvest::user_agent())
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
            .header("User-Agent", Harvest::user_agent())
            .send()
            .unwrap()
    }
}

/* TODO move to TimeEntry */
pub fn duration_str_to_f32(duration: &str) -> f32 {
    if duration.len() > 0 {
        let mut parts = duration.split(":");
        let hours: f32 = match parts.next() {
            None => 0.0,
            Some(h) => match h.parse() {
                Ok(p) => p,
                Err(_) => 0.0,
            },
        };
        let minutes: f32 = match parts.next() {
            None => 0.0,
            Some(m) => match m.parse() {
                Ok(p) => p,
                Err(_) => 0.0,
            },
        };
        hours + minutes / 60.0
    } else {
        0.0
    }
}

/* TODO move to TimeEntry */
pub fn f32_to_duration_str(duration: f32) -> String {
    let mut minutes = duration % 1.0;
    let mut hours = duration - minutes;

    if format!("{:0>2.0}", minutes * 60.0) == "60" {
        minutes = 0.0;
        hours += 1.0;
    }
    format!("{:.0}:{:0>2.0}", hours, minutes * 60.0)
}

/* TODO improve this messy parse function */
pub fn parse_account_details(request: &str) -> (String, String, String) {
    let mut parts = request.split(" ");
    parts.next(); /* GET */
    let uri = parts.next().unwrap().parse::<hyper::Uri>().unwrap();
    let parts = uri.query().unwrap().split("&");
    let mut access_token = "";
    let mut account_id = "";
    let mut expires_in = "";

    for part in parts {
        if part.starts_with("access_token") {
            let mut parts = part.split("=");
            parts.next();
            access_token = parts.next().unwrap();
        } else if part.starts_with("scope") {
            let mut parts = part.split("=");
            parts.next();
            account_id = parts.next().unwrap();
            let mut parts = account_id.split("%3A");
            parts.next();
            account_id = parts.next().unwrap();
        } else if part.starts_with("expires_in") {
            let mut parts = part.split("=");
            parts.next();
            expires_in = parts.next().unwrap();
        }
    }
    (
        access_token.to_string(),
        account_id.to_string(),
        expires_in.to_string(),
    )
}
