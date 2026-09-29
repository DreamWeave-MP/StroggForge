//! Turns the validated model into Zola content under `site/content` and diagram sources
//! under `site/static/generated`. Both directories are disposable and gitignored.

use std::{collections::BTreeMap, fmt::Write as _, fs, path::Path};

use anyhow::{Context, Result};
use serde::Serialize;
use serde_yaml_ng::Value;

use crate::{
    graphs::{self, View},
    model::{
        Domain, Kind, Model, PlanKind, Project, RelationshipKind, RequirementKind, State,
        Successor, TaskState, Validation, label,
    },
    workflows::{Note, Workflow, text},
};

pub fn build(root: &Path, model: &Model) -> Result<()> {
    let site = root.join("site");
    for directory in ["content", "static/generated"] {
        let directory = site.join(directory);
        if directory.exists() {
            fs::remove_dir_all(&directory)?;
        }
        fs::create_dir_all(&directory)?;
    }
    copy_tree(&site.join("pages"), &site.join("content"))?;
    copy_tree(&site.join("maps"), &site.join("static/generated/maps"))?;

    let site = Site(site);
    site.map("ecosystem", &graphs::ecosystem(model, View::Full)?)?;
    site.map("compact", &graphs::ecosystem(model, View::Compact)?)?;
    site.map("rust-dependencies", &graphs::ecosystem(model, View::Rust)?)?;
    site.map("history", &graphs::history(model)?)?;
    supply_line(&site, model)?;

    summary(&site, model)?;
    homepage(&site, model)?;
    ecosystem_pages(&site, model)?;
    history_pages(&site, model)?;
    release_pages(&site, model)?;
    platforms(&site, model)?;
    workflow_pages(&site, root, model)?;
    Ok(())
}

fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    let mut entries = fs::read_dir(source)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort();
    for entry in entries {
        let destination = target.join(entry.file_name().context("directory entry has a name")?);
        if entry.is_dir() {
            copy_tree(&entry, &destination)?;
        } else {
            fs::copy(&entry, &destination).with_context(|| format!("copy {}", entry.display()))?;
        }
    }
    Ok(())
}

struct Site(std::path::PathBuf);

impl Site {
    fn write(&self, path: &str, text: &str) -> Result<()> {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().context("generated path has a parent")?)?;
        fs::write(&path, text).with_context(|| format!("write {}", path.display()))
    }

    fn map(&self, name: &str, source: &str) -> Result<()> {
        self.write(&format!("static/generated/maps/{name}.mmd"), source)
    }

    fn page(&self, path: &str, title: &str, description: &str, body: &str) -> Result<()> {
        self.write(
            &format!("content/{path}.md"),
            &format!(
                "+++\ntitle = {}\ndescription = {}\nweight = 10\n+++\n\n{body}",
                serde_json::to_string(title)?,
                serde_json::to_string(description)?
            ),
        )
    }

    fn section(&self, path: &str, title: &str, weight: u32, body: &str) -> Result<()> {
        self.write(
            &format!("content/{path}/_index.md"),
            &format!(
                "+++\ntitle = {}\nsort_by = \"title\"\nweight = {weight}\ntemplate = \"docs/section.html\"\npage_template = \"docs/page.html\"\n+++\n\n{body}",
                serde_json::to_string(title)?
            ),
        )
    }
}

/// Data for the imported `schematic` shortcode (see `templates/shortcodes/schematic.html`).
#[derive(Serialize)]
struct Schematic {
    id: String,
    mode: &'static str,
    title: String,
    heading_level: u8,
    description: String,
    direction: &'static str,
    nodes: Vec<SchematicNode>,
    edges: Vec<SchematicEdge>,
}

#[derive(Serialize)]
struct SchematicNode {
    id: String,
    title: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    links: Vec<SchematicLink>,
}

#[derive(Serialize)]
struct SchematicLink {
    label: String,
    page: String,
}

#[derive(Serialize)]
struct SchematicEdge {
    from: String,
    to: String,
}

