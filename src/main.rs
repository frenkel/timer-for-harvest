pub mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    ui::main_window();

    Ok(())
}
