use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, ensure};

use crate::{
    model::{
        Domain, Lifecycle, Maturity, Model, PlanKind, Relationship, RequirementKind, RetiredKind,
        State, Successor, TaskState,
    },
    sources::local_file,
    workflows::resolve,
};

fn identifier(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'),
        "invalid ID {value:?}; use lowercase letters, digits and hyphens"
    );
    Ok(())
}

fn unique<'a>(values: impl Iterator<Item = &'a str>, what: &str) -> Result<()> {
    let mut seen = BTreeSet::new();
    for value in values {
        identifier(value)?;
        ensure!(seen.insert(value), "duplicate {what} ID: {value}");
    }
    Ok(())
}

fn required(value: &str, context: &str) -> Result<()> {
    ensure!(!value.trim().is_empty(), "{context}: empty required field");
    Ok(())
}

fn date(value: &str) -> Result<()> {
    let parts: Vec<&str> = value.split('-').collect();
    ensure!(
        parts.len() == 3 && parts[0].len() == 4 && parts[1].len() == 2 && parts[2].len() == 2,
        "invalid date {value}; expected YYYY-MM-DD"
    );
    let year: u32 = parts[0].parse()?;
    let month: u32 = parts[1].parse()?;
    let day: u32 = parts[2].parse()?;
    let maximum = match month {
        2 => {
            if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    };
    ensure!(
        year > 0 && day > 0 && day <= maximum,
        "invalid calendar date {value}"
    );
    Ok(())
}

pub fn validate(model: &Model) -> Result<()> {
    let inventory = &model.inventory;
    ensure!(
        inventory.schema == 1,
        "unsupported inventory schema {}",
        inventory.schema
    );
    date(&inventory.reviewed)?;
    required(&inventory.scope, "inventory scope")?;
    unique(
        inventory
            .projects
            .iter()
            .map(|project| project.id.as_str())
            .chain(inventory.retired.iter().map(|retired| retired.id.as_str())),
        "project or retired",
    )?;
    unique(
        inventory.sources.iter().map(|source| source.id.as_str()),
        "source",
    )?;
    unique(
        model.plans.plans.iter().map(|plan| plan.id.as_str()),
        "plan",
    )?;
    unique(
        model
            .toolchains
            .platforms
            .iter()
            .map(|platform| platform.id.as_str()),
        "platform",
    )?;
    validate_projects(model)?;
    validate_edges(model)?;
    validate_plans(model)?;
    validate_platforms(model)?;
    validate_updates(model)?;
    validate_retired(model)?;
    validate_operations(model)?;
    validate_census(model)?;
    Ok(())
}

/// Every archived organization repository must be documented as retired, and no current
/// project may live in a repository GitHub reports archived.
fn validate_census(model: &Model) -> Result<()> {
    let census = &model.organization;
    ensure!(
        census.schema == 1 && census.name == model.inventory.organization,
        "war-room/sources/organization.json: schema/organization mismatch; run capture-organization"
    );
    date(&census.observed)?;
    let mut names = BTreeSet::new();
    for repository in &census.repositories {
        ensure!(
            names.insert(repository.name.to_lowercase()),
            "organization census lists {} twice",
            repository.name
        );
        ensure!(
            !model.census_exclusions.contains(&repository.name),
            "organization census contains an excluded repository; rerun capture-organization"
        );
        date(&repository.pushed)?;
        let key = repository.url.to_lowercase();
        if repository.archived {
            ensure!(
                model.inventory.retired.iter().any(|retired| {
                    retired.kind == RetiredKind::Repository
                        && retired.location.to_lowercase() == key
                }),
                "{} is archived on GitHub but has no [[retired]] record in war-room/ecosystem.toml; document it as no longer maintained",
                repository.name
            );
        }
        for project in &model.inventory.projects {
            let source = model
                .inventory
                .sources
                .iter()
                .find(|source| source.id == project.source)
                .expect("source reference validated");
            ensure!(
                !(repository.archived
                    && source.repository.to_lowercase() == key
                    && !project.lifecycle.historical()),
                "{}: marked {} but GitHub reports {} archived",
                project.id,
                crate::model::label(&project.lifecycle),
                repository.name
            );
        }
    }
    Ok(())
}

fn validate_operations(model: &Model) -> Result<()> {
    let operations = &model.operations;
    for id in operations.workflow.keys() {
        ensure!(
            model.workflows.iter().any(|workflow| &workflow.id == id),
            "workflow notes reference nonexistent workflow {id}"
        );
    }
    for workflow in &model.workflows {
        ensure!(
            !(workflow.callable || workflow.action)
                || operations.workflow.contains_key(&workflow.id),
            "{}: callable workflow/action requires notes in .github/war-room-workflows.toml",
            workflow.id
        );
    }
    unique(
        operations.stage.iter().map(|stage| stage.id.as_str()),
        "supply-chain stage",
    )?;
    ensure!(
        !operations.stage.is_empty(),
        "supply-chain stages cannot be empty"
    );
    for stage in &operations.stage {
        required(&stage.name, &stage.id)?;
        required(&stage.summary, &stage.id)?;
        let mut references = BTreeSet::new();
        for reference in &stage.jobs {
            ensure!(
                references.insert(reference),
                "{}: duplicate job reference {reference}",
                stage.id
            );
            resolve(&model.workflows, reference)
                .with_context(|| format!("supply-chain stage {}", stage.id))?;
        }
    }
    Ok(())
}

fn validate_updates(model: &Model) -> Result<()> {
    let mut previous: Option<&str> = None;
    for update in &model.inventory.updates {
        date(&update.date)?;
        required(&update.title, "infrastructure update")?;
        required(&update.detail, &update.title)?;
        ensure!(
            previous.is_none_or(|previous| previous >= update.date.as_str()),
            "{}: infrastructure updates must be listed newest first",
            update.title
        );
        previous = Some(&update.date);
        ensure!(
            !update.commits.is_empty(),
            "{}: an infrastructure update needs the commits that carry it",
            update.title
        );
        for commit in &update.commits {
            ensure!(
                (7..=40).contains(&commit.len())
                    && commit
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
                "{}: invalid commit {commit:?}; use a lowercase abbreviated or full hash",
                update.title
            );
        }
        if let Some(plan) = &update.plan {
            ensure!(
                model
                    .plans
                    .plans
                    .iter()
                    .any(|candidate| &candidate.id == plan),
                "{}: unknown plan {plan}",
                update.title
            );
        }
    }
    Ok(())
}

fn validate_retired(model: &Model) -> Result<()> {
    for retired in &model.inventory.retired {
        for text in [&retired.name, &retired.summary, &retired.evidence] {
            required(text, &retired.id)?;
        }
        ensure!(
            retired.location.starts_with("https://"),
            "{}: retired location must be an https:// URL",
            retired.id
        );
        match &retired.successor {
            None => {}
            Some(Successor::Project(id)) => {
                let project = model
                    .inventory
                    .projects
                    .iter()
                    .find(|project| &project.id == id);
                ensure!(
                    project.is_some_and(|project| !project.lifecycle.historical()),
                    "{}: successor project {id} must exist and be current",
                    retired.id
                );
            }
            Some(Successor::Platform(id)) => ensure!(
                model
                    .toolchains
                    .platforms
                    .iter()
                    .any(|platform| &platform.id == id),
                "{}: successor platform {id} does not exist",
                retired.id
            ),
        }
    }
    Ok(())
}

fn validate_projects(model: &Model) -> Result<()> {
    for source in &model.inventory.sources {
        identifier(&source.id)?;
        local_file(&source.checkout)?;
        ensure!(
            source.repository == "local-only" || source.repository.starts_with("https://"),
            "{}: repository must be https:// or local-only",
            source.id
        );
        for path in source.manifests.iter().chain(&source.evidence_files) {
            local_file(path)?;
        }
        let snapshot = &model.sources[&source.id];
        ensure!(
            snapshot.schema == 1 && snapshot.source == source.id,
            "{}: snapshot identity/schema mismatch",
            source.id
        );
        ensure!(
            snapshot.revision.len() == 40
                && snapshot
                    .revision
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit()),
            "{}: invalid Git revision",
            source.id
        );
        date(&snapshot.observed)?;
        for evidence in &source.evidence_files {
            ensure!(
                snapshot.evidence.contains_key(evidence),
                "{}: missing snapshot evidence {evidence}",
                source.id
            );
        }
    }
    for project in &model.inventory.projects {
        for text in [&project.name, &project.summary, &project.evidence] {
            required(text, &project.id)?;
        }
        ensure!(
            !project.owners.is_empty()
                && !project.languages.is_empty()
                && !project.contribution_areas.is_empty()
                && !project.distribution.is_empty(),
            "{}: owners, languages, contributions and distribution are required",
            project.id
        );
        if project.path != "." {
            local_file(&project.path)?;
        }
        ensure!(
            project.packages.is_empty() || project.domain == Domain::Rust,
            "{}: a component with Cargo packages belongs to the rust domain",
            project.id
        );
        ensure!(
            project.domain != Domain::Rust
                || project.languages.iter().any(|language| language == "Rust"),
            "{}: a rust-domain component must list Rust among its languages",
            project.id
        );
        ensure!(
            project.lifecycle.historical() == (project.maturity == Maturity::Historical),
            "{}: historical lifecycle and maturity must agree",
            project.id
        );
        for url in &project.docs {
            ensure!(
                url.starts_with("https://"),
                "{}: invalid documentation URL {url}",
                project.id
            );
        }
    }
    Ok(())
}

