use harvest::Harvest;

fn main() -> Result<(), Box<std::error::Error>> {
    let api = Harvest::new();

    let projects = api.active_projects();
    for project in projects {
        println!("{} {}", project.client.name, project.name);
    }

    Ok(())
}
