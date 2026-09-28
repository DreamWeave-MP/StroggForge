use std::{collections::BTreeSet, fmt::Write as _, path::Path};

use anyhow::Result;

use super::{
    cell, diagram,
    graphs::{node, quoted},
    page, project_link, section, write,
};
use crate::model::{Model, Relationship, RetiredKind, Successor, label};

const COLUMNS: usize = 4;
const COMMITS: &str = "https://github.com/DreamWeave-MP/StroggForge/commit/";

pub fn build(site: &Path, model: &Model) -> Result<()> {
    write(
        site,
        "static/generated/maps/history.mmd",
        &history_graph(model)?,
    )?;
    let mut body = String::from(
        "Current and retired infrastructure never share the current topology. A **superseded by** edge exists only where a migration has evidence; an archived repository without a reviewed successor stays unconnected rather than guessed at. Successor destinations appear here only to explain the migration.\n",
    );
    body.push_str(&diagram("history"));
    body.push_str("## Superseded components\n\nComponents whose source still exists but whose identity has been replaced.\n\n| Component | Superseded by | Evidence |\n|---|---|---|\n");
    for edge in model
        .edges
        .iter()
        .filter(|edge| edge.kind == Relationship::Successor)
    {
        writeln!(
            body,
            "| {} | {} | {} |",
            project_link(model.project(&edge.from)),
            project_link(model.project(&edge.to)),
            cell(&edge.evidence)
        )?;
    }
    body.push_str("\n## Retired infrastructure\n\nRepositories, build environments and sites with no current checkout. Records live in the `[[retired]]` section of `war-room/ecosystem.toml`.\n\n| Retired | Kind | Superseded by | What it was | Evidence |\n|---|---|---|---|---|\n");
    for retired in &model.inventory.retired {
        writeln!(
            body,
            "| [{}](@/archaeology/retired/{}.md) | {} | {} | {} | {} |",
            cell(&retired.name),
            retired.id,
            label(&retired.kind),
            successor_link(model, retired.successor.as_ref()),
            cell(&retired.summary),
            cell(&retired.evidence)
        )?;
    }
    retired_pages(site, model)?;
    census(site, model)?;
    body.push_str("\n## Why things changed\n\nThe [infrastructure change log](@/releases/changes.md) records each reviewed migration with its commits. The [maintainer notebooks](@/archaeology/notebooks.md) hold the older narrative material and say which parts are still current.\n");
    page(
        site,
        "archaeology/history",
        "History and migrations",
        "Useful history without making retired infrastructure look current.",
        &body,
    )?;
    let mut changes = String::from(
        "A reviewed digest of infrastructure changes that alter what consumers get, newest first. It comes from the `[[updates]]` records in `war-room/ecosystem.toml`. The [commit history](https://github.com/DreamWeave-MP/StroggForge/commits/main/) is the complete record; a consumer pinned to an older tag keeps that tag's behavior.\n\n",
    );
    changes.push_str(&update_table(model, usize::MAX));
    page(
        site,
        "releases/changes",
        "Infrastructure change log",
        "What changed in the supply line, when, and in which commits.",
        &changes,
    )
}

/// The newest `limit` reviewed updates as a Markdown table.
pub fn update_table(model: &Model, limit: usize) -> String {
    let mut table = String::from("| Date | Change | Detail | Commits |\n|---|---|---|---|\n");
    for update in model.inventory.updates.iter().take(limit) {
        let commits = update
            .commits
            .iter()
            .map(|commit| format!("[`{commit}`]({COMMITS}{commit})"))
            .collect::<Vec<_>>()
            .join(" ");
        let plan = update.plan.as_ref().map_or(String::new(), |plan| {
            format!(" [Campaign](@/releases/plans/{plan}.md).")
        });
        writeln!(
            table,
            "| {} | **{}** | {}{} | {} |",
            update.date,
            cell(&update.title),
            cell(&update.detail),
            plan,
            commits
        )
        .expect("writing to a String cannot fail");
    }
    table
}

fn successor_link(model: &Model, successor: Option<&Successor>) -> String {
    match successor {
        None => "no reviewed successor".to_owned(),
        Some(Successor::Project(id)) => project_link(model.project(id)),
        Some(Successor::Platform(id)) => format!(
            "[{}](@/releases/platforms.md) platform",
            cell(&model.platform(id).name)
        ),
    }
}

