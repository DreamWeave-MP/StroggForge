//! Mermaid source for every diagram on the site. Rendered to SVG at build time.

use std::fmt::Write as _;

use anyhow::Result;

use crate::{
    model::{Domain, Kind, Model, Project, RelationshipKind, RetiredKind, Successor, label},
    workflows::Workflow,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Every current project and relationship, including StroggForge callers.
    Full,
    /// The homepage map: relationships without the StroggForge caller fan-in.
    Compact,
    /// Rust package dependencies between current Rust projects.
    Rust,
}

/// Escape text for a quoted Mermaid label.
pub fn quoted(text: &str) -> String {
    text.replace('&', "#38;")
        .replace('"', "#34;")
        .replace('<', "#60;")
        .replace('>', "#62;")
        .replace('[', "#91;")
        .replace(']', "#93;")
        .replace('|', "#124;")
        .replace(['\n', '\r'], " ")
}

fn node(id: &str) -> String {
    format!("n_{}", id.replace('-', "_"))
}

pub fn ecosystem(model: &Model, view: View) -> Result<String> {
    let included = |project: &Project| match view {
        View::Rust => project.domain == Domain::Rust && !project.packages.is_empty(),
        View::Full | View::Compact => true,
    };
    let projects: Vec<&Project> = model.current_projects().filter(|p| included(p)).collect();
    let selected = |id: &str| projects.iter().any(|project| project.id == id);

    let mut graph = String::from("flowchart LR\n");
    for domain in Domain::ALL {
        let in_domain: Vec<&&Project> = projects
            .iter()
            .filter(|project| project.domain == domain)
            .collect();
        if in_domain.is_empty() {
            continue;
        }
        let nested = view != View::Rust;
        let indent = if nested { "    " } else { "  " };
        let domain_id = label(&domain).replace('-', "_");
        if nested {
            writeln!(
                graph,
                "  subgraph domain_{domain_id}[\"{}\"]",
                domain.title().to_uppercase()
            )?;
        }
        for kind in Kind::ALL {
            let members: Vec<_> = in_domain.iter().filter(|p| p.kind == kind).collect();
            if members.is_empty() {
                continue;
            }
            let kind_label = label(&kind);
            writeln!(
                graph,
                "{indent}subgraph kind_{domain_id}_{}[\"{}\"]",
                kind_label.replace('-', "_"),
                kind_label.replace('-', " ")
            )?;
            for project in members {
                writeln!(
                    graph,
                    "{indent}  {}[\"{}\"]",
                    node(&project.id),
                    quoted(&project.name)
                )?;
            }
            writeln!(graph, "{indent}end")?;
        }
        if nested {
            graph.push_str("  end\n");
        }
    }

    for relationship in &model.ecosystem.relationships {
        if !selected(&relationship.from)
            || !selected(&relationship.to)
            || (view == View::Rust && relationship.kind != RelationshipKind::Rust)
        {
            continue;
        }
        let arrow = match relationship.kind {
            RelationshipKind::Rust => "-->",
            RelationshipKind::Documentation => "-.->",
            RelationshipKind::Contains | RelationshipKind::Successor => "==>",
        };
        let text = if view == View::Compact {
            label(&relationship.kind)
        } else {
            relationship.label.clone()
        };
        writeln!(
            graph,
            "  {} {arrow}|\"{}\"| {}",
            node(&relationship.from),
            quoted(&text),
            node(&relationship.to)
        )?;
    }
    if view == View::Full {
        for project in &projects {
            for workflow in &project.stroggforge {
                writeln!(
                    graph,
                    "  {} -.->|\"{}\"| {}",
                    node(&project.id),
                    quoted(&format!("calls {workflow}")),
                    node("stroggforge")
                )?;
            }
        }
    }
    Ok(graph)
}

/// Superseded components and retired infrastructure, each pointing at what replaced it.
pub fn history(model: &Model) -> Result<String> {
    let mut graph = String::from("flowchart LR\n");
    let mut successors = Vec::new();
    let mut edges = Vec::new();

    graph.push_str("  subgraph superseded[\"SUPERSEDED COMPONENTS\"]\n");
    for project in model
        .ecosystem
        .projects
        .iter()
        .filter(|project| project.lifecycle.historical())
    {
        writeln!(
            graph,
            "    {}[\"{}\"]",
            node(&project.id),
            quoted(&project.name)
        )?;
    }
    graph.push_str("  end\n");
    for relationship in &model.ecosystem.relationships {
        if relationship.kind == RelationshipKind::Successor {
            let target = node(&relationship.to);
            successors.push((target.clone(), model.project(&relationship.to).name.clone()));
            edges.push((node(&relationship.from), target));
        }
    }

    for (kind, title) in [
        (RetiredKind::BuildEnvironment, "RETIRED BUILD ENVIRONMENTS"),
        (RetiredKind::Site, "RETIRED SITES"),
    ] {
        writeln!(
            graph,
            "  subgraph retired_{}[\"{title}\"]",
            label(&kind).replace('-', "_")
        )?;
        for retired in model.ecosystem.retired.iter().filter(|r| r.kind == kind) {
            let id = format!("r_{}", retired.id.replace('-', "_"));
            writeln!(graph, "    {id}[\"{}\"]", quoted(&retired.name))?;
            match &retired.successor {
                None => {}
                Some(Successor::Project(project)) => {
                    successors.push((node(project), model.project(project).name.clone()));
                    edges.push((id, node(project)));
                }
                Some(Successor::Platform(platform)) => {
                    let target = format!("t_{}", platform.replace('-', "_"));
                    successors.push((
                        target.clone(),
                        format!("{} platform", model.platform(platform).name),
                    ));
                    edges.push((id, target));
                }
            }
        }
        graph.push_str("  end\n");
    }

    successors.sort();
    successors.dedup();
    graph.push_str("  subgraph current[\"CURRENT\"]\n");
    for (id, name) in &successors {
        writeln!(graph, "    {id}[\"{}\"]", quoted(name))?;
    }
    graph.push_str("  end\n");
    for (from, to) in edges {
        writeln!(graph, "  {from} ==>|\"superseded by\"| {to}")?;
    }
    Ok(graph)
}

/// A workflow's jobs and their `needs` edges.
pub fn workflow(workflow: &Workflow) -> Result<String> {
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