fn validate_edges(model: &Model) -> Result<()> {
    let projects: BTreeMap<_, _> = model
        .inventory
        .projects
        .iter()
        .map(|project| (project.id.as_str(), project))
        .collect();
    let mut edges = BTreeSet::new();
    for edge in &model.edges {
        ensure!(
            projects.contains_key(edge.from.as_str()) && projects.contains_key(edge.to.as_str()),
            "unresolved relationship {} -> {}",
            edge.from,
            edge.to
        );
        ensure!(edge.from != edge.to, "self relationship: {}", edge.from);
        required(&edge.label, &edge.from)?;
        required(&edge.evidence, &edge.from)?;
        ensure!(
            edges.insert(edge),
            "duplicate relationship {} -> {}",
            edge.from,
            edge.to
        );
        if edge.kind == Relationship::Successor {
            ensure!(
                projects[edge.from.as_str()].lifecycle == Lifecycle::Superseded,
                "{}: successor edges require a superseded origin",
                edge.from
            );
        }
    }
    for project in &model.inventory.projects {
        if project.lifecycle == Lifecycle::Superseded {
            ensure!(
                model
                    .edges
                    .iter()
                    .any(|edge| edge.from == project.id && edge.kind == Relationship::Successor),
                "{}: superseded project requires successor evidence",
                project.id
            );
        }
    }
    ensure!(
        model
            .inventory
            .relationships
            .iter()
            .all(|edge| !matches!(edge.kind, Relationship::Rust | Relationship::Build)),
        "Rust and build edges must be derived from source snapshots, not hand-maintained"
    );
    acyclic(
        model
            .edges
            .iter()
            .filter(|edge| edge.kind == Relationship::Successor)
            .map(|edge| (edge.from.as_str(), edge.to.as_str())),
    )?;
    Ok(())
}

