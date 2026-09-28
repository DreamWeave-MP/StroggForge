use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    path::Path,
};

use anyhow::Result;
use serde_yaml_ng::Value;

use super::{cell, diagram, graphs, page, project_link, section, write};
use crate::{
    model::{Model, Relationship},
    workflows::{Note, Workflow, resolve, text},
};

pub fn build(site: &Path, root: &Path, model: &Model) -> Result<()> {
    section(
        site,
        "stroggforge/workflows",
        "Canonical workflows and actions",
        10,
        "Contracts and job graphs below are read from the actual YAML at build time. Inputs, secrets, outputs, job conditions, permissions and platform strategy are not copied into a parallel API schema. Companion operational notes live beside the workflows in `.github/war-room-workflows.toml`.\n",
    )?;
    for workflow in &model.workflows {
        workflow_page(site, workflow, model.operations.workflow.get(&workflow.id))?;
    }
    integration(site, root, model)?;
    inputs(site, model)?;
    supply_chain(site, model)
}

fn integration(site: &Path, root: &Path, model: &Model) -> Result<()> {
    let mut body = String::from(
        "## Verified callers\n\nThese are repository-level calls parsed from reviewed workflow snapshots. A caller at an older ref does not automatically acquire the configuration described by this checkout. Integration is not successful-run evidence.\n\n| Component | Reusable workflow and ref | Source receipt |\n|---|---|---|\n",
    );
    for edge in model
        .edges
        .iter()
        .filter(|edge| edge.kind == Relationship::Build)
    {
        writeln!(
            body,
            "| {} | {} | {} |",
            project_link(model.project(&edge.from)),
            cell(&edge.label),
            cell(&edge.evidence)
        )?;
    }
    for (heading, introduction, template) in [
        (
            "Binary integration template",
            "Review permissions, secrets, publish flags, binary/package selection and the pinned ref before installing this caller.",
            "rust_template.yaml",
        ),
        (
            "Library integration template",
            "`crate_names` must match Cargo `[package].name`. Enable `cargo_publish_workspace` for dependency-ordered workspace publishing.",
            "lib_template.yaml",
        ),
        (
            "Daily quality template",
            "A scheduled fmt/Clippy/RustSec check that needs no release secrets. It calls `setup-llvm` directly because Clippy compiles.",
            "daily_quality_template.yaml",
        ),
    ] {
        let source = fs::read_to_string(root.join(".github/action_templates").join(template))?;
        writeln!(
            body,
            "\n## {heading}\n\n{introduction} Source: `.github/action_templates/{template}`.\n\n```yaml\n{}\n```",
            source.trim_end()
        )?;
    }
    body.push_str("\n[Input reference](@/stroggforge/inputs.md) · [Failure triage](@/stroggforge/troubleshooting.md) · [Publishing policy](@/releases/publishing.md)\n");
    page(
        site,
        "stroggforge/integration",
        "Integration and actual consumers",
        "Who calls StroggForge, at which ref, and how to integrate.",
        &body,
    )
}

