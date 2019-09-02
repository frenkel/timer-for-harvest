fn main() -> Result<(), Box<std::error::Error>> {
    let config = harvest::load_config();

    let mut res = harvest::api_get_request(&config, "https://api.harvestapp.com/v2/users/me");
    println!("{:}", res.text()?);

    res = harvest::api_get_request(&config, "https://api.harvestapp.com/v2/projects");
    let project_page: harvest::ProjectPage = serde_json::from_str(&res.text()?).unwrap();
    println!("{} {}", project_page.projects[0].client.name, project_page.projects[0].name);
    println!("{}", project_page.total_entries);

    Ok(())
}
