use std::{fmt::Write as _, path::Path};

use anyhow::Result;

use serde::Serialize;

use super::{cell, diagram, history, page, pipelines, project_link, section, write};
use crate::model::{Domain, Kind, Model, PlanKind, Relationship, State, TaskState, label};

pub fn build(site: &Path, model: &Model) -> Result<()> {
    projects(site, model)?;
    plans(site, model)?;
    homepage(site, model)?;
    platforms(site, model)?;
    Ok(())
}

fn projects(site: &Path, model: &Model) -> Result<()> {
    section(
        site,
        "ecosystem/projects",
        "Project directory",
        20,
        "Current components, including experimental and maintenance work. [Retired names live in Archaeology](@/archaeology/projects/_index.md).\n",
    )?;
    section(
        site,
        "archaeology/projects",
        "Historical projects",
        10,
        "Historical nodes are excluded from current graphs. A successor edge requires evidence.\n",
    )?;
    let mut directory = format!(
        "{}\n\nDreamWeave has three sides. **StroggForge serves the Rust ecosystem only.** The Lua(u)/OpenMW content side and the web sites are part of DreamWeave and documented here, but they are not Rust components and do not depend on the Rust supply line.\n",
        model.inventory.scope
    );
    for domain in Domain::ALL {
        writeln!(
            directory,
            "\n## {}\n\n| Project | Role | Lifecycle | Approach | Summary |\n|---|---|---|---|---|",
            domain.title()
        )?;
        for project in model
            .inventory
            .projects
            .iter()
            .filter(|project| project.domain == domain && !project.lifecycle.historical())
        {
            writeln!(
                directory,
                "| {} | {} | {} | {} | {} |",
                project_link(project),
                label(&project.kind),
                label(&project.lifecycle),
                label(&project.approach),
                cell(&project.summary)
            )?;
        }
    }
    let mut foundations = String::from(
        "Reusable foundations are identified by package name, not directory spelling. Arrows point **consumer → dependency**. Optional, development and target-specific dependencies keep their qualifiers. These are declared direct requirements, not a resolved Cargo.lock or proof that a consumer has upgraded to the newest version.\n",
    );
    foundations.push_str(&diagram("rust-dependencies"));
    foundations.push_str("| Project | Packages | Role |\n|---|---|---|\n");
    let mut contributions = String::from(
        "Pick a domain and an approachable change. These are contribution areas, not a claim that a particular issue is unassigned. Open the repository's issue tracker and contribution instructions before starting.\n\n| Project | Domain | Approach | Useful work |\n|---|---|---|---|\n",
    );
    for project in &model.inventory.projects {
        project_page(site, model, project)?;
        if !project.lifecycle.historical() {
            writeln!(
                contributions,
                "| {} | {} | {} | {} |",
                project_link(project),
                project.domain.title(),
                label(&project.approach),
                cell(&project.contribution_areas.join("; "))
            )?;
            if project.kind == Kind::Foundation {
                writeln!(
                    foundations,
                    "| {} | {} | {} |",
                    project_link(project),
                    project.packages.join(", "),
                    cell(&project.summary)
                )?;
            }
        }
    }
    page(
        site,
        "ecosystem/overview",
        "Ecosystem overview",
        "A reviewed inventory, with source receipts and honest unknowns.",
        &directory,
    )?;
    page(
        site,
        "ecosystem/foundations",
        "Rust foundations",
        "The reusable libraries underneath the applications.",
        &foundations,
    )?;
    page(
        site,
        "contributing/targets",
        "Good contribution targets",
        "Find a repository and a meaningful first task.",
        &contributions,
    )
}