fn history_graph(model: &Model) -> Result<String> {
    let mut graph = String::from("flowchart LR\n");
    let superseded: Vec<_> = model
        .inventory
        .projects
        .iter()
        .filter(|project| project.lifecycle.historical())
        .collect();
    if !superseded.is_empty() {
        graph.push_str("  subgraph superseded[\"SUPERSEDED COMPONENTS\"]\n");
        for project in &superseded {
            writeln!(
                graph,
                "    {}[\"{}\"]",
                node(&project.id),
                quoted(&project.name)
            )?;
        }
        graph.push_str("  end\n");
    }
    retired_groups(&mut graph, model)?;
    let mut successors = BTreeSet::new();
    let mut edges = Vec::new();
    for edge in model
        .edges
        .iter()
        .filter(|edge| edge.kind == Relationship::Successor)
    {
        successors.insert(Successor::Project(edge.to.clone()));
        edges.push((
            node(&edge.from),
            successor_node(&Successor::Project(edge.to.clone())),
        ));
    }
    for retired in &model.inventory.retired {
        if let Some(successor) = &retired.successor {
            successors.insert(successor.clone());
            edges.push((retired_node(&retired.id), successor_node(successor)));
        }
    }
    if !successors.is_empty() {
        graph.push_str("  subgraph current[\"CURRENT SUCCESSORS\"]\n");
        for successor in &successors {
            let name = match successor {
                Successor::Project(id) => model.project(id).name.clone(),
                Successor::Platform(id) => format!("{} platform", model.platform(id).name),
            };
            writeln!(
                graph,
                "    {}[\"{}\"]",
                successor_node(successor),
                quoted(&name)
            )?;
        }
        graph.push_str("  end\n");
    }
    for (from, to) in edges {
        writeln!(graph, "  {from} ==>|\"superseded by\"| {to}")?;
    }
    Ok(graph)
}

/// One subgraph per retired kind. Unconnected records would otherwise form one unreadable
/// row, so large groups are stacked into columns with invisible links.
fn retired_groups(graph: &mut String, model: &Model) -> Result<()> {
    for (kind, title) in [
        (RetiredKind::BuildEnvironment, "RETIRED BUILD ENVIRONMENTS"),
        (RetiredKind::Site, "RETIRED SITES"),
        (RetiredKind::Repository, "RETIRED REPOSITORIES"),
        (RetiredKind::Organization, "ABANDONED ORGANIZATIONS"),
    ] {
        let records: Vec<_> = model
            .inventory
            .retired
            .iter()
            .filter(|retired| retired.kind == kind)
            .collect();
        if records.is_empty() {
            continue;
        }
        writeln!(
            graph,
            "  subgraph retired_{}[\"{title}\"]",
            label(&kind).replace('-', "_")
        )?;
        if records.len() > COLUMNS {
            graph.push_str("    direction TB\n");
        }
        for retired in &records {
            writeln!(
                graph,
                "    {}[\"{}\"]",
                retired_node(&retired.id),
                quoted(&retired.name)
            )?;
        }
        if records.len() > COLUMNS {
            for column in 0..COLUMNS {
                let chain: Vec<String> = records
                    .iter()
                    .skip(column)
                    .step_by(COLUMNS)
                    .map(|retired| retired_node(&retired.id))
                    .collect();
                if chain.len() > 1 {
                    writeln!(graph, "    {}", chain.join(" ~~~ "))?;
                }
            }
        }
        graph.push_str("  end\n");
    }
    Ok(())
}

fn retired_node(id: &str) -> String {
    format!("r_{}", id.replace('-', "_"))
}

fn successor_node(successor: &Successor) -> String {
    match successor {
        Successor::Project(id) => node(id),
        Successor::Platform(id) => format!("t_{}", id.replace('-', "_")),
    }
}

/// Shown at the top of every retired record and superseded component page.
pub fn unsupported_notice(successor: &str) -> String {
    format!(
        "<div class=\"retired-notice\" role=\"note\"><strong>No longer maintained or supported.</strong> Do not build new work on it, file issues against it, or treat it as a current dependency. {successor}</div>\n\n"
    )
}

