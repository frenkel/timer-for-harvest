use harvest::Harvest;

fn main() -> Result<(), Box<std::error::Error>> {
    let api = Harvest::new();
    /*
        let projects = api.active_projects();
        for project in projects {
            println!("{} {}", project.client.name, project.name);
        }
    */
    let user = api.current_user();
    let time_entries = api.time_entries_today(user);
    for time_entry in time_entries {
        println!(
            "{} {}: {} {} at {}",
            time_entry.client.name,
            time_entry.project.name,
            time_entry.hours,
            time_entry.task.name,
            time_entry.spent_date
        );
    }

    Ok(())
}