fn validate_plans(model: &Model) -> Result<()> {
    let plans: BTreeMap<_, _> = model
        .plans
        .plans
        .iter()
        .map(|plan| (plan.id.as_str(), plan))
        .collect();
    let projects: BTreeSet<_> = model
        .inventory
        .projects
        .iter()
        .map(|project| project.id.as_str())
        .collect();
    let platforms: BTreeSet<_> = model
        .toolchains
        .platforms
        .iter()
        .map(|platform| platform.id.as_str())
        .collect();
    for plan in &model.plans.plans {
        required(&plan.name, &plan.id)?;
        required(&plan.notes, &plan.id)?;
        date(&plan.reviewed)?;
        if let Some(target) = &plan.target_date {
            date(target)?;
        }
        ensure!(
            !plan.projects.is_empty() && !plan.requirements.is_empty(),
            "{}: projects and requirements cannot be empty",
            plan.id
        );
        for project in &plan.projects {
            ensure!(
                projects.contains(project.as_str()),
                "{}: unknown project {project}",
                plan.id
            );
        }
        for platform in &plan.platforms {
            ensure!(
                platforms.contains(platform.as_str()),
                "{}: unknown platform {platform}",
                plan.id
            );
        }
        unique(
            plan.requirements
                .iter()
                .map(|requirement| requirement.id.as_str()),
            "requirement",
        )?;
        validate_plan_requirements(plan)?;
        let blocked = plan
            .requirements
            .iter()
            .any(|requirement| requirement.state == TaskState::Blocked);
        ensure!(
            !blocked || plan.state == State::Blocked,
            "{}: blocked requirement must make the plan blocked",
            plan.id
        );
        let mut blocked_dependency = false;
        for dependency in &plan.depends_on {
            ensure!(
                dependency != &plan.id && plans.contains_key(dependency.as_str()),
                "{}: unresolved/self plan dependency {dependency}",
                plan.id
            );
            let state = plans[dependency.as_str()].state;
            blocked_dependency |= state == State::Blocked;
            if matches!(plan.state, State::Ready | State::Released | State::Done) {
                ensure!(
                    matches!(state, State::Released | State::Done),
                    "{}: prerequisite {dependency} has not completed",
                    plan.id
                );
            }
        }
        ensure!(
            !blocked_dependency || plan.state == State::Blocked,
            "{}: blocked prerequisite must make the plan blocked",
            plan.id
        );
        ensure!(
            plan.state != State::Blocked || blocked || blocked_dependency,
            "{}: blocked plan needs an explicit blocker",
            plan.id
        );
    }
    acyclic(model.plans.plans.iter().flat_map(|plan| {
        plan.depends_on
            .iter()
            .map(move |dependency| (plan.id.as_str(), dependency.as_str()))
    }))
}