/// One matrix per contract kind, so shared knobs across the binary and library workflows
/// can be compared without opening each contract page.
fn inputs(site: &Path, model: &Model) -> Result<()> {
    let mut body = String::from(
        "Every input and secret declared by a reusable workflow or composite action, side by side. Cells show the default, **required**, or `·` where the contract does not declare the name. Types and descriptions live on each [contract page](@/stroggforge/workflows/_index.md). Generated from YAML; there is no second schema to keep in sync.\n",
    );
    for (heading, select) in [
        (
            "Inputs",
            (|workflow| &workflow.inputs) as fn(&Workflow) -> &BTreeMap<String, Value>,
        ),
        ("Secrets", |workflow| &workflow.secrets),
    ] {
        let contracts: Vec<&Workflow> = model
            .workflows
            .iter()
            .filter(|workflow| !select(workflow).is_empty())
            .collect();
        let names: BTreeSet<&String> = contracts
            .iter()
            .flat_map(|workflow| select(workflow).keys())
            .collect();
        writeln!(body, "\n## {heading}\n")?;
        body.push_str("| Name |");
        for workflow in &contracts {
            write!(
                body,
                " [{}](@/stroggforge/workflows/{}.md) |",
                workflow.id, workflow.id
            )?;
        }
        body.push_str("\n|---|");
        body.push_str(&"---|".repeat(contracts.len()));
        body.push('\n');
        for name in names {
            write!(body, "| `{}` |", cell(name))?;
            for workflow in &contracts {
                let value = select(workflow).get(name);
                let shown = match value {
                    None => "·".to_owned(),
                    Some(value) if value.get("required").and_then(Value::as_bool) == Some(true) => {
                        "**required**".to_owned()
                    }
                    Some(value) => value
                        .get("default")
                        .map_or("optional".to_owned(), |default| {
                            format!("`{}`", cell(&text(default)))
                        }),
                };
                write!(body, " {shown} |")?;
            }
            body.push('\n');
        }
    }
    body.push_str("\nAn optional secret can still be operationally required: non-PR binary releases need `VT_API_KEY`, and publishing needs `CARGO_REGISTRY_TOKEN`. The [contract notes](@/stroggforge/workflows/rustGlobalBuild.md#assumptions-and-permissions) record those conditions.\n");
    page(
        site,
        "stroggforge/inputs",
        "Input and secret reference",
        "Every workflow knob and secret, compared across contracts.",
        &body,
    )
}

fn workflow_page(site: &Path, workflow: &Workflow, note: Option<&Note>) -> Result<()> {
    let mut body = format!(
        "Source: [{}](https://github.com/DreamWeave-MP/StroggForge/blob/main/{}) · **{}**\n\n",
        workflow.path,
        workflow.path,
        if workflow.action {
            "composite action"
        } else if workflow.callable {
            "reusable workflow"
        } else {
            "repository automation"
        }
    );
    if let Some(note) = note {
        append_notes(&mut body, note)?;
    } else {
        body.push_str("Repository-local automation. Inspect the generated job table, conditions and source before invoking it.\n\n");
    }
    for (heading, fields) in [
        ("Inputs", &workflow.inputs),
        ("Secrets", &workflow.secrets),
        ("Outputs", &workflow.outputs),
    ] {
        writeln!(body, "## {heading}\n")?;
        if fields.is_empty() {
            body.push_str("None declared in this contract.\n\n");
            continue;
        }
        body.push_str(
            "| Name | Type | Required | Default / value | Description |\n|---|---|---|---|---|\n",
        );
        for (name, value) in fields {
            writeln!(
                body,
                "| `{}` | {} | {} | {} | {} |",
                cell(name),
                field(value, "type"),
                field(value, "required"),
                value
                    .get("default")
                    .or_else(|| value.get("value"))
                    .map_or("—".into(), |value| cell(&text(value))),
                field(value, "description")
            )?;
        }
        body.push('\n');
    }
    writeln!(
        body,
        "## Permissions\n\nTop-level declaration: `{}`. A missing declaration is **inherited**, not an assurance that no permissions are needed. Job overrides appear below; reusable callers must grant sufficient permissions. `GITHUB_TOKEN` is built-in and is not listed as a workflow-call secret.\n",
        cell(&text(&workflow.permissions))
    )?;
    if !workflow.jobs.is_empty() {
        let graph_id = format!("workflow-{}", workflow.id);
        write(
            site,
            &format!("static/generated/maps/{graph_id}.mmd"),
            &graphs::workflow_graph(workflow)?,
        )?;
        body.push_str("## Job dependency graph\n\nEdges represent YAML `needs`, not a claim that every conditional job runs. Conditions are listed exactly below.\n");
        body.push_str(&diagram(&graph_id));
        body.push_str("## Jobs and platforms\n\n| Job | Needs | Runner / delegate | Container / strategy | Permissions | Condition |\n|---|---|---|---|---|---|\n");
        for (name, job) in &workflow.jobs {
            writeln!(
                body,
                "| `{}` | {} | {} / {} | {} / {} | {} | {} |",
                cell(name),
                cell(&job.dependencies()?.join(", ")),
                cell(&text(&job.runner)),
                cell(&text(&job.uses)),
                cell(&text(&job.container)),
                cell(&text(&job.strategy)),
                cell(&text(&job.permissions)),
                cell(&text(&job.condition))
            )?;
        }
    }
    body.push_str("\n## Executed steps\n\nThe commands and delegated actions below are extracted from the workflow, including conditions. This makes the actual quality commands and packaging order inspectable without opening YAML.\n");
    for (name, job) in &workflow.jobs {
        steps(&mut body, name, &job.steps)?;
    }
    if !workflow.steps.is_empty() {
        steps(&mut body, "Composite action", &workflow.steps)?;
    }
    body.push_str("\n[Incident desk](@/stroggforge/troubleshooting.md) · [Platform policy](@/releases/platforms.md)\n");
    page(
        site,
        &format!("stroggforge/workflows/{}", workflow.id),
        &workflow.name,
        &workflow.path,
        &body,
    )
}

