mod generate;
mod model;
mod sources;
mod validate;
mod workflows;

use anyhow::{Context, Result, bail};
use std::{env, fs, path::Path};

fn main() -> Result<()> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let inventory: model::Inventory = read_toml(&root.join("war-room/ecosystem.toml"))?;

    match arguments.as_slice() {
        [command, clones, source] if command == "capture" => {
            sources::capture(&root, Path::new(clones), source, &inventory)?;
        }
        [command] if command == "capture-organization" => {
            sources::capture_organization(&root, &inventory)?;
        }
        [command] if command == "build" || command == "check" => {
            let model = model::load(&root, inventory)?;
            validate::validate(&model)?;
            validate::policy_assertions(&root, &model)?;
            if command == "build" {
                generate::build(&root, &model)?;
            }
            println!(
                "Validated {} projects, {} retired records, {} relationships, {} plans, {} supply-chain stages and {} workflow/action contracts.",
                model.inventory.projects.len(),
                model.inventory.retired.len(),
                model.edges.len(),
                model.plans.plans.len(),
                model.operations.stage.len(),
                model.workflows.len()
            );
        }
        _ => bail!(
            "Usage: cargo run -p stroggforge-war-room -- <check|build|capture CLONES_DIRECTORY SOURCE_ID|capture-organization>\nRun from any directory. capture is explicit and offline; capture-organization reads GitHub through gh. See site/pages/contributing/maintenance.md."
        ),
    }
    Ok(())
}

fn read_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    toml::from_str(&fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("parse {}", path.display()))
}
