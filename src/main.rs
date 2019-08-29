use std::io::Read;
use std::fs::File;
use serde;
use serde_json;

#[derive(serde::Serialize, serde::Deserialize)]
struct Config {
    token: String,
    account_id: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Project {
    id: u32,
    name: String,
    client: Client,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Client {
    id: u32,
    name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ProjectPage {
    projects: Vec<Project>,
    per_page: u32,
    total_pages: u32,
    total_entries: u32,
    page: u32,
}

fn load_config() -> Config {
    let mut file = File::open("config.json").unwrap();
    let mut content = String::new();

    file.read_to_string(&mut content).unwrap();

    serde_json::from_str(&content).unwrap()
}

fn api_get_request(config: &Config, url: &str) -> reqwest::Response {
    let client = reqwest::Client::new();

    client.get(url)
        .header("Authorization", format!("Bearer {}", config.token))
        .header("Harvest-Account-Id", format!("{}", config.account_id))
        .header("User-Agent", "Harvest Linux (TODO)")
        .send()
        .unwrap()
}

fn main() -> Result<(), Box<std::error::Error>> {
    let config = load_config();

    let mut res = api_get_request(&config, "https://api.harvestapp.com/v2/users/me");
    println!("{:}", res.text()?);

    res = api_get_request(&config, "https://api.harvestapp.com/v2/projects");
    let project_page: ProjectPage = serde_json::from_str(&res.text()?).unwrap();
    println!("{} {}", project_page.projects[0].client.name, project_page.projects[0].name);
    println!("{}", project_page.total_entries);

    Ok(())
}
