use dwntp::receiver::listen::{Config, listen};

fn main() -> std::io::Result<()> {
    let config = Config {
        host: "0.0.0.0".to_string(),
        ..Default::default()
    };

    listen(config)?;

    Ok(())
}
