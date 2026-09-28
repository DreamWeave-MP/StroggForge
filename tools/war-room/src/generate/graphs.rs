use std::{collections::BTreeSet, fmt::Write as _, path::Path};

use anyhow::Result;

use super::{cell, diagram, page, pipelines, project_link, write};
use crate::model::{Domain, Kind, Model, Relationship, label};

pub(super) fn quoted(text: &str) -> String {
    text.replace('&', "#38;")
        .replace('"', "#34;")
        .replace('<', "#60;")
        .replace('>', "#62;")
        .replace('[', "#91;")
        .replace(']', "#93;")
        .replace('|', "#124;")
        .replace(['\n', '\r'], " ")
}

pub(super) fn node(id: &str) -> String {
    format!("p_{}", id.replace('-', "_"))
}

pub fn build(site: &Path, model: &Model) -> Result<()> {
    for (id, rust_only, compact) in [
        ("ecosystem", false, false),
        ("rust-dependencies", true, false),
        ("compact", false, true),
    ] {
        let graph = ecosystem(model, rust_only, compact)?;
        write(site, &format!("static/generated/maps/{id}.mmd"), &graph)?;
    }
    let mut body = String::from(
        "## Current topology\n\nArrow direction is **consumer → dependency**, except containment/publication and historical successor edges, which carry explicit labels. Solid Rust edges are derived from captured Cargo metadata; dashed build edges from actual reusable workflow callers. Repository-level CI integration is labelled as such on workspace components.\n\nOuter boxes are **domains**: the Rust ecosystem that StroggForge serves, the Lua(u)/OpenMW content side, and the web/documentation sites. A dashed edge across domains is a CI call to a StroggForge helper, not membership in the Rust ecosystem: St4sh's site workflow calls only `createRelease`.\n\n**Legend:** solid = Rust dependency; dashed = build/release infrastructure or documentation input; thick = distribution/containment/successor.\n",
    );
    body.push_str(&diagram("ecosystem"));
    body.push_str("## Relationship ledger\n\nThis table is the accessible, searchable equivalent of the graph. Version requirements describe the consuming manifest, not necessarily the newest source version.\n\n| From | To | Type | Meaning | Evidence |\n|---|---|---|---|---|\n");
    for edge in &model.edges {
        if !model.project(&edge.from).lifecycle.historical()
            && !model.project(&edge.to).lifecycle.historical()
        {
            writeln!(
                body,
                "| {} | {} | {} | {} | {} |",
                project_link(model.project(&edge.from)),
                project_link(model.project(&edge.to)),
                label(&edge.kind),
                cell(&edge.label),
                cell(&edge.evidence)
            )?;
        }
    }
    page(
        site,
        "ecosystem/map",
        "Ecosystem map",
        "Verified relationships, generated from the same inventory as the directory.",
        &body,
    )
}