/// The supply line twice: in full for the supply-chain page, and names-only for the homepage.
fn supply_line(site: &Site, model: &Model) -> Result<()> {
    for (file, compact) in [("supply-line", false), ("supply-line-compact", true)] {
        let mut nodes = Vec::new();
        for stage in &model.operations.stage {
            let mut workflows: Vec<&str> = Vec::new();
            for reference in &stage.jobs {
                let (workflow, _) = reference.split_once('/').unwrap_or((reference, ""));
                if !workflows.contains(&workflow) {
                    workflows.push(workflow);
                }
            }
            nodes.push(SchematicNode {
                id: stage.id.clone(),
                title: stage.name.clone(),
                kind: label(&stage.kind),
                detail: (!compact).then(|| stage.summary.clone()),
                links: if compact {
                    Vec::new()
                } else {
                    workflows
                        .iter()
                        .map(|workflow| SchematicLink {
                            label: (*workflow).to_owned(),
                            page: format!("@/stroggforge/workflows/{workflow}.md"),
                        })
                        .collect()
                },
            });
        }
        let callers = model.callers();
        nodes.push(SchematicNode {
            id: "callers".into(),
            title: "Callers".into(),
            kind: "result".into(),
            detail: Some(format!(
                "{} projects call StroggForge",
                model
                    .current_projects()
                    .filter(|project| !project.stroggforge.is_empty())
                    .count()
            )),
            links: callers
                .iter()
                .map(|(workflow, projects)| SchematicLink {
                    label: format!("{workflow}: {}", projects.len()),
                    page: "@/stroggforge/integration.md".into(),
                })
                .collect(),
        });
        let edges = nodes
            .windows(2)
            .map(|pair| SchematicEdge {
                from: pair[0].id.clone(),
                to: pair[1].id.clone(),
            })
            .collect();
        let schematic = Schematic {
            id: file.into(),
            mode: "flow",
            title: "From a commit to the people running it".into(),
            heading_level: 3,
            description: "Stages are ordered by the earliest point each can start; jobs overlap where the workflows' needs allow.".into(),
            direction: if compact { "horizontal" } else { "vertical" },
            nodes,
            edges,
        };
        site.write(
            &format!("static/generated/schematics/{file}.json"),
            &format!("{}\n", serde_json::to_string_pretty(&schematic)?),
        )?;
    }
    Ok(())
}

/// Escape text for a Markdown table cell.
fn cell(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "&#124;")
        .replace(['\n', '\r'], " ")
}

fn diagram(name: &str) -> String {
    format!("\n{{{{ diagram(name=\"{name}\") }}}}\n\n")
}

fn project_link(project: &Project) -> String {
    let lane = if project.lifecycle.historical() {
        "archaeology/projects"
    } else {
        "ecosystem/projects"
    };
    format!("[{}](@/{lane}/{}.md)", cell(&project.name), project.id)
}

fn workflow_link(id: &str) -> String {
    format!("[{id}](@/stroggforge/workflows/{id}.md)")
}

fn labels<T: Serialize>(values: &[T]) -> String {
    values.iter().map(label).collect::<Vec<_>>().join(", ")
}

const UNSUPPORTED: &str = "<div class=\"retired-notice\" role=\"note\"><strong>No longer maintained or supported.</strong> Do not build new work on it or treat it as a current dependency.</div>\n\n";

/// The numbers in the homepage pulse strip, rendered by the `pulse` shortcode.
#[derive(Serialize)]
struct Summary<'a> {
    reviewed: &'a str,
    components: usize,
    open_campaigns: usize,
    planned_releases: usize,
    blocked_plans: usize,
    callers: usize,
    platforms: usize,
    validated_platforms: usize,
}

fn summary(site: &Site, model: &Model) -> Result<()> {
    let open = |kind: PlanKind| {
        model
            .plans
            .iter()
            .filter(|plan| plan.kind == kind && !plan.state.finished())
            .count()
    };
    let summary = Summary {
        reviewed: &model.ecosystem.reviewed,
        components: model.current_projects().count(),
        open_campaigns: open(PlanKind::Campaign),
        planned_releases: open(PlanKind::Release),
        blocked_plans: model
            .plans
            .iter()
            .filter(|plan| plan.state == State::Blocked)
            .count(),
        callers: model
            .current_projects()
            .filter(|project| !project.stroggforge.is_empty())
            .count(),
        platforms: model.toolchains.platforms.len(),
        validated_platforms: model
            .toolchains
            .platforms
            .iter()
            .filter(|platform| platform.validation == Validation::Ready)
            .count(),
    };
    site.write(
        "static/generated/summary.json",
        &format!("{}\n", serde_json::to_string_pretty(&summary)?),
    )
}

fn homepage(site: &Site, model: &Model) -> Result<()> {
    let mut body = String::from(
        "+++\ntitle = \"War Room\"\ndescription = \"DreamWeave engineering · source, dependencies, releases, and the supply line.\"\ntemplate = \"war-room/home.html\"\npage_template = \"docs/page.html\"\nsort_by = \"weight\"\n[extra]\ndocs_root = true\ndocs_project_name = \"StroggForge\"\ndocs_short_title = \"DreamWeave War Room\"\ndocs_repository_url = \"https://github.com/DreamWeave-MP/StroggForge\"\ndocs_sidebar_label = \"War Room\"\nkind = \"DreamWeave engineering\"\n+++\n\n{{ pulse() }}\n\nEverything here comes from a few TOML files and the workflow YAML. Nothing polls GitHub, so **unknown** means nobody wrote it down, not that it broke.\n\n## Attention required\n\n",
    );
    body.push_str(&blockers(model));
    body.push_str("\n## Campaigns\n\n");
    body.push_str(&campaigns(model));
    body.push_str("\n## Release pulse\n\nApplications and developer tools. A Cargo version is not a release plan; only `war-room/plans.toml` creates one.\n\n");
    body.push_str(&release_pulse(model));
    writeln!(
        body,
        "\n[Release board](@/releases/board.md) · [Plan a release](@/contributing/maintenance.md#planning-a-release)\n\n## Toolchain and targets\n\n**Reference measurement:** {}\n",
        model.toolchains.reference
    )?;
    body.push_str(&target_table(model));
    body.push_str("\n[Platform matrix and intentional differences](@/releases/platforms.md)\n\n## System map\n\nConsumer → dependency. The [full map](@/ecosystem/map.md) adds which projects call StroggForge.\n");
    body.push_str(&diagram("compact"));
    body.push_str("## Supply line\n\n");
    body.push_str("{{ schematic(data_path=\"static/generated/schematics/supply-line-compact.json\") }}\n\n**Who calls it:**\n\n");
    body.push_str(&callers_list(model));
    let exceptional: Vec<String> = model
        .toolchains
        .platforms
        .iter()
        .filter(|platform| platform.job != "release")
        .map(|platform| {
            format!(
                "{} (`{}`, {})",
                platform.name, platform.job, platform.baseline
            )
        })
        .collect();
    writeln!(
        body,
        "\n**Paths outside the hosted runner matrix:** {}.\n\n[Supply-chain stages](@/engineering/supply-chain.md) · [Workflow contracts](@/stroggforge/workflows/_index.md) · [Incident desk](@/stroggforge/troubleshooting.md)\n\n## Recent infrastructure changes\n",
        exceptional.join("; ")
    )?;
    body.push_str(&update_table(model, 4));
    body.push_str("\n[Full change log](@/releases/changes.md) · [Start contributing](@/contributing/_index.md)\n");
    site.write("content/_index.md", &body)
}

