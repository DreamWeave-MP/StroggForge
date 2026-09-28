use std::collections::BTreeSet;

use anyhow::{Context, Result, bail, ensure};

use crate::{
    model::{Model, RelationshipKind, State, Successor, TaskState},
    workflows::resolve,
};

/// Structural invariants only: unique IDs, references that resolve, plan states that agree
/// with their requirements, and no cycles. Editorial judgment stays in review.
pub fn validate(model: &Model) -> Result<()> {
    let ecosystem = &model.ecosystem;
    ensure!(
        ecosystem.schema == 1,
        "war-room/ecosystem.toml: unsupported schema {}",
        ecosystem.schema
    );
    date(&ecosystem.reviewed)?;
    date(&model.toolchains.reviewed)?;

    unique(
        ecosystem
            .projects
            .iter()
            .map(|project| project.id.as_str())
            .chain(ecosystem.retired.iter().map(|retired| retired.id.as_str())),
        "project/retired",
    )?;
    unique(model.plans.iter().map(|plan| plan.id.as_str()), "plan")?;
    unique(
        model
            .toolchains
            .platforms
            .iter()
            .map(|platform| platform.id.as_str()),
        "platform",
    )?;
    unique(
        model.operations.stage.iter().map(|stage| stage.id.as_str()),
        "stage",
    )?;

    projects(model)?;
    relationships(model)?;
    plans(model)?;
    references(model)?;
    Ok(())
}

fn projects(model: &Model) -> Result<()> {
    for project in &model.ecosystem.projects {
        for workflow in &project.stroggforge {
            ensure!(
                model
                    .workflows
                    .iter()
                    .any(|candidate| &candidate.id == workflow && candidate.callable),
                "project {}: {workflow} is not a reusable StroggForge workflow",
                project.id
            );
        }
    }
    Ok(())
}

fn relationships(model: &Model) -> Result<()> {
    let ecosystem = &model.ecosystem;
    let mut seen = BTreeSet::new();
    for relationship in &ecosystem.relationships {
        let (from, to) = (&relationship.from, &relationship.to);
        project_exists(model, from, "relationship")?;
        project_exists(model, to, "relationship")?;
        ensure!(from != to, "relationship {from} -> {to} points at itself");
        ensure!(
            seen.insert((from, to, relationship.kind)),
            "duplicate relationship {from} -> {to}"
        );
        if relationship.kind == RelationshipKind::Successor {
            ensure!(
                model.project(from).lifecycle.historical(),
                "{from} has a successor but is not superseded or archived"
            );
        }
    }
    acyclic(
        ecosystem
            .relationships
            .iter()
            .filter(|relationship| relationship.kind == RelationshipKind::Successor)
            .map(|relationship| (relationship.from.as_str(), relationship.to.as_str())),
        "successor",
    )
}

fn plans(model: &Model) -> Result<()> {
    for plan in &model.plans {
        let context = format!("plan {}", plan.id);
        date(&plan.reviewed).context(context.clone())?;
        if let Some(target) = &plan.target_date {
            date(target).context(context.clone())?;
        }
        for project in &plan.projects {
            project_exists(model, project, &context)?;
        }
        for platform in &plan.platforms {
            platform_exists(model, platform, &context)?;
        }
        unique(
            plan.requirements
                .iter()
                .map(|requirement| requirement.id.as_str()),
            "requirement",
        )
        .context(context.clone())?;

        let blocked_requirement = plan
            .requirements
            .iter()
            .any(|requirement| requirement.state == TaskState::Blocked);
        let mut blocked_prerequisite = false;
        for dependency in &plan.depends_on {
            let prerequisite = model
                .plans
                .iter()
                .find(|candidate| &candidate.id == dependency)
                .with_context(|| format!("{context}: unknown prerequisite {dependency}"))?;
            blocked_prerequisite |= prerequisite.state == State::Blocked;
            let claims_done = plan.state == State::Ready || plan.state.finished();
            ensure!(
                !claims_done || prerequisite.state.finished(),
                "{context} is {:?} but prerequisite {dependency} is not finished",
                plan.state
            );
        }
        ensure!(
            (blocked_requirement || blocked_prerequisite) == (plan.state == State::Blocked),
            "{context}: a plan is blocked exactly when a requirement or prerequisite is blocked"
        );
        if plan.state == State::Ready || plan.state.finished() {
            ensure!(
                plan.requirements
                    .iter()
                    .all(|requirement| requirement.state == TaskState::Done),
                "{context} is {:?} with unfinished requirements",
                plan.state
            );
        }
    }
    acyclic(
        model.plans.iter().flat_map(|plan| {
            plan.depends_on
                .iter()
                .map(move |dependency| (plan.id.as_str(), dependency.as_str()))
        }),
        "plan prerequisite",
    )
}