fn project_page(site: &Path, model: &Model, project: &crate::model::Project) -> Result<()> {
    let snapshot = &model.sources[&project.source];
    let repository = model.repository(project);
    let mut body = format!(
        "**{} · {} · {} · {}**\n\n{}\n\n## Field record\n\n| Field | Value |\n|---|---|\n| Repository | {} |\n| Component path | `{}` |\n| Language | {} |\n| Stewards | {} |\n| Approach | {} |\n| Distribution channels | {} |\n| Snapshot | {} · `{}` · {} |\n\n",
        project.domain.title(),
        label(&project.kind),
        label(&project.lifecycle),
        label(&project.maturity),
        project.summary,
        if repository == "local-only" {
            "Local-only checkout; public location unknown".to_owned()
        } else {
            format!("[Repository]({repository}) · [Issues]({repository}/issues)")
        },
        cell(&project.path),
        cell(&project.languages.join(", ")),
        cell(&project.owners.join(", ")),
        label(&project.approach),
        project
            .distribution
            .iter()
            .map(label)
            .collect::<Vec<_>>()
            .join(", "),
        snapshot.observed,
        snapshot.revision,
        if snapshot.dirty {
            "working tree had local changes"
        } else {
            "clean working tree"
        }
    );
    if project.lifecycle.historical() {
        let successors = model
            .edges
            .iter()
            .filter(|edge| edge.from == project.id && edge.kind == Relationship::Successor)
            .map(|edge| project_link(model.project(&edge.to)))
            .collect::<Vec<_>>()
            .join(", ");
        body.insert_str(
            0,
            &history::unsupported_notice(&format!("Use {successors} instead.")),
        );
    }
    body.push_str("Distribution channels describe intent/configuration, not successful uploads. Snapshot dates are explicit; this is not a live CI monitor.\n\n## Documentation\n\n");
    if project.docs.is_empty() {
        body.push_str("Public documentation location **unknown**. Consult the source checkout's README and captured evidence.\n");
    }
    for url in &project.docs {
        writeln!(body, "- <{url}>")?;
    }
    body.push_str(
        "\n## Package facts\n\n| Package | Observed version | Declared MSRV |\n|---|---|---|\n",
    );
    for name in &project.packages {
        let package = snapshot
            .packages
            .iter()
            .find(|package| &package.name == name)
            .unwrap();
        writeln!(
            body,
            "| {} | {} | {} |",
            package.name,
            package.version,
            package
                .rust_version
                .as_deref()
                .unwrap_or("unknown / not declared")
        )?;
    }
    if project.packages.is_empty() {
        body.push_str("\nNo Cargo packages assigned to this component.\n");
    }
    project_relationships(&mut body, model, project)?;
    body.push_str("\n## Contribute\n\n");
    for area in &project.contribution_areas {
        writeln!(body, "- {}", cell(area))?;
    }
    writeln!(
        body,
        "\n## Evidence and limitations\n\n{}\n\nSource snapshot: `war-room/sources/{}.json`. Refresh from an explicit local clone; review the diff.\n",
        project.evidence, project.source
    )?;
    let lane = if project.lifecycle.historical() {
        "archaeology/projects"
    } else {
        "ecosystem/projects"
    };
    page(
        site,
        &format!("{lane}/{}", project.id),
        &project.name,
        &project.summary,
        &body,
    )
}

fn project_relationships(
    body: &mut String,
    model: &Model,
    project: &crate::model::Project,
) -> Result<()> {
    body.push_str("\n## Supply-line integration\n\n");
    let calls: Vec<_> = model
        .edges
        .iter()
        .filter(|edge| edge.from == project.id && edge.kind == Relationship::Build)
        .collect();
    if calls.is_empty() {
        body.push_str("**No verified StroggForge caller in the captured source.** That does not establish that no external automation exists.\n");
    }
    for call in calls {
        writeln!(body, "- {} — `{}`", cell(&call.label), cell(&call.evidence))?;
    }
    body.push_str("\n## Relationships\n\n| Direction | Project | Relationship | Evidence |\n|---|---|---|---|\n");
    for edge in &model.edges {
        if edge.from == project.id || edge.to == project.id {
            let outgoing = edge.from == project.id;
            let other = model.project(if outgoing { &edge.to } else { &edge.from });
            writeln!(
                body,
                "| {} | {} | {}: {} | {} |",
                if outgoing { "outgoing" } else { "incoming" },
                project_link(other),
                label(&edge.kind),
                cell(&edge.label),
                cell(&edge.evidence)
            )?;
        }
    }
    Ok(())
}