fn retired_pages(site: &Path, model: &Model) -> Result<()> {
    section(
        site,
        "archaeology/retired",
        "Retired infrastructure",
        20,
        "Everything here is **no longer maintained or supported**. Each record says what it was, where it lived, what replaced it if anything, and the evidence for its retirement.\n",
    )?;
    for retired in &model.inventory.retired {
        let successor = match &retired.successor {
            None => "No successor has been reviewed.".to_owned(),
            Some(successor) => format!("Use {} instead.", successor_link(model, Some(successor))),
        };
        let body = format!(
            "{}**{}** · retired\n\n{}\n\n| Field | Value |\n|---|---|\n| Location | <{}> |\n| Superseded by | {} |\n| Evidence | {} |\n\n[All retired infrastructure](@/archaeology/history.md#retired-infrastructure) · [Organization census](@/ecosystem/census.md)\n",
            unsupported_notice(&successor),
            label(&retired.kind),
            retired.summary,
            retired.location,
            successor_link(model, retired.successor.as_ref()),
            cell(&retired.evidence)
        );
        page(
            site,
            &format!("archaeology/retired/{}", retired.id),
            &retired.name,
            &format!(
                "Retired {} · no longer maintained",
                label(&retired.kind).replace('-', " ")
            ),
            &body,
        )?;
    }
    Ok(())
}

/// Every organization repository, and whether the war room documents it.
fn census(site: &Path, model: &Model) -> Result<()> {
    let census = &model.organization;
    let mut inventoried = String::new();
    let mut retired = String::new();
    let mut unreviewed = String::new();
    let mut counts = [0_usize; 3];
    for repository in &census.repositories {
        let key = repository.url.to_lowercase();
        let projects: Vec<_> = model
            .inventory
            .projects
            .iter()
            .filter(|project| model.repository(project).to_lowercase() == key)
            .collect();
        let record = model
            .inventory
            .retired
            .iter()
            .find(|record| record.location.to_lowercase() == key);
        let facts = format!(
            "{}{}",
            if repository.fork { "fork · " } else { "" },
            if repository.archived {
                "archived · "
            } else {
                ""
            }
        );
        let name = format!("[{}]({})", cell(&repository.name), repository.url);
        if let Some(record) = record {
            counts[1] += 1;
            writeln!(
                retired,
                "| {name} | [No longer maintained](@/archaeology/retired/{}.md) | {facts}{} | {} |",
                record.id,
                repository.pushed,
                cell(&record.summary)
            )?;
        } else if !projects.is_empty() {
            counts[0] += 1;
            writeln!(
                inventoried,
                "| {name} | {} | {facts}{} | {} |",
                projects
                    .iter()
                    .map(|project| format!(
                        "{} ({})",
                        project_link(project),
                        label(&project.lifecycle)
                    ))
                    .collect::<Vec<_>>()
                    .join(", "),
                repository.pushed,
                cell(&projects[0].summary)
            )?;
        } else {
            counts[2] += 1;
            writeln!(
                unreviewed,
                "| {name} | **not reviewed** | {facts}{} | {} |",
                repository.pushed,
                cell(if repository.description.is_empty() {
                    "No description on GitHub."
                } else {
                    &repository.description
                })
            )?;
        }
    }
    let header = "| Repository | Status | Last push | Summary |\n|---|---|---|---|\n";
    let body = format!(
        "Every repository in the [{organization}](https://github.com/{organization}) GitHub organization as observed on **{observed}**: {total} repositories, {inventoried_count} documented in the inventory, {retired_count} retired, {unreviewed_count} not yet reviewed. Captured with `capture-organization` into `war-room/sources/organization.json`; builds never query GitHub.\n\nValidation fails if GitHub reports a repository archived and it has no retired record, or if a current project lives in an archived repository. A repository that is dead but not archived needs a maintainer decision recorded as a `[[retired]]` entry.\n\n## Retired: no longer maintained or supported\n\n{header}{retired}\n## Documented in the inventory\n\n{header}{inventoried}\n## Not yet reviewed\n\nThese exist in the organization but have no inventory decision. **Not reviewed does not mean current**: check the last push date, and ask a maintainer before depending on one. Summaries here are GitHub's own descriptions.\n\n{header}{unreviewed}",
        organization = census.name,
        observed = census.observed,
        total = census.repositories.len(),
        inventoried_count = counts[0],
        retired_count = counts[1],
        unreviewed_count = counts[2],
    );
    page(
        site,
        "ecosystem/census",
        "Organization census",
        "Every repository in the organization: documented, retired, or not yet reviewed.",
        &body,
    )
}
