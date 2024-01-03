mod app;

use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new("FerroDMG".to_string())?;

    app.run(|_, _| ()).expect("Failed to run event loop");

    Ok(())
}