fn callers_list(model: &Model) -> String {
    let mut list = String::new();
    for (workflow, projects) in model.callers() {
        let names: Vec<String> = projects
            .iter()
            .map(|project| project_link(project))
            .collect();
        writeln!(list, "- {}: {}", workflow_link(workflow), names.join(", "))
            .expect("writing to a String cannot fail");
    }
    list
}

fn target_table(model: &Model) -> String {
    let mut table = String::from(
        "| Platform | Target | Baseline | Compiler | Validation |\n|---|---|---|---|---|\n",
    );
    for platform in &model.toolchains.platforms {
        writeln!(
            table,
            "| {} | `{}` | {} | {} | **{}** |",
            cell(&platform.name),
            platform.target,
            cell(&platform.baseline),
            cell(&platform.compiler),
            label(&platform.validation)
        )
        .expect("writing to a String cannot fail");
    }
    table
}

fn ecosystem_pages(site: &Site, model: &Model) -> Result<()> {
    site.section(
        "ecosystem/projects",
        "Project directory",
        20,
        "Current components, including experimental and maintenance work. Superseded ones live in [Archaeology](@/archaeology/history.md).\n",
    )?;
    site.section(
        "archaeology/projects",
        "Superseded projects",
        10,
        "Components replaced by a current project. They never appear in the current maps.\n",
    )?;
    for project in &model.ecosystem.projects {
        project_page(site, model, project)?;
    }

    let mut overview = format!(
        "{}\n\nStroggForge builds the first table and nothing else.\n",
        model.ecosystem.scope
    );
    for domain in Domain::ALL {
        writeln!(
            overview,
            "\n## {}\n\n| Project | Role | Lifecycle | Summary |\n|---|---|---|---|",
            domain.title()
        )?;
        for project in model.current_projects().filter(|p| p.domain == domain) {
            writeln!(
                overview,
                "| {} | {} | {} | {} |",
                project_link(project),
                label(&project.kind),
                label(&project.lifecycle),
                cell(&project.summary)
            )?;
        }
    }
    site.page(
        "ecosystem/overview",
        "Ecosystem overview",
        "Every current DreamWeave component, by domain.",
        &overview,
    )?;

    let mut foundations = String::from(
        "The reusable Rust libraries underneath the applications. Arrows point **consumer → dependency**; `(optional)` and `(dev)` mark optional features and development-only dependencies.\n",
    );
    foundations.push_str(&diagram("rust-dependencies"));
    foundations.push_str("| Foundation | Packages | Summary |\n|---|---|---|\n");
    for project in model
        .current_projects()
        .filter(|project| project.kind == Kind::Foundation)
    {
        writeln!(
            foundations,
            "| {} | {} | {} |",
            project_link(project),
            project.packages.join(", "),
            cell(&project.summary)
        )?;
    }
    site.page(
        "ecosystem/foundations",
        "Rust foundations",
        "The reusable libraries underneath the applications.",
        &foundations,
    )?;

    map_page(site, model)?;
    targets_page(site, model)
}