fn plans(site: &Path, model: &Model) -> Result<()> {
    section(
        site,
        "releases/plans",
        "Release and campaign records",
        20,
        "Reviewed intent from `war-room/plans.toml`. Missing version/date means no commitment has been recorded.\n",
    )?;
    let mut board = String::from(
        "## Release pulse\n\nNo release version is inferred from Cargo.toml. A captured package version may be development, already published, or unreleased. CI and docs health remain unknown until a plan records evidence.\n\n",
    );
    board.push_str(&release_pulse(model));
    board.push_str("\n## Cross-project campaigns\n\n");
    board.push_str(&campaign_table(model));
    board.push_str("\n## Blockers and next actions\n\n");
    board.push_str(&blockers(model));
    board.push_str("\n[Plan a release](@/contributing/maintenance.md#planning-a-release) by adding a small reviewed TOML record. No dated release commitment was supplied for this initial inventory.\n");
    page(
        site,
        "releases/board",
        "Release board",
        "What is planned, what is blocked, and what is still unknown.",
        &board,
    )?;
    for plan in &model.plans.plans {
        let mut body = format!(
            "**{} · {}** · reviewed {}\n\n{}\n\n| Field | Value |\n|---|---|\n| Projects | {} |\n| Version | {} |\n| Target date | {} |\n| Platforms | {} |\n| Distribution | {} |\n\n## Requirements\n\n| Requirement | Type | State | Detail / evidence |\n|---|---|---|---|\n",
            label(&plan.kind),
            label(&plan.state),
            plan.reviewed,
            plan.notes,
            plan.projects
                .iter()
                .map(|id| project_link(model.project(id)))
                .collect::<Vec<_>>()
                .join(", "),
            plan.version.as_deref().unwrap_or("not assigned"),
            plan.target_date.as_deref().unwrap_or("not committed"),
            plan.platforms.join(", "),
            plan.distribution
                .iter()
                .map(label)
                .collect::<Vec<_>>()
                .join(", ")
        );
        for requirement in &plan.requirements {
            writeln!(
                body,
                "| {} | {} | **{}** | {} {} |",
                cell(&requirement.label),
                label(&requirement.kind),
                label(&requirement.state),
                cell(&requirement.detail),
                cell(&requirement.evidence.join("; "))
            )?;
        }
        body.push_str("\n## Prerequisites\n\n");
        if plan.depends_on.is_empty() {
            body.push_str("No other release/campaign prerequisite declared.\n");
        }
        for dependency in &plan.depends_on {
            writeln!(body, "- [{dependency}](@/releases/plans/{dependency}.md)")?;
        }
        body.push_str("\n## Links\n\n");
        for link in &plan.links {
            writeln!(body, "- <{link}>")?;
        }
        page(
            site,
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
    for project in model.inventory.projects.iter().filter(|project| {
        !project.lifecycle.historical()
            && matches!(project.kind, Kind::Application | Kind::DeveloperTool)
    }) {
        let plan = model.plans.plans.iter().find(|plan| {
            plan.kind == PlanKind::Release
                && plan.projects.contains(&project.id)
                && plan.state != State::Released
        });
        let (version, state, health) = plan.map_or_else(
            || {
                (
                    "not planned".into(),
                    "unknown".into(),
                    "unknown / unknown".into(),
                )
            },
            |plan| {
                let health = [
                    crate::model::RequirementKind::Ci,
                    crate::model::RequirementKind::Documentation,
                ]
                .map(|kind| {
                    let requirements: Vec<_> = plan
                        .requirements
                        .iter()
                        .filter(|requirement| requirement.kind == kind)
                        .collect();
                    if requirements
                        .iter()
                        .all(|requirement| requirement.state == TaskState::Done)
                    {
                        "recorded done"
                    } else {
                        "pending"
                    }
                })
                .join(" / ");
                (
                    format!(
                        "[{}](@/releases/plans/{}.md)",
                        plan.version.as_deref().unwrap_or("unnamed"),
                        plan.id
                    ),
                    label(&plan.state),
                    health,
                )
            },
        );
        writeln!(
            table,
            "| {} | {} | {} | {} | {} |",
            project_link(project),
            version,
            state,
            health,
            project
                .distribution
                .iter()
                .map(label)
                .collect::<Vec<_>>()
                .join(", ")
        )
        .unwrap();
    }
    table
}

fn campaign_table(model: &Model) -> String {
    let mut table = String::from("| Campaign | State | Next attention |\n|---|---|---|\n");
    for plan in model
        .plans
        .plans
        .iter()
        .filter(|plan| plan.kind == PlanKind::Campaign)
    {
        let attention = plan
            .requirements
            .iter()
            .find(|requirement| requirement.state == TaskState::Blocked)
            .or_else(|| {
                plan.requirements
                    .iter()
                    .find(|requirement| requirement.state != TaskState::Done)
            });
        writeln!(
            table,
            "| [{}](@/releases/plans/{}.md) | **{}** | {} |",
            cell(&plan.name),
            plan.id,
            label(&plan.state),
            attention.map_or("All recorded tasks complete".into(), |requirement| cell(
                &requirement.label
            ))
        )
        .unwrap();
    }
    table
}

fn blockers(model: &Model) -> String {
    let mut text = String::new();
    for plan in &model.plans.plans {
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
            .unwrap();
        }
    }
    if text.is_empty() {
        text.push_str("No blockers recorded. This is not a claim of green CI.\n");
    }
    text
}

/// Counts shown in the homepage pulse strip. Every number is derived from canonical data.
#[derive(Serialize)]
struct Summary<'a> {
    reviewed: &'a str,
    components: usize,
    open_campaigns: usize,
    planned_releases: usize,
    blocked_plans: usize,
    consumers: usize,
    platforms: usize,
    validated_platforms: usize,
}