fn validate_plan_requirements(plan: &crate::model::Plan) -> Result<()> {
    if plan.kind == PlanKind::Release {
        ensure!(
            plan.projects.len() == 1
                && plan
                    .version
                    .as_ref()
                    .is_some_and(|version| !version.trim().is_empty()),
            "{}: release needs exactly one project and a version",
            plan.id
        );
        for kind in [
            RequirementKind::Ci,
            RequirementKind::Documentation,
            RequirementKind::Artifact,
        ] {
            ensure!(
                plan.requirements
                    .iter()
                    .any(|requirement| requirement.kind == kind),
                "{}: release missing {kind:?} requirement",
                plan.id
            );
        }
    } else {
        ensure!(
            plan.version.is_none(),
            "{}: campaigns use a name, not a release version",
            plan.id
        );
    }
    for requirement in &plan.requirements {
        required(&requirement.label, &plan.id)?;
        required(&requirement.detail, &plan.id)?;
        ensure!(
            requirement.state != TaskState::Done || !requirement.evidence.is_empty(),
            "{}/{}: done requires evidence",
            plan.id,
            requirement.id
        );
        for evidence in &requirement.evidence {
            required(evidence, &plan.id)?;
        }
    }
    if matches!(plan.state, State::Ready | State::Released | State::Done) {
        ensure!(
            plan.requirements
                .iter()
                .all(|requirement| requirement.state == TaskState::Done),
            "{}: ready/completed plan has unfinished requirements",
            plan.id
        );
    }
    ensure!(
        plan.kind != PlanKind::Campaign || plan.state != State::Released,
        "{}: a campaign completes; it is not released",
        plan.id
    );
    ensure!(
        plan.kind != PlanKind::Release || plan.state != State::Done,
        "{}: a release uses released, not done",
        plan.id
    );
    Ok(())
}

fn validate_platforms(model: &Model) -> Result<()> {
    date(&model.toolchains.reviewed)?;
    for platform in &model.toolchains.platforms {
        let workflow = model
            .workflows
            .iter()
            .find(|workflow| workflow.id == platform.workflow);
        ensure!(
            workflow.is_some_and(|workflow| workflow.jobs.contains_key(&platform.job)),
            "{}: platform references unknown workflow/job",
            platform.id
        );
        ensure!(
            matches!(
                platform.validation,
                State::Unknown | State::Blocked | State::Ready
            ),
            "{}: platform validation must be unknown, blocked or ready",
            platform.id
        );
        for assertion in &platform.assertions {
            local_file(&assertion.file)?;
            required(&assertion.contains, &platform.id)?;
        }
    }
    Ok(())
}

/// Reviewed platform baselines must still be true of the executable source they describe.
pub fn policy_assertions(root: &Path, model: &Model) -> Result<()> {
    for platform in &model.toolchains.platforms {
        for assertion in &platform.assertions {
            let text = fs::read_to_string(root.join(&assertion.file))
                .with_context(|| format!("{}: read {}", platform.id, assertion.file))?;
            ensure!(
                text.contains(&assertion.contains),
                "platform {} policy drift: {} no longer contains {:?}; review war-room/toolchains.toml",
                platform.id,
                assertion.file,
                assertion.contains
            );
        }
    }
    Ok(())
}