fn map_page(site: &Site, model: &Model) -> Result<()> {
    let mut map = String::from(
        "Outer boxes are **domains**: the Rust ecosystem StroggForge serves, the Lua(u)/OpenMW content side, and the web sites. Arrows point **consumer → dependency**. Solid: Rust dependency. Dashed: documentation input or a call to a StroggForge workflow. Thick: containment or succession. A call across domains is CI usage, not membership in the Rust ecosystem.\n",
    );
    map.push_str(&diagram("ecosystem"));
    map.push_str("## Relationship ledger\n\n| From | To | Type | Meaning |\n|---|---|---|---|\n");
    for relationship in &model.ecosystem.relationships {
        let (from, to) = (
            model.project(&relationship.from),
            model.project(&relationship.to),
        );
        if from.lifecycle.historical() || to.lifecycle.historical() {
            continue;
        }
        writeln!(
            map,
            "| {} | {} | {} | {} |",
            project_link(from),
            project_link(to),
            label(&relationship.kind),
            cell(&relationship.label)
        )?;
    }
    for project in model.current_projects() {
        for workflow in &project.stroggforge {
            writeln!(
                map,
                "| {} | [StroggForge](@/ecosystem/projects/stroggforge.md) | build | calls {} |",
                project_link(project),
                workflow_link(workflow)
            )?;
        }
    }
    site.page(
        "ecosystem/map",
        "Ecosystem map",
        "How the current projects relate.",
        &map,
    )
}

fn targets_page(site: &Site, model: &Model) -> Result<()> {
    let mut targets = String::from(
        "Contribution areas, not a claim that a particular issue is open. Check the repository's issues and contribution guide first.\n\n| Project | Domain | Approach | Useful work |\n|---|---|---|---|\n",
    );
    for project in model.current_projects() {
        writeln!(
            targets,
            "| {} | {} | {} | {} |",
            project_link(project),
            project.domain.title(),
            label(&project.approach),
            cell(&project.contribution_areas.join("; "))
        )?;
    }
    site.page(
        "contributing/targets",
        "Good contribution targets",
        "Find a repository and a meaningful first task.",
        &targets,
    )
}

