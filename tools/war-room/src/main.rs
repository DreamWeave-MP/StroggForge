mod generate;
mod graphs;
mod inspect;
mod model;
mod validate;
mod workflows;

use std::{env, path::PathBuf};

use anyhow::{Result, bail};

fn main() -> Result<()> {
    let command = env::args().nth(1);
    // `check` and `inspect` read another tree when given one: the fixture corpus the JSL forge is
    // compared against. `build` writes into site/ and always works on this checkout.
    let root = match (command.as_deref(), env::args().nth(2)) {
        (Some("check" | "inspect"), Some(root)) => PathBuf::from(root),
        _ => PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
    };
    let model = model::Model::load(&root)?;
    validate::validate(&model)?;
    match command.as_deref() {
        Some("check") => {}
        Some("build") => generate::build(&root, &model)?,
        // The contracts on stdout and nothing else, so the output can be diffed as it is.
        Some("inspect") => {
            print!("{}", inspect::inspect(&root, &model.workflows)?);
            return Ok(());
        }
        _ => {
            bail!("usage: cargo run -p stroggforge-war-room -- <check [root]|build|inspect [root]>")
        }
    }
    println!(
        "Validated {} projects, {} plans and {} workflows.",
        model.ecosystem.projects.len(),
        model.plans.len(),
        model.workflows.len()
    );
    Ok(())
}