fn ecosystem(model: &Model, rust_only: bool, compact: bool) -> Result<String> {
    let selected: BTreeSet<_> = model
        .inventory
        .projects
        .iter()
        .filter(|project| {
            !project.lifecycle.historical() && (!rust_only || !project.packages.is_empty())
        })
        .map(|project| project.id.as_str())
        .collect();
    let mut graph = String::from("flowchart LR\n");
    // Domain first, so the Rust side, the Lua(u)/OpenMW side and the web sites never blur.
    // The Rust-only view has a single domain and skips the outer box.
    for domain in Domain::ALL {
        let in_domain: Vec<_> = model
            .inventory
            .projects
            .iter()
            .filter(|project| selected.contains(project.id.as_str()) && project.domain == domain)
            .collect();
        if in_domain.is_empty() {
            continue;
        }
        let domain_id = label(&domain).replace('-', "_");
        let indent = if rust_only { "  " } else { "    " };
        if !rust_only {
            writeln!(
                graph,
                "  subgraph domain_{domain_id}[\"{}\"]",
                quoted(&domain.title().to_uppercase())
            )?;
        }
        for kind in [
            Kind::Foundation,
            Kind::Application,
            Kind::DeveloperTool,
            Kind::Runtime,
            Kind::Infrastructure,
            Kind::Documentation,
            Kind::Content,
        ] {
            let projects: Vec<_> = in_domain
                .iter()
                .filter(|project| project.kind == kind)
                .collect();
            if projects.is_empty() {
                continue;
            }
            writeln!(
                graph,
                "{indent}subgraph group_{domain_id}_{}[\"{}\"]",
                label(&kind).replace('-', "_"),
                label(&kind).replace('-', " ")
            )?;
            for project in projects {
                writeln!(
                    graph,
                    "{indent}  {}[\"{}\"]",
                    node(&project.id),
                    quoted(&project.name)
                )?;
            }
            writeln!(graph, "{indent}end")?;
        }
        if !rust_only {
            graph.push_str("  end\n");
        }
    }
    let mut compact_edges = BTreeSet::new();
    for edge in &model.edges {
        if !selected.contains(edge.from.as_str())
            || !selected.contains(edge.to.as_str())
            || (rust_only && edge.kind != Relationship::Rust)
            || (compact && edge.kind == Relationship::Build)
        {
            continue;
        }
        if compact && !compact_edges.insert((&edge.from, &edge.to, edge.kind)) {
            continue;
        }
        let arrow = match edge.kind {
            Relationship::Build | Relationship::Documentation => "-.->",
            Relationship::Successor | Relationship::Distribution | Relationship::Contains => "==>",
            Relationship::Rust | Relationship::Runtime => "-->",
        };
        let edge_label = if compact {
            label(&edge.kind)
        } else {
            format!("{}: {}", label(&edge.kind), edge.label)
        };
        writeln!(
            graph,
            "  {} {}|\"{}\"| {}",
            node(&edge.from),
            arrow,
            quoted(&edge_label),
            node(&edge.to)
        )?;
    }
    Ok(graph)
}

pub fn workflow_graph(workflow: &crate::workflows::Workflow) -> Result<String> {
    let mut graph = String::from("flowchart TD\n");
    for name in workflow.jobs.keys() {
        writeln!(graph, "  {}[\"{}\"]", node(name), quoted(name))?;
    }
    for (name, job) in &workflow.jobs {
        for dependency in job.dependencies()? {
            writeln!(graph, "  {} --> {}", node(&dependency), node(name))?;
        }
    }
    Ok(graph)
}

/// A compact pipeline: one node per stage naming the workflows that take part, ending in the
/// verified callers per workflow. Job-level detail lives in the stage table beside it.
pub fn supply_chain_graph(model: &Model) -> Result<String> {
    let mut graph = String::from("flowchart TB\n");
    let mut previous: Option<String> = None;
    for (index, stage) in model.operations.stage.iter().enumerate() {
        let id = format!("stage_{}", stage.id.replace('-', "_"));
        let workflows = pipelines::stage_groups(model, &stage.jobs)?
            .into_iter()
            .map(|(workflow, _)| workflow)
            .collect::<Vec<_>>();
        let detail = if workflows.is_empty() {
            "caller workflow at a pinned ref".to_owned()
        } else {
            workflows.join(" · ")
        };
        writeln!(
            graph,
            "  {id}[\"{}<br>{}\"]",
            quoted(&format!("{} · {}", index + 1, stage.name.to_uppercase())),
            quoted(&detail)
        )?;
        if let Some(previous) = &previous {
            writeln!(graph, "  {previous} --> {id}")?;
        }
        previous = Some(id);
    }
    let callers = model
        .callers
        .iter()
        .map(|(workflow, projects)| quoted(&format!("{workflow}: {}", projects.len())))
        .collect::<Vec<_>>()
        .join("<br>");
    writeln!(graph, "  consumers[\"CONSUMERS<br>{callers}\"]")?;
    if let Some(previous) = previous {
        writeln!(graph, "  {previous} --> consumers")?;
    }
    Ok(graph)
}
