use std::error;

use toml::Value;

fn main() -> Result<(), Box<dyn error::Error>> {
    let cargo_toml = include_str!("Cargo.toml");
    let cargo_toml = cargo_toml.parse::<Value>()?;

    println!(
        "cargo:rustc-env=LIB_BUILD_VERSION={}",
        cargo_toml["package"]["version"].as_str().ok_or("Cargo.toml: `version` not specify.")?
    );

    Ok(())
}
