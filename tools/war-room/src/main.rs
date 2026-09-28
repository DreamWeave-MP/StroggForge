mod generate;
mod graphs;
mod model;
mod validate;
mod workflows;

use std::{env, path::Path};

use anyhow::{Result, bail};

fn main() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let command = env::args().nth(1);
    let model = model::Model::load(&root)?;
    validate::validate(&model)?;
    match command.as_deref() {
        Some("check") => {}
        Some("build") => generate::build(&root, &model)?,
        _ => bail!("usage: cargo run -p stroggforge-war-room -- <check|build>"),
    }
    println!(
        "Validated {} projects, {} plans and {} workflows.",
        model.ecosystem.projects.len(),
        model.plans.len(),
        model.workflows.len()
    );
    Ok(())
}