fn summary(site: &Path, model: &Model) -> Result<()> {
    let open = |kind: PlanKind| {
        model
            .plans
            .plans
            .iter()
            .filter(|plan| {
                plan.kind == kind && !matches!(plan.state, State::Done | State::Released)
            })
            .count()
    };
    let summary = Summary {
        reviewed: &model.inventory.reviewed,
        components: model
            .inventory
            .projects
            .iter()
            .filter(|project| !project.lifecycle.historical())
            .count(),
        open_campaigns: open(PlanKind::Campaign),
        planned_releases: open(PlanKind::Release),
        blocked_plans: model
            .plans
            .plans
            .iter()
            .filter(|plan| plan.state == State::Blocked)
            .count(),
        consumers: pipelines::consumers(model).len(),
        platforms: model.toolchains.platforms.len(),
        validated_platforms: model
            .toolchains
            .platforms
            .iter()
            .filter(|platform| platform.validation == State::Ready)
            .count(),
    };
    write(
        site,
        "static/generated/summary.json",
        &format!("{}\n", serde_json::to_string_pretty(&summary)?),
    )
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

fn homepage(site: &Path, model: &Model) -> Result<()> {
    summary(site, model)?;
    let mut body = String::from(
        "+++\ntitle = \"War Room\"\ndescription = \"DreamWeave engineering · source, dependencies, releases, and the supply line.\"\ntemplate = \"war-room/home.html\"\npage_template = \"docs/page.html\"\nsort_by = \"weight\"\n[extra]\ndocs_root = true\ndocs_project_name = \"StroggForge\"\ndocs_short_title = \"DreamWeave War Room\"\ndocs_repository_url = \"https://github.com/DreamWeave-MP/StroggForge\"\ndocs_sidebar_label = \"War Room\"\nkind = \"DreamWeave engineering\"\n+++\n\n{{ pulse() }}\n\nEverything below is generated from version-controlled records and dated source snapshots. Nothing polls GitHub: **unknown** means no evidence has been recorded, not that something is broken.\n\n## Attention required\n\n",
    );
    body.push_str(&blockers(model));
    body.push_str("\n## Campaigns\n\n");
    body.push_str(&campaign_table(model));
    body.push_str("\n## Release pulse\n\nApplications and developer tools. A Cargo version is not a release plan; only `war-room/plans.toml` creates one.\n\n");
    body.push_str(&release_pulse(model));
    body.push_str("\n[Release board](@/releases/board.md) · [Plan a release](@/contributing/maintenance.md#planning-a-release)\n\n## Toolchain and targets\n\n");
    writeln!(
        body,
        "**Reference measurement:** {} Retired baselines are in the [history lane](@/archaeology/history.md#retired-infrastructure).\n",
        model.toolchains.reference
    )?;
    body.push_str(&target_table(model));
    body.push_str("\n[Platform matrix and intentional differences](@/releases/platforms.md)\n\n## System map\n\nConsumer → dependency. The compact view leaves StroggForge build edges to the [full map](@/ecosystem/map.md).\n");
    body.push_str(&diagram("compact"));
    body.push_str("[Full ecosystem map](@/ecosystem/map.md) · [Rust foundations](@/ecosystem/foundations.md) · [Historical lane](@/archaeology/history.md)\n\n## Supply line\n\n");
    let stages = model
        .operations
        .stage
        .iter()
        .map(|stage| cell(&stage.name))
        .collect::<Vec<_>>()
        .join(" → ");
    let mut callers = String::new();
    for line in pipelines::callers_by_workflow(model) {
        writeln!(callers, "- {line}")?;
    }
    let exceptional = model
        .toolchains
        .platforms
        .iter()
        .filter(|platform| platform.job != "release")
        .map(|platform| {
            format!(
                "{} (`{}`, {})",
                cell(&platform.name),
                platform.job,
                cell(&platform.baseline)
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    writeln!(
        body,
        "**{stages}** → consumers\n\n**Verified callers, by the workflow they call:**\n\n{callers}\n**Paths outside the hosted runner matrix:** {exceptional}.\n\nCapability is not run health: callers pin their own refs, and a configured platform is not a validated one.\n\n[Supply-chain stages](@/engineering/supply-chain.md) · [Workflow contracts](@/stroggforge/workflows/_index.md) · [Incident desk](@/stroggforge/troubleshooting.md)\n\n## Recent infrastructure changes\n"
    )?;
    body.push_str(&history::update_table(model, 4));
    body.push_str("\n[Full change log](@/releases/changes.md) · [Start contributing](@/contributing/_index.md)\n");
    write(site, "content/_index.md", &body)
}

fn platforms(site: &Path, model: &Model) -> Result<()> {
    let mut body = format!(
        "Policy reviewed **{}**. Runner/container/condition columns come directly from current workflow YAML. Baselines are reviewed assertions in `war-room/toolchains.toml`; drift fails generation. Validation records are separate.\n\n## Compiler policy\n\n{}\n\n{}\n\n**Reference measurement:** {}\n\n## Target matrix\n\n| Platform / target | Baseline | Compiler | Release validation |\n|---|---|---|---|\n",
        model.toolchains.reviewed,
        model.toolchains.rust_policy,
        model.toolchains.llvm_policy,
        model.toolchains.reference
    );
    for platform in &model.toolchains.platforms {
        writeln!(
            body,
            "| {} / `{}` | {} | {} | **{}** |",
            platform.name,
            platform.target,
            cell(&platform.baseline),
            cell(&platform.compiler),
            label(&platform.validation)
        )?;
    }
    for platform in &model.toolchains.platforms {
        let workflow = model
            .workflows
            .iter()
            .find(|workflow| workflow.id == platform.workflow)
            .unwrap();
        let job = &workflow.jobs[&platform.job];
        writeln!(
            body,
            "\n## {}\n\n{}\n\n- Job: [{} / {}](@/stroggforge/workflows/{}.md)\n- Runner: `{}`\n- Container: `{}`\n- Condition: `{}`\n- Strategy: `{}`\n",
            platform.name,
            platform.notes,
            workflow.id,
            platform.job,
            workflow.id,
            cell(&crate::workflows::text(&job.runner)),
            cell(&crate::workflows::text(&job.container)),
            cell(&crate::workflows::text(&job.condition)),
            cell(&crate::workflows::text(&job.strategy))
        )?;
    }
    body.push_str("\n## Package-declared MSRV\n\nA declaration is not a successful CI check. An empty `msrv` workflow input skips check steps while keeping the dependency job present.\n\n| Project | Package | Declared MSRV |\n|---|---|---|\n");
    for project in &model.inventory.projects {
        for name in &project.packages {
            let package = model.sources[&project.source]
                .packages
                .iter()
                .find(|package| &package.name == name)
                .unwrap();
            writeln!(
                body,
                "| {} | {} | {} |",
                project_link(project),
                package.name,
                package
                    .rust_version
                    .as_deref()
                    .unwrap_or("unknown / not declared")
            )?;
        }
    }
    page(
        site,
        "releases/platforms",
        "Platforms and toolchains",
        "Configured environments, intentional differences, and rollout evidence.",
        &body,
    )
}