/// References from updates, retired records, platforms and stages to other records.
fn references(model: &Model) -> Result<()> {
    for update in &model.ecosystem.updates {
        date(&update.date).with_context(|| format!("update {:?}", update.title))?;
        if let Some(plan) = &update.plan {
            ensure!(
                model.plans.iter().any(|candidate| &candidate.id == plan),
                "update {:?}: unknown plan {plan}",
                update.title
            );
        }
    }
    for retired in &model.ecosystem.retired {
        let context = format!("retired {}", retired.id);
        match &retired.successor {
            None => {}
            Some(Successor::Project(id)) => project_exists(model, id, &context)?,
            Some(Successor::Platform(id)) => platform_exists(model, id, &context)?,
        }
    }
    for platform in &model.toolchains.platforms {
        ensure!(
            model.workflows.iter().any(|workflow| {
                workflow.id == platform.workflow && workflow.jobs.contains_key(&platform.job)
            }),
            "platform {}: no job {}/{}",
            platform.id,
            platform.workflow,
            platform.job
        );
    }
    for id in model.operations.workflow.keys() {
        ensure!(
            model.workflows.iter().any(|workflow| &workflow.id == id),
            "war-room/workflows.toml: notes for unknown workflow {id}"
        );
    }
    for stage in &model.operations.stage {
        for reference in &stage.jobs {
            resolve(&model.workflows, reference).with_context(|| format!("stage {}", stage.id))?;
        }
    }
    Ok(())
}

fn project_exists(model: &Model, id: &str, context: &str) -> Result<()> {
    ensure!(
        model
            .ecosystem
            .projects
            .iter()
            .any(|project| project.id == id),
        "{context}: unknown project {id}"
    );
    Ok(())
}

fn platform_exists(model: &Model, id: &str, context: &str) -> Result<()> {
    ensure!(
        model
            .toolchains
            .platforms
            .iter()
            .any(|platform| platform.id == id),
        "{context}: unknown platform {id}"
    );
    Ok(())
}

fn unique<'a>(values: impl Iterator<Item = &'a str>, what: &str) -> Result<()> {
    let mut seen = BTreeSet::new();
    for value in values {
        ensure!(
            !value.is_empty()
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'),
            "invalid {what} ID {value:?}; use lowercase letters, digits and hyphens"
        );
        ensure!(seen.insert(value), "duplicate {what} ID {value}");
    }
    Ok(())
}

fn date(value: &str) -> Result<()> {
    let parts: Vec<u32> = value
        .split('-')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .with_context(|| format!("invalid date {value:?}; expected YYYY-MM-DD"))?;
    let [year, month, day] = parts[..] else {
        bail!("invalid date {value:?}; expected YYYY-MM-DD");
    };
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    };
    ensure!(
        value.len() == 10 && (1..=days).contains(&day),
        "invalid date {value:?}; expected YYYY-MM-DD"
    );
    Ok(())
}

fn acyclic<'a>(edges: impl Iterator<Item = (&'a str, &'a str)>, what: &str) -> Result<()> {
    let mut remaining: Vec<_> = edges.collect();
    while !remaining.is_empty() {
        let origins: BTreeSet<_> = remaining.iter().map(|(from, _)| *from).collect();
        let before = remaining.len();
        remaining.retain(|(_, to)| origins.contains(to));
        ensure!(remaining.len() < before, "{what} cycle among {remaining:?}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::model::Relationship;

    fn committed() -> Model {
        Model::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap()
    }

    fn rejects(mutate: impl FnOnce(&mut Model), expected: &str) {
        let mut model = committed();
        mutate(&mut model);
        let error = format!(
            "{:#}",
            validate(&model).expect_err("broken metadata accepted")
        );
        assert!(
            error.contains(expected),
            "expected {expected:?} in {error:?}"
        );
    }

    #[test]
    fn committed_metadata_is_valid() {
        validate(&committed()).unwrap();
    }

    #[test]
    fn dates_are_calendar_dates() {
        assert!(date("2024-02-29").is_ok());
        assert!(date("2026-02-29").is_err());
        assert!(date("2026-9-28").is_err());
    }

    #[test]
    fn duplicate_ids_fail() {
        rejects(
            |model| model.ecosystem.projects[1].id = model.ecosystem.projects[0].id.clone(),
            "duplicate project/retired ID",
        );
    }

    #[test]
    fn unresolved_relationships_fail() {
        rejects(
            |model| {
                model.ecosystem.relationships.push(Relationship {
                    from: "vfstool".into(),
                    to: "no-such-project".into(),
                    kind: RelationshipKind::Rust,
                    label: "x".into(),
                });
            },
            "unknown project no-such-project",
        );
    }

    #[test]
    fn unknown_relationship_kinds_fail_to_parse() {
        let text = "from = \"a\"\nto = \"b\"\nkind = \"vibes\"\nlabel = \"x\"\n";
        assert!(toml::from_str::<Relationship>(text).is_err());
    }

    #[test]
    fn stroggforge_calls_must_name_reusable_workflows() {
        rejects(
            |model| model.ecosystem.projects[0].stroggforge = vec!["deployToMars".into()],
            "is not a reusable StroggForge workflow",
        );
    }

    #[test]
    fn plan_state_must_agree_with_blockers() {
        rejects(
            |model| model.plans[0].state = State::Active,
            "blocked exactly when",
        );
    }

    #[test]
    fn plan_prerequisite_cycles_fail() {
        rejects(
            |model| {
                let (first, second) = (model.plans[0].id.clone(), model.plans[1].id.clone());
                model.plans[0].depends_on.push(second);
                model.plans[1].depends_on.push(first);
                model.plans[1].state = State::Blocked;
            },
            "plan prerequisite cycle",
        );
    }
}