fn field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .map_or("—".into(), |value| cell(&text(value)))
}

fn append_notes(body: &mut String, note: &Note) -> Result<()> {
    writeln!(
        body,
        "{}\n\n**Intended for:** {}\n\n## Release behavior\n\n{}\n\n## Artifacts and effects\n",
        note.purpose, note.intended_for, note.release_behavior
    )?;
    for artifact in &note.artifacts {
        writeln!(body, "- {artifact}")?;
    }
    body.push_str("\n## Assumptions and permissions\n\n");
    for assumption in &note.assumptions {
        writeln!(body, "- {assumption}")?;
    }
    body.push_str("\n## Failure modes\n\n");
    for failure in &note.failures {
        writeln!(body, "- {failure}")?;
    }
    writeln!(body, "\n## Example / entry point\n\n{}\n", note.example)?;
    Ok(())
}

fn steps(body: &mut String, heading: &str, steps: &[Value]) -> Result<()> {
    writeln!(
        body,
        "\n### {heading}\n\n| Step | Action / command | Condition |\n|---|---|---|\n"
    )?;
    for step in steps {
        writeln!(
            body,
            "| {} | {} | {} |",
            field(step, "name"),
            step.get("uses")
                .or_else(|| step.get("run"))
                .map_or("—".into(), |value| cell(&text(value))),
            field(step, "if")
        )?;
    }
    Ok(())
}

/// Stage references grouped by workflow, in first-seen order: `[(workflow id, [job or step])]`.
pub fn stage_groups<'a>(
    model: &'a Model,
    jobs: &'a [String],
) -> Result<Vec<(&'a str, Vec<&'a str>)>> {
    let mut groups: Vec<(&str, Vec<&str>)> = Vec::new();
    for reference in jobs {
        let (workflow, name) = resolve(&model.workflows, reference)?;
        match groups.iter_mut().find(|(id, _)| *id == workflow.id) {
            Some((_, names)) => names.push(name),
            None => groups.push((&workflow.id, vec![name])),
        }
    }
    Ok(groups)
}

/// Every current project with a verified StroggForge caller, whichever workflow it calls.
pub fn consumers(model: &Model) -> BTreeSet<&str> {
    model
        .callers
        .values()
        .flatten()
        .map(String::as_str)
        .collect()
}

/// `workflow: Project, Project` per called workflow, linking workflows that exist at this checkout.
pub fn callers_by_workflow(model: &Model) -> Vec<String> {
    model
        .callers
        .iter()
        .map(|(workflow, projects)| {
            let projects = projects
                .iter()
                .map(|id| project_link(model.project(id)))
                .collect::<Vec<_>>()
                .join(", ");
            if model
                .workflows
                .iter()
                .any(|candidate| &candidate.id == workflow)
            {
                format!("[{workflow}](@/stroggforge/workflows/{workflow}.md): {projects}")
            } else {
                format!("{workflow} (not in this checkout): {projects}")
            }
        })
        .collect()
}

