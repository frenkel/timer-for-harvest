use chrono::Local;
use hyper;
use serde;
use serde_json;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::process::Command;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Harvest {
    token: String,
    account_id: u32,
    expires_in: u32,
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

impl Project {
    pub fn name_and_code(&self) -> String {
        if self.code == "" {
            self.name.clone()
        } else {
            format!("[{}] {}", self.code, self.name)
        }
    }
}

impl Harvest {
    const CLIENT_ID: &'static str = "8ApPquiiqcpFrBt-GX7DhRDN";

    pub fn new() -> Harvest {
        let listener = TcpListener::bind("127.0.0.1:12345")
            .expect("port 12345 is already in use");

        Command::new("xdg-open")
            .arg(format!(
                "https://id.getharvest.com/oauth2/authorize?client_id={}&response_type=token",
                Harvest::CLIENT_ID
            ))
            .output()
            .expect("Unable to open browser");

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let result = Harvest::authorize_callback(stream);

            return Harvest {
                token: result.0,
                account_id: result.1.parse().unwrap(),
                expires_in: result.2.parse().unwrap(),
            };
        }

        panic!("unable to authorize");
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
                },
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

    fn user_agent() -> String {
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
    let minutes = duration % 1.0;
    let hours = duration - minutes;

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