fn acyclic<'a>(edges: impl Iterator<Item = (&'a str, &'a str)>) -> Result<()> {
    let mut remaining: Vec<_> = edges.collect();
    while !remaining.is_empty() {
        let origins: BTreeSet<_> = remaining.iter().map(|(from, _)| *from).collect();
        let count = remaining.len();
        remaining.retain(|(_, to)| origins.contains(to));
        ensure!(
            remaining.len() < count,
            "cycle in migration or release prerequisites: {remaining:?}"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Edge, Model, Successor};

    /// The committed canonical data, loaded exactly as `check` loads it.
    fn repository_model() -> Model {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let inventory = crate::read_toml(&root.join("war-room/ecosystem.toml")).unwrap();
        crate::model::load(&root, inventory).unwrap()
    }

    /// Break one thing in the real model and require a specific, actionable rejection.
    fn rejects(mutate: impl FnOnce(&mut Model), expected: &str) {
        let mut model = repository_model();
        mutate(&mut model);
        let error = format!(
            "{:#}",
            validate(&model).expect_err("invalid metadata was accepted")
        );
        assert!(
            error.contains(expected),
            "expected an error containing {expected:?}, got {error:?}"
        );
    }

    fn plan<'a>(model: &'a mut Model, id: &str) -> &'a mut crate::model::Plan {
        model
            .plans
            .plans
            .iter_mut()
            .find(|plan| plan.id == id)
            .unwrap()
    }

    #[test]
    fn committed_metadata_is_valid() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let model = repository_model();
        validate(&model).unwrap();
        policy_assertions(&root, &model).unwrap();
    }

    #[test]
    fn dates_and_cycles_are_validated() {
        assert!(date("2026-02-29").is_err());
        assert!(date("2024-02-29").is_ok());
        assert!(date("2026-9-28").is_err());
        assert!(acyclic([("a", "b"), ("b", "a")].into_iter()).is_err());
        assert!(acyclic([("a", "b"), ("b", "c")].into_iter()).is_ok());
    }

    #[test]
    fn duplicate_project_ids_fail() {
        rejects(
            |model| model.inventory.projects[1].id = model.inventory.projects[0].id.clone(),
            "duplicate project or retired ID",
        );
    }

    #[test]
    fn retired_ids_share_the_project_namespace() {
        rejects(
            |model| model.inventory.retired[0].id = "l3i".into(),
            "duplicate project or retired ID: l3i",
        );
    }

    #[test]
    fn broken_relationship_references_fail() {
        rejects(
            |model| model.edges[0].to = "no-such-project".into(),
            "unresolved relationship",
        );
    }

    #[test]
    fn unsupported_relationship_types_fail_to_parse() {
        let edge = "from = \"a\"\nto = \"b\"\nkind = \"vibes\"\nlabel = \"x\"\nevidence = \"y\"\n";
        assert!(toml::from_str::<Edge>(edge).is_err());
    }

    #[test]
    fn unknown_fields_fail_to_parse() {
        let edge = "from = \"a\"\nto = \"b\"\nkind = \"rust\"\nlabel = \"x\"\nevidence = \"y\"\nweight = 3\n";
        assert!(toml::from_str::<Edge>(edge).is_err());
    }

    #[test]
    fn hand_maintained_rust_edges_fail() {
        rejects(
            |model| model.inventory.relationships[0].kind = Relationship::Rust,
            "must be derived from source snapshots",
        );
    }

    #[test]
    fn superseded_projects_need_a_successor() {
        rejects(
            |model| {
                model
                    .edges
                    .retain(|edge| edge.kind != Relationship::Successor);
            },
            "superseded project requires successor evidence",
        );
    }

    #[test]
    fn plans_must_reference_real_projects() {
        rejects(
            |model| {
                plan(model, "war-room")
                    .projects
                    .push("no-such-project".into());
            },
            "unknown project no-such-project",
        );
    }

    #[test]
    fn a_blocked_requirement_blocks_its_plan() {
        rejects(
            |model| plan(model, "llvm-rollout").state = State::Active,
            "blocked requirement must make the plan blocked",
        );
    }

    #[test]
    fn a_blocked_plan_needs_a_blocker() {
        rejects(
            |model| plan(model, "war-room").state = State::Blocked,
            "blocked plan needs an explicit blocker",
        );
    }

    #[test]
    fn ready_plans_cannot_have_unfinished_requirements() {
        rejects(
            |model| plan(model, "war-room").state = State::Ready,
            "unfinished requirements",
        );
    }

    #[test]
    fn done_requirements_need_evidence() {
        rejects(
            |model| {
                let requirement = &mut plan(model, "war-room").requirements[0];
                requirement.state = TaskState::Done;
                requirement.evidence.clear();
            },
            "done requires evidence",
        );
    }

    #[test]
    fn plan_prerequisite_cycles_fail() {
        rejects(
            |model| {
                // Blocked by its prerequisite, so the only defect left is the cycle.
                plan(model, "war-room").state = State::Blocked;
                plan(model, "war-room")
                    .depends_on
                    .push("llvm-rollout".into());
                plan(model, "llvm-rollout")
                    .depends_on
                    .push("war-room".into());
            },
            "cycle in migration or release prerequisites",
        );
    }

    #[test]
    fn releases_need_ci_documentation_and_artifact_requirements() {
        rejects(
            |model| {
                let release = plan(model, "war-room");
                release.kind = PlanKind::Release;
                release.projects.truncate(1);
                release.version = Some("1.0.0".into());
            },
            "release missing",
        );
    }

    #[test]
    fn platforms_must_reference_real_jobs() {
        rejects(
            |model| model.toolchains.platforms[0].job = "release-amiga".into(),
            "platform references unknown workflow/job",
        );
    }

    #[test]
    fn supply_chain_stages_must_reference_real_jobs() {
        rejects(
            |model| {
                model.operations.stage[1]
                    .jobs
                    .push("rustGlobalBuild/deploy-to-mars".into());
            },
            "has no job named",
        );
    }

    #[test]
    fn callable_workflows_need_operational_notes() {
        rejects(
            |model| {
                model.operations.workflow.remove("rustGlobalBuild");
            },
            "rustGlobalBuild: callable workflow/action requires notes",
        );
    }

    #[test]
    fn archived_repositories_must_be_documented_as_retired() {
        rejects(
            |model| {
                model
                    .inventory
                    .retired
                    .retain(|retired| retired.id != "dream-dice");
            },
            "dreamDice is archived on GitHub but has no [[retired]] record",
        );
    }

    #[test]
    fn current_projects_cannot_live_in_archived_repositories() {
        rejects(
            |model| {
                let url = model.repository(model.project("vfstool")).to_owned();
                for repository in &mut model.organization.repositories {
                    if repository.url == url {
                        repository.archived = true;
                    }
                }
                model.inventory.retired.push(crate::model::Retired {
                    id: "vfstool-archive".into(),
                    name: "vfstool".into(),
                    kind: crate::model::RetiredKind::Repository,
                    summary: "x".into(),
                    location: url,
                    successor: None,
                    evidence: "x".into(),
                });
            },
            "but GitHub reports vfstool archived",
        );
    }

    #[test]
    fn retired_successors_must_exist() {
        rejects(
            |model| {
                model.inventory.retired[0].successor = Some(Successor::Platform("amiga".into()));
            },
            "successor platform amiga does not exist",
        );
    }

    #[test]
    fn updates_must_be_newest_first_with_real_plans() {
        rejects(
            |model| model.inventory.updates.reverse(),
            "must be listed newest first",
        );
        rejects(
            |model| model.inventory.updates[0].plan = Some("no-such-plan".into()),
            "unknown plan no-such-plan",
        );
        rejects(
            |model| model.inventory.updates[0].commits = vec!["HEAD~1".into()],
            "invalid commit",
        );
    }

    #[test]
    fn cargo_packages_belong_to_the_rust_domain() {
        rejects(
            |model| {
                model
                    .inventory
                    .projects
                    .iter_mut()
                    .find(|project| project.id == "dream-path")
                    .unwrap()
                    .domain = Domain::OpenmwLua;
            },
            "belongs to the rust domain",
        );
    }

    #[test]
    fn excluded_repositories_are_never_recorded() {
        rejects(
            |model| {
                model.organization.repositories[0].name = model.census_exclusions[0].clone();
            },
            "contains an excluded repository",
        );
    }
}