fn project_page(site: &Site, model: &Model, project: &Project) -> Result<()> {
    let mut body = String::new();
    if project.lifecycle.historical() {
        body.push_str(UNSUPPORTED);
    }
    let repository = project.repository.as_ref().map_or_else(
        || "No public repository yet".to_owned(),
        |url| format!("[Repository]({url}) · [Issues]({url}/issues)"),
    );
    let stroggforge = if project.stroggforge.is_empty() {
        "Does not call StroggForge".to_owned()
    } else {
        project
            .stroggforge
            .iter()
            .map(|workflow| workflow_link(workflow))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let packages = if project.packages.is_empty() {
        "—".to_owned()
    } else {
        project
            .packages
            .iter()
            .map(|package| format!("`{package}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    writeln!(
        body,
        "**{} · {} · {} · {}**\n\n{}\n\n## Field record\n\n| Field | Value |\n|---|---|\n| Repository | {repository} |\n| Language | {} |\n| Stewards | {} |\n| Approach | {} |\n| Packages | {packages} |\n| Distribution | {} |\n| StroggForge | {stroggforge} |\n\n## Documentation\n",
        project.domain.title(),
        label(&project.kind),
        label(&project.lifecycle),
        label(&project.maturity),
        project.summary,
        cell(&project.languages.join(", ")),
        cell(&project.owners.join(", ")),
        label(&project.approach),
        labels(&project.distribution),
    )?;
    if project.docs.is_empty() {
        body.push_str("No public documentation yet.\n");
    }
    for url in &project.docs {
        writeln!(body, "- <{url}>")?;
    }

    body.push_str("\n## Relationships\n\n| Direction | Project | Relationship |\n|---|---|---|\n");
    for relationship in &model.ecosystem.relationships {
        let (direction, other) = if relationship.from == project.id {
            ("uses", &relationship.to)
        } else if relationship.to == project.id {
            ("used by", &relationship.from)
        } else {
            continue;
        };
        writeln!(
            body,
            "| {direction} | {} | {}: {} |",
            project_link(model.project(other)),
            label(&relationship.kind),
            cell(&relationship.label)
        )?;
    }
    if project.id == "stroggforge" {
        for (workflow, callers) in model.callers() {
            for caller in callers {
                writeln!(
                    body,
                    "| used by | {} | build: calls {} |",
                    project_link(caller),
                    workflow_link(workflow)
                )?;
            }
        }
    }

    body.push_str("\n## Contribute\n\n");
    for area in &project.contribution_areas {
        writeln!(body, "- {}", cell(area))?;
    }
    let lane = if project.lifecycle.historical() {
        "archaeology/projects"
    } else {
        "ecosystem/projects"
    };
    site.page(
        &format!("{lane}/{}", project.id),
        &project.name,
        &project.summary,
        &body,
    )
}

fn history_pages(site: &Site, model: &Model) -> Result<()> {
    let mut body = String::from(
        "Retirements that explain the current architecture. Nothing here is maintained or supported, and none of it appears in the current maps.\n",
    );
    body.push_str(&diagram("history"));
    body.push_str(
        "## Superseded components\n\n| Component | Superseded by | Why |\n|---|---|---|\n",
    );
    for relationship in &model.ecosystem.relationships {
        if relationship.kind == RelationshipKind::Successor {
            writeln!(
                body,
                "| {} | {} | {} |",
                project_link(model.project(&relationship.from)),
                project_link(model.project(&relationship.to)),
                cell(&relationship.label)
            )?;
        }
    }
    body.push_str("\n## Retired infrastructure\n\n| Retired | Kind | Superseded by | What it was | Why |\n|---|---|---|---|---|\n");
    for retired in &model.ecosystem.retired {
        let successor = match &retired.successor {
            None => "—".to_owned(),
            Some(Successor::Project(id)) => project_link(model.project(id)),
            Some(Successor::Platform(id)) => format!(
                "[{}](@/releases/platforms.md) platform",
                cell(&model.platform(id).name)
            ),
        };
        writeln!(
            body,
            "| [{}]({}) | {} | {successor} | {} | {} |",
            cell(&retired.name),
            retired.location,
            label(&retired.kind).replace('-', " "),
            cell(&retired.summary),
            cell(&retired.reason)
        )?;
    }
    body.push_str("\nThe [change log](@/releases/changes.md) records each migration with its commits. The [maintainer notebooks](@/archaeology/notebooks.md) say which root documents are still current.\n");
    site.page(
        "archaeology/history",
        "History and migrations",
        "What was replaced, by what, and why.",
        &body,
    )?;

    let mut changes = String::from(
        "Infrastructure changes that alter what consumers get, newest first. The [commit history](https://github.com/DreamWeave-MP/StroggForge/commits/main/) is the complete record; a consumer pinned to an older tag keeps that tag's behavior.\n\n",
    );
    changes.push_str(&update_table(model, usize::MAX));
    site.page(
        "releases/changes",
        "Infrastructure change log",
        "What changed in the supply line, when, and in which commits.",
        &changes,
    )
}

fn update_table(model: &Model, limit: usize) -> String {
    let mut updates: Vec<_> = model.ecosystem.updates.iter().collect();
    updates.sort_by(|a, b| b.date.cmp(&a.date));
    let mut table = String::from("| Date | Change | Detail | Commits |\n|---|---|---|---|\n");
    for update in updates.into_iter().take(limit) {
        let commits: Vec<String> = update
            .commits
            .iter()
            .map(|commit| {
                format!(
                    "[`{commit}`](https://github.com/DreamWeave-MP/StroggForge/commit/{commit})"
                )
            })
            .collect();
        let plan = update.plan.as_ref().map_or(String::new(), |plan| {
            format!(" [Campaign](@/releases/plans/{plan}.md).")
        });
        writeln!(
            table,
            "| {} | **{}** | {}{plan} | {} |",
            update.date,
            cell(&update.title),
            cell(&update.detail),
            commits.join(" ")
        )
        .expect("writing to a String cannot fail");
    }
    table
}

fn release_pages(site: &Site, model: &Model) -> Result<()> {
    site.section(
        "releases/plans",
        "Release and campaign records",
        20,
        "Reviewed intent from `war-room/plans.toml`. A missing version or date means nothing has been committed to.\n",
    )?;
    let mut board = String::from(
        "## Release pulse\n\nA Cargo version is not a release plan. CI and docs stay unknown until a plan records them.\n\n",
    );
    board.push_str(&release_pulse(model));
    board.push_str("\n## Cross-project campaigns\n\n");
    board.push_str(&campaigns(model));
    board.push_str("\n## Blockers\n\n");
    board.push_str(&blockers(model));
    board.push_str("\n[Plan a release](@/contributing/maintenance.md#planning-a-release) by adding a record to `war-room/plans.toml`.\n");
    site.page(
        "releases/board",
        "Release board",
        "What is planned, what is blocked, and what is still unknown.",
        &board,
    )?;

    for plan in &model.plans {
        let projects: Vec<String> = plan
            .projects
            .iter()
            .map(|id| project_link(model.project(id)))
            .collect();
        let platforms: Vec<&str> = plan
            .platforms
            .iter()
            .map(|id| model.platform(id).name.as_str())
            .collect();
        let mut body = format!(
            "**{} · {}** · reviewed {}\n\n{}\n\n| Field | Value |\n|---|---|\n| Projects | {} |\n| Version | {} |\n| Target date | {} |\n| Platforms | {} |\n| Distribution | {} |\n\n## Requirements\n\n| Requirement | Type | State | Detail |\n|---|---|---|---|\n",
            label(&plan.kind),
            label(&plan.state),
            plan.reviewed,
            plan.notes,
            projects.join(", "),
            plan.version.as_deref().unwrap_or("not assigned"),
            plan.target_date.as_deref().unwrap_or("not committed"),
            platforms.join(", "),
            labels(&plan.distribution),
        );
        for requirement in &plan.requirements {
            let evidence = if requirement.evidence.is_empty() {
                String::new()
            } else {
                format!(" Evidence: {}.", requirement.evidence.join("; "))
            };
            writeln!(
                body,
                "| {} | {} | **{}** | {}{} |",
                cell(&requirement.label),
                label(&requirement.kind),
                label(&requirement.state),
                cell(&requirement.detail),
                cell(&evidence)
            )?;
        }
        if !plan.depends_on.is_empty() {
            body.push_str("\n## Prerequisites\n\n");
            for dependency in &plan.depends_on {
                writeln!(body, "- [{dependency}](@/releases/plans/{dependency}.md)")?;
            }
        }
        if !plan.links.is_empty() {
            body.push_str("\n## Links\n\n");
            for link in &plan.links {
                writeln!(body, "- <{link}>")?;
            }
        }
        site.page(
            &format!("releases/plans/{}", plan.id),
            &plan.name,
            &format!("{} · reviewed {}", label(&plan.state), plan.reviewed),
            &body,
        )?;
    }
    Ok(())
}

fn release_pulse(model: &Model) -> String {
    let mut table = String::from(
        "| Project | Next release | Plan state | CI / docs | Distribution |\n|---|---|---|---|---|\n",
    );
    for project in model
        .current_projects()
        .filter(|project| matches!(project.kind, Kind::Application | Kind::DeveloperTool))
    {
        let plan = model.plans.iter().find(|plan| {
            plan.kind == PlanKind::Release
                && plan.projects.contains(&project.id)
                && !plan.state.finished()
        });
        let (version, state, health) = plan.map_or_else(
            || {
                (
                    "not planned".to_owned(),
                    "unknown".to_owned(),
                    "unknown".to_owned(),
                )
            },
            |plan| {
                let done = |kind: RequirementKind| {
                    let finished = plan
                        .requirements
                        .iter()
                        .filter(|requirement| requirement.kind == kind)
                        .all(|requirement| requirement.state == TaskState::Done);
                    if finished { "done" } else { "pending" }
                };
                (
                    format!(
                        "[{}](@/releases/plans/{}.md)",
                        plan.version.as_deref().unwrap_or("unnamed"),
                        plan.id
                    ),
                    label(&plan.state),
                    format!(
                        "{} / {}",
                        done(RequirementKind::Ci),
                        done(RequirementKind::Documentation)
                    ),
                )
            },
        );
        writeln!(
            table,
            "| {} | {version} | {state} | {health} | {} |",
            project_link(project),
            labels(&project.distribution)
        )
        .expect("writing to a String cannot fail");
    }
    table
}

fn campaigns(model: &Model) -> String {
    let mut table = String::from("| Campaign | State | Next attention |\n|---|---|---|\n");
    for plan in model
        .plans
        .iter()
        .filter(|plan| plan.kind == PlanKind::Campaign)
    {
        let next = plan
            .requirements
            .iter()
            .find(|requirement| requirement.state == TaskState::Blocked)
            .or_else(|| {
                plan.requirements
                    .iter()
                    .find(|requirement| requirement.state != TaskState::Done)
            })
            .map_or("All requirements done".to_owned(), |requirement| {
                cell(&requirement.label)
            });
        writeln!(
            table,
            "| [{}](@/releases/plans/{}.md) | **{}** | {next} |",
            cell(&plan.name),
            plan.id,
            label(&plan.state)
        )
        .expect("writing to a String cannot fail");
    }
    table
}

fn blockers(model: &Model) -> String {
    let mut text = String::new();
    for plan in &model.plans {
        for requirement in plan
            .requirements
            .iter()
            .filter(|requirement| requirement.state == TaskState::Blocked)
        {
            writeln!(
                text,
                "- **[{}](@/releases/plans/{}.md): {}** — {}",
                cell(&plan.name),
                plan.id,
                cell(&requirement.label),
                cell(&requirement.detail)
            )
            .expect("writing to a String cannot fail");
        }
    }
    if text.is_empty() {
        text.push_str("Nothing recorded as blocked.\n");
    }
    text
}

fn platforms(site: &Site, model: &Model) -> Result<()> {
    let toolchains = &model.toolchains;
    let mut body = format!(
        "Policy reviewed **{}** in `war-room/toolchains.toml`. Runner, container and condition come from the workflow YAML. *Validation* records whether a platform has been proven, not merely configured.\n\n## Compiler policy\n\n{}\n\n{}\n\n**Reference measurement:** {}\n\n## Target matrix\n\n",
        toolchains.reviewed, toolchains.rust_policy, toolchains.llvm_policy, toolchains.reference
    );
    body.push_str(&target_table(model));
    for platform in &toolchains.platforms {
        let workflow = model.workflow(&platform.workflow);
        let job = &workflow.jobs[&platform.job];
        writeln!(
            body,
            "\n## {}\n\n{}\n\n- Job: [{} / {}](@/stroggforge/workflows/{}.md)\n- Runner: `{}`\n- Container: `{}`\n- Condition: `{}`\n- Strategy: `{}`",
            platform.name,
            platform.notes,
            workflow.id,
            platform.job,
            workflow.id,
            cell(&text(&job.runner)),
            cell(&text(&job.container)),
            cell(&text(&job.condition)),
            cell(&text(&job.strategy))
        )?;
    }
    body.push_str("\nRetired baselines are in the [history](@/archaeology/history.md#retired-infrastructure).\n");
    site.page(
        "releases/platforms",
        "Platforms and toolchains",
        "Configured environments, intentional differences, and what has been proven.",
        &body,
    )
}

fn workflow_pages(site: &Site, root: &Path, model: &Model) -> Result<()> {
    site.section(
        "stroggforge/workflows",
        "Canonical workflows and actions",
        10,
        "Contracts, jobs and conditions are read from the workflow YAML at build time. Operating notes come from `war-room/workflows.toml`.\n",
    )?;
    for workflow in &model.workflows {
        workflow_page(site, workflow, model.operations.workflow.get(&workflow.id))?;
    }

    let mut integration = String::from(
        "## Callers\n\nProjects whose repositories call a StroggForge workflow, as recorded in `war-room/ecosystem.toml`. Rust manufacturing goes through `rustGlobalBuild` and `libGlobalBuild`; Mod Template sites go through `modGlobalBuild`, which Rust repositories reach through `mod_template`.\n\n",
    );
    integration.push_str(&callers_list(model));
    for (heading, introduction, template) in [
        (
            "Binary integration template",
            "Review permissions, secrets, publish flags, binary/package selection and the pinned ref.",
            "rust_template.yaml",
        ),
        (
            "Library integration template",
            "`crate_names` must match Cargo `[package].name`. Enable `cargo_publish_workspace` for dependency-ordered workspace publishing.",
            "lib_template.yaml",
        ),
        (
            "Mod Template integration template",
            "For a site of mods, as `build_site.yml`. Rust repositories set `mod_template: true` on their application or library caller instead.",
            "mod_template.yaml",
        ),
        (
            "Daily quality template",
            "Scheduled fmt/Clippy/RustSec checks with no release secrets. It calls `setup-llvm` directly because Clippy compiles.",
            "daily_quality_template.yaml",
        ),
    ] {
        let source = fs::read_to_string(root.join(".github/action_templates").join(template))?;
        writeln!(
            integration,
            "\n## {heading}\n\n{introduction} Source: `.github/action_templates/{template}`.\n\n```yaml\n{}\n```",
            source.trim_end()
        )?;
    }
    integration.push_str("\n[Input reference](@/stroggforge/inputs.md) · [Incident desk](@/stroggforge/troubleshooting.md) · [Publishing](@/releases/publishing.md)\n");
    site.page(
        "stroggforge/integration",
        "Integration and callers",
        "Who calls StroggForge and how to integrate.",
        &integration,
    )?;

    inputs(site, model)?;
    supply_chain(site, model)
}

fn workflow_page(site: &Site, workflow: &Workflow, note: Option<&Note>) -> Result<()> {
    let kind = if workflow.action {
        "composite action"
    } else if workflow.callable {
        "reusable workflow"
    } else {
        "repository automation"
    };
    let mut body = format!(
        "Source: [{0}](https://github.com/DreamWeave-MP/StroggForge/blob/main/{0}) · **{kind}**\n\n",
        workflow.path
    );
    if let Some(note) = note {
        writeln!(
            body,
            "{}\n\n**Intended for:** {}\n\n## Release behavior\n\n{}\n\n## Artifacts and effects\n",
            note.purpose, note.intended_for, note.release_behavior
        )?;
        for (heading, items) in [
            ("", &note.artifacts),
            ("\n## Assumptions and permissions\n\n", &note.assumptions),
            ("\n## Failure modes\n\n", &note.failures),
        ] {
            body.push_str(heading);
            for item in items {
                writeln!(body, "- {item}")?;
            }
        }
        writeln!(body, "\n## Example\n\n{}\n", note.example)?;
    }
    for (heading, fields) in [
        ("Inputs", &workflow.inputs),
        ("Secrets", &workflow.secrets),
        ("Outputs", &workflow.outputs),
    ] {
        if fields.is_empty() {
            continue;
        }
        writeln!(
            body,
            "## {heading}\n\n| Name | Type | Required | Default | Description |\n|---|---|---|---|---|"
        )?;
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
        "## Permissions\n\nTop-level: `{}`. Undeclared means inherited from the caller, not \"none needed\".\n",
        cell(&text(&workflow.permissions))
    )?;
    if !workflow.jobs.is_empty() {
        let name = format!("workflow-{}", workflow.id);
        site.map(&name, &graphs::workflow(workflow)?)?;
        body.push_str("## Job dependency graph\n\nEdges are YAML `needs`; conditional jobs may not run. Conditions are listed below.\n");
        body.push_str(&diagram(&name));
        body.push_str("## Jobs\n\n| Job | Needs | Runner / delegate | Container / strategy | Condition |\n|---|---|---|---|---|\n");
        for (name, job) in &workflow.jobs {
            writeln!(
                body,
                "| `{}` | {} | {} / {} | {} / {} | {} |",
                cell(name),
                cell(&job.dependencies()?.join(", ")),
                cell(&text(&job.runner)),
                cell(&text(&job.uses)),
                cell(&text(&job.container)),
                cell(&text(&job.strategy)),
                cell(&text(&job.condition))
            )?;
        }
    }
    body.push_str("\n## Steps\n");
    for (name, job) in &workflow.jobs {
        steps(&mut body, name, &job.steps)?;
    }
    if !workflow.steps.is_empty() {
        steps(&mut body, "Composite action", &workflow.steps)?;
    }
    site.page(
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

fn steps(body: &mut String, heading: &str, steps: &[Value]) -> Result<()> {
    writeln!(
        body,
        "\n### {heading}\n\n| Step | Action / command | Condition |\n|---|---|---|"
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

/// Every input and secret across the contracts, side by side.
fn inputs(site: &Site, model: &Model) -> Result<()> {
    let mut body = String::from(
        "Every input and secret declared by a reusable workflow or composite action. Cells show the default, **required**, or `·` where the contract lacks the name. Types and descriptions are on each [contract page](@/stroggforge/workflows/_index.md).\n",
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
        let mut names: Vec<&String> = contracts
            .iter()
            .flat_map(|workflow| select(workflow).keys())
            .collect();
        names.sort();
        names.dedup();
        writeln!(body, "\n## {heading}\n")?;
        body.push_str("| Name |");
        for workflow in &contracts {
            write!(body, " {} |", workflow_link(&workflow.id))?;
        }
        writeln!(body, "\n|---|{}", "---|".repeat(contracts.len()))?;
        for name in names {
            write!(body, "| `{}` |", cell(name))?;
            for workflow in &contracts {
                let shown = match select(workflow).get(name) {
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
    body.push_str("\nAn optional secret can still be required in practice: non-PR binary releases need `VT_API_KEY`, and publishing needs `CARGO_REGISTRY_TOKEN`.\n");
    site.page(
        "stroggforge/inputs",
        "Input and secret reference",
        "Every workflow input and secret, compared across contracts.",
        &body,
    )
}

fn supply_chain(site: &Site, model: &Model) -> Result<()> {
    let mut body = String::from(
        "## Stages\n\nStages are ordered by the earliest point each can start, not a strict sequence; jobs overlap wherever the [binary](@/stroggforge/workflows/rustGlobalBuild.md#job-dependency-graph) and [library](@/stroggforge/workflows/libGlobalBuild.md#job-dependency-graph) `needs` graphs allow. Signing covers the **binary before archiving**. Pull requests build and upload a workflow artifact; they skip signing, scanning and uploads.\n",
    );
    body.push_str(
        "\n{{ schematic(data_path=\"static/generated/schematics/supply-line.json\") }}\n\n",
    );
    body.push_str("| # | Stage | Jobs and steps | What happens |\n|---|---|---|---|\n");
    for (index, stage) in model.operations.stage.iter().enumerate() {
        let mut groups: Vec<(&str, Vec<&str>)> = Vec::new();
        for reference in &stage.jobs {
            let (workflow, name) = reference.split_once('/').unwrap_or((reference, ""));
            match groups.iter_mut().find(|(id, _)| *id == workflow) {
                Some((_, names)) => names.push(name),
                None => groups.push((workflow, vec![name])),
            }
        }
        let jobs = if groups.is_empty() {
            "caller workflow".to_owned()
        } else {
            groups
                .iter()
                .map(|(workflow, names)| {
                    format!("{}: {}", workflow_link(workflow), cell(&names.join(", ")))
                })
                .collect::<Vec<_>>()
                .join("<br>")
        };
        writeln!(
            body,
            "| {} | **{}** | {jobs} | {} |",
            index + 1,
            cell(&stage.name),
            cell(&stage.summary)
        )?;
    }
    body.push_str("\n## Callers\n\n");
    body.push_str(&callers_list(model));
    body.push_str("\n## Boundaries\n\nStroggForge manufactures Rust components. The Mod Template provides the static docs shell. St4sh (with Cod3x) is the Lua(u)/OpenMW side; it and the Mod Template call only `createRelease` to refresh the release their static site uploads to. Libraries skip binary packaging and use the crate publication lane.\n");
    site.page(
        "engineering/supply-chain",
        "Supply chain",
        "Source to callers, with actual workflow responsibilities.",
        &body,
    )?;

    let body = format!(
        "## Binary release\n\nThe caller decides which events invoke the workflow. This is the exact `needs` graph; the [contract](@/stroggforge/workflows/rustGlobalBuild.md) lists every `if` guard.\n{}## Library release\n\nQuality gates precede `release_cleanup`; publishing and release material follow their job prerequisites. No binary signing or packaging.\n{}## Trigger boundaries\n\n- Pull request: quality and build checks; no signing, scanning or release upload.\n- Main push: quality checks and configured rustdoc Pages. Non-tag releases use the rolling `development` release.\n- Tag: configured crates.io publication and a named GitHub Release.\n- AUR: configured push path after platform success; not restricted to tags.\n- Nexus: configured non-PR path after platform success; needs the secret pair and group IDs.\n- Android/PortMaster: opt-in native artifacts, not APKs or a PortMaster catalog submission.\n\n[Publishing and recovery](@/releases/publishing.md) · [Supply-chain stages](@/engineering/supply-chain.md)\n",
        diagram("workflow-rustGlobalBuild"),
        diagram("workflow-libGlobalBuild")
    );
    site.page(
        "releases/pipelines",
        "Release pipelines",
        "From Git state to artifacts, with conditional lanes intact.",
        &body,
    )
}
