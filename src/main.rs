use harvest::Harvest;

fn main() -> Result<(), Box<std::error::Error>> {
    let api = Harvest::new();

    let project_pages = api.active_projects();
    for page in project_pages {
        for project in page.projects {
            println!("{} {}", project.client.name, project.name);
        }
    }

    Ok(())
}
