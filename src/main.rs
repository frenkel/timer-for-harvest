use harvest::Harvest;

fn main() -> Result<(), Box<std::error::Error>> {
    let api = Harvest::new();
/*
    let projects = api.active_projects();
    for project in projects {
        println!("{} {}", project.client.name, project.name);
    }
*/
    let time_entries = api.time_entries();
    for time_entry in time_entries {
        println!("{} {}", time_entry.client.name, time_entry.project.name);
    }

    Ok(())
}
