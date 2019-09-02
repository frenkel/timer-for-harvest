use std::io::Read;
use std::fs::File;
use serde;
use serde_json;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    token: String,
    account_id: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub id: u32,
    pub name: String,
    pub client: Client,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Client {
    pub id: u32,
    pub name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProjectPage {
    pub projects: Vec<Project>,
    pub per_page: u32,
    pub total_pages: u32,
    pub total_entries: u32,
    pub page: u32,
}

pub fn load_config() -> Config {
    let mut file = File::open("config.json").unwrap();
    let mut content = String::new();

    file.read_to_string(&mut content).unwrap();

    serde_json::from_str(&content).unwrap()
}

pub fn api_get_request(config: &Config, url: &str) -> reqwest::Response {
    let client = reqwest::Client::new();

    client.get(url)
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Harvest-Account-Id", format!("{}", config.account_id))
        .header("User-Agent", "Harvest Linux (TODO)")
        .send()
        .unwrap()
}