pub fn stage_table(model: &Model) -> Result<String> {
    let mut table = String::from(
        "| # | Stage | Participating jobs and steps | What happens |\n|---|---|---|---|\n",
    );
    for (index, stage) in model.operations.stage.iter().enumerate() {
        let groups = stage_groups(model, &stage.jobs)?;
        let jobs = if groups.is_empty() {
            "caller workflow".to_owned()
        } else {
            groups
                .iter()
                .map(|(workflow, names)| {
                    format!(
                        "[{workflow}](@/stroggforge/workflows/{workflow}.md): {}",
                        cell(&names.join(", "))
                    )
                })
                .collect::<Vec<_>>()
                .join("<br>")
        };
        writeln!(
            table,
            "| {} | **{}** | {} | {} |",
            index + 1,
            cell(&stage.name),
            jobs,
            cell(&stage.summary)
        )?;
    }
    Ok(table)
}

fn supply_chain(site: &Path, model: &Model) -> Result<()> {
    write(
        site,
        "static/generated/maps/supply-chain.mmd",
        &graphs::supply_chain_graph(model)?,
    )?;
    let mut body = String::from(
        "## Manufacturing stages\n\nStages and their participating jobs are reviewed data in `.github/war-room-workflows.toml`; every reference is resolved against the executable YAML, so a renamed or deleted job fails generation. Stages are ordered by the earliest point each can start, not a strict sequence: jobs overlap wherever the [binary](@/stroggforge/workflows/rustGlobalBuild.md#job-dependency-graph) and [library](@/stroggforge/workflows/libGlobalBuild.md#job-dependency-graph) `needs` graphs allow. Signing covers the **binary before archive creation**. Pull requests build and upload a workflow artifact; they skip signing, scanning and every upload.\n",
    );
    body.push_str(&diagram("supply-chain"));
    body.push_str(&stage_table(model)?);
    body.push_str("\n## Who uses the line\n\nCurrent components whose repository workflows call StroggForge, grouped by the workflow they call. Rust manufacturing goes through `rustGlobalBuild` and `libGlobalBuild`; `createRelease` alone is the release-boundary helper that static sites use. [Exact refs](@/stroggforge/integration.md#verified-callers).\n\n");
    for line in callers_by_workflow(model) {
        writeln!(body, "- {line}")?;
    }
    body.push_str("\n[Release pipelines](@/releases/pipelines.md) · [When a lane fails](@/stroggforge/troubleshooting.md)\n\n## Boundaries\n\nStroggForge manufactures Rust components. Mod Template provides static project/docs presentation. St4sh (with Cod3x) and the Mod Template call only `createRelease` to refresh the release their static site uploads to; they use no Rust manufacturing, and appearing in this inventory makes nothing a consumer. Libraries bypass binary packaging and use the crate publication lane.\n");
    page(
        site,
        "engineering/supply-chain",
        "Supply chain",
        "Source to consumers, with actual workflow responsibilities.",
        &body,
    )?;
    let body = format!(
        "## Binary release\n\nA caller decides which branches/tags/events invoke the reusable workflow. Main verification, PR artifacts, branch development releases and tagged publication have different effects. This is the exact current `needs` graph; consult the contract page for every `if` guard.\n{}\n[Binary contract and conditions](@/stroggforge/workflows/rustGlobalBuild.md).\n\n## Library release\n\nQuality gates precede `release_cleanup`; library publishing and release material follow their actual job prerequisites. No binary signing/packaging is implied.\n{}\n[Library contract and conditions](@/stroggforge/workflows/libGlobalBuild.md).\n\n## Trigger boundaries\n\n- PR: quality/build checks; no release-side signing, scanning or release upload.\n- Main push: quality checks; configured rustdoc Pages deployment. Non-tag releases use the rolling `development` boundary.\n- Tag: configured crates.io publication; named GitHub Release. Version selection remains the consumer's responsibility.\n- AUR: configured push path after platform success; not restricted to tags.\n- Nexus: configured non-PR path after platform success; secret pair/group IDs required.\n- Android/PortMaster: opt-in native artifacts, not APKs or an automatic PortMaster catalog submission.\n\n[Publishing and recovery](@/releases/publishing.md) · [Supply-chain stages](@/engineering/supply-chain.md).\n",
        diagram("workflow-rustGlobalBuild"),
        diagram("workflow-libGlobalBuild")
    );
    page(
        site,
        "releases/pipelines",
        "Release pipelines",
        "From Git state to artifacts, with conditional lanes intact.",
        &body,
    )
}
