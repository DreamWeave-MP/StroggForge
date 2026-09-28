use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};

use crate::model::{Edge, Inventory, Relationship};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub schema: u32,
    pub source: String,
    pub revision: String,
    pub dirty: bool,
    pub observed: String,
    pub packages: Vec<Package>,
    pub workflows: BTreeMap<String, String>,
    pub evidence: BTreeMap<String, String>,
}

// Cargo owns this input schema; retain the fields the war room actually consumes.
#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub rust_version: Option<String>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Dependency {
    pub name: String,
    pub req: String,
    pub kind: Option<String>,
    pub optional: bool,
    pub target: Option<String>,
    pub features: Vec<String>,
}

/// Every repository in the GitHub organization, captured explicitly with `gh`. The census
/// proves coverage: an archived repository without a retired record fails validation.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    pub schema: u32,
    pub name: String,
    pub observed: String,
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Repository {
    pub name: String,
    pub url: String,
    pub archived: bool,
    pub fork: bool,
    pub pushed: String,
    pub description: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GithubRepository {
    name: String,
    url: String,
    is_archived: bool,
    is_fork: bool,
    pushed_at: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
}

fn command(directory: &Path, program: &str, arguments: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(directory)
        .output()
        .with_context(|| format!("run {program} in {}", directory.display()))?;
    ensure!(
        output.status.success(),
        "{program} {} in {}: {}",
        arguments.join(" "),
        directory.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).context("command output is not UTF-8")
}

pub fn capture(root: &Path, clones: &Path, id: &str, inventory: &Inventory) -> Result<()> {
    let source = inventory
        .sources
        .iter()
        .find(|source| source.id == id)
        .with_context(|| format!("unknown source {id}"))?;
    let checkout = clones.join(&source.checkout);
    let mut packages = BTreeMap::new();
    for manifest in &source.manifests {
        let text = command(
            &checkout,
            "cargo",
            &[
                "metadata",
                "--no-deps",
                "--offline",
                "--format-version",
                "1",
                "--manifest-path",
                manifest,
            ],
        )?;
        for mut package in serde_json::from_str::<Metadata>(&text)?.packages {
            package
                .dependencies
                .sort_by(|a, b| (&a.name, &a.kind, &a.target).cmp(&(&b.name, &b.kind, &b.target)));
            ensure!(
                !packages.contains_key(&package.name),
                "{id}: duplicate package {} across manifest roots",
                package.name
            );
            packages.insert(package.name.clone(), package);
        }
    }
    let mut workflows = BTreeMap::new();
    let workflow_directory = checkout.join(".github/workflows");
    if id != "stroggforge" && workflow_directory.exists() {
        for entry in fs::read_dir(workflow_directory)? {
            let path = entry?.path();
            if matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("yml" | "yaml")
            ) {
                workflows.insert(
                    format!(
                        ".github/workflows/{}",
                        path.file_name().unwrap().to_string_lossy()
                    ),
                    fs::read_to_string(path)?,
                );
            }
        }
    }
    let evidence = source
        .evidence_files
        .iter()
        .map(|file| {
            Ok((
                file.clone(),
                fs::read_to_string(checkout.join(file))
                    .with_context(|| format!("{id}: read evidence {file}"))?,
            ))
        })
        .collect::<Result<_>>()?;
    let snapshot = Snapshot {
        schema: 1,
        source: id.to_owned(),
        revision: command(&checkout, "git", &["rev-parse", "HEAD"])?
            .trim()
            .to_owned(),
        dirty: !command(&checkout, "git", &["status", "--porcelain"])?
            .trim()
            .is_empty(),
        observed: inventory.reviewed.clone(),
        packages: packages.into_values().collect(),
        workflows,
        evidence,
    };
    let directory = root.join("war-room/sources");
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join(format!("{id}.json")),
        format!("{}\n", serde_json::to_string_pretty(&snapshot)?),
    )?;
    println!(
        "Captured {id}: {} packages, {} workflows; dirty={}",
        snapshot.packages.len(),
        snapshot.workflows.len(),
        snapshot.dirty
    );
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CensusExclusions {
    repositories: Vec<String>,
}

/// Repositories deliberately kept out of the census. Never part of the published model.
pub fn census_exclusions(root: &Path) -> Result<Vec<String>> {
    let exclusions: CensusExclusions =
        crate::read_toml(&root.join("war-room/census-exclusions.toml"))?;
    Ok(exclusions.repositories)
}

pub fn capture_organization(root: &Path, inventory: &Inventory) -> Result<()> {
    let text = command(
        root,
        "gh",
        &[
            "repo",
            "list",
            &inventory.organization,
            "--limit",
            "1000",
            "--json",
            "name,url,isArchived,isFork,pushedAt,description",
        ],
    )?;
    let excluded = census_exclusions(root)?;
    let mut repositories: Vec<Repository> = serde_json::from_str::<Vec<GithubRepository>>(&text)?
        .into_iter()
        .filter(|repository| !excluded.contains(&repository.name))
        .map(|repository| Repository {
            pushed: repository.pushed_at.chars().take(10).collect(),
            name: repository.name,
            url: repository.url,
            archived: repository.is_archived,
            fork: repository.is_fork,
            description: repository.description.unwrap_or_default(),
        })
        .collect();
    repositories.sort_by_key(|repository| repository.name.to_lowercase());
    let organization = Organization {
        schema: 1,
        name: inventory.organization.clone(),
        observed: inventory.reviewed.clone(),
        repositories,
    };
    fs::write(
        root.join("war-room/sources/organization.json"),
        format!("{}\n", serde_json::to_string_pretty(&organization)?),
    )?;
    println!(
        "Captured {}: {} repositories, {} archived",
        organization.name,
        organization.repositories.len(),
        organization
            .repositories
            .iter()
            .filter(|repository| repository.archived)
            .count()
    );
    Ok(())
}

pub fn derive_edges(
    inventory: &Inventory,
    sources: &BTreeMap<String, Snapshot>,
) -> Result<Vec<Edge>> {
    let mut owners = BTreeMap::new();
    for project in &inventory.projects {
        for package in &project.packages {
            ensure!(
                owners
                    .insert(package.as_str(), project.id.as_str())
                    .is_none(),
                "package {package} belongs to more than one project"
            );
        }
    }
    let mut edges: BTreeSet<Edge> = inventory.relationships.iter().cloned().collect();
    ensure!(
        edges.len() == inventory.relationships.len(),
        "duplicate explicit relationship"
    );
    for project in &inventory.projects {
        let snapshot = sources
            .get(&project.source)
            .with_context(|| format!("{}: unknown source {}", project.id, project.source))?;
        for name in &project.packages {
            let package = snapshot
                .packages
                .iter()
                .find(|package| &package.name == name)
                .with_context(|| {
                    format!(
                        "{}: package {name} absent from snapshot {}",
                        project.id, project.source
                    )
                })?;
            for dependency in &package.dependencies {
                if let Some(owner) = owners.get(dependency.name.as_str())
                    && *owner != project.id
                {
                    edges.insert(Edge {
                        from: project.id.clone(),
                        to: (*owner).to_owned(),
                        kind: Relationship::Rust,
                        label: format!(
                            "{} {}{}{}{}",
                            dependency.name,
                            dependency.req,
                            dependency
                                .kind
                                .as_ref()
                                .map_or(String::new(), |kind| format!(" ({kind})")),
                            if dependency.optional {
                                " (optional)"
                            } else {
                                ""
                            },
                            dependency
                                .target
                                .as_ref()
                                .map_or(String::new(), |target| format!(" [{target}]"))
                        ),
                        evidence: format!("{} Cargo metadata: {}", project.source, package.name),
                    });
                }
            }
        }
        // Repository-level workflow observations apply to that repository's components.
        if project.id != "stroggforge" && !project.lifecycle.historical() {
            for (file, text) in &snapshot.workflows {
                for call in
                    workflow_calls(text).with_context(|| format!("{}:{file}", project.source))?
                {
                    edges.insert(Edge {
                        from: project.id.clone(),
                        to: "stroggforge".into(),
                        kind: Relationship::Build,
                        label: format!("repository calls {call}"),
                        evidence: format!("{}:{file}", project.source),
                    });
                }
            }
        }
    }
    Ok(edges.into_iter().collect())
}

/// Current projects grouped by the StroggForge workflow their repository calls:
/// `{workflow id: {project id}}`. Uses the same parser as the derived build edges.
pub fn callers(
    inventory: &Inventory,
    sources: &BTreeMap<String, Snapshot>,
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut callers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for project in &inventory.projects {
        if project.id == "stroggforge" || project.lifecycle.historical() {
            continue;
        }
        for (file, text) in &sources[&project.source].workflows {
            for call in
                workflow_calls(text).with_context(|| format!("{}:{file}", project.source))?
            {
                let (workflow, _) = call.split_once('@').expect("workflow_calls requires a ref");
                let workflow = workflow
                    .strip_suffix(".yml")
                    .or_else(|| workflow.strip_suffix(".yaml"))
                    .with_context(|| {
                        format!(
                            "{}:{file}: unexpected workflow file {workflow}",
                            project.source
                        )
                    })?;
                callers
                    .entry(workflow.to_owned())
                    .or_default()
                    .insert(project.id.clone());
            }
        }
    }
    Ok(callers)
}

pub fn workflow_calls(text: &str) -> Result<Vec<String>> {
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(text)?;
    let Some(jobs) = value.get("jobs").and_then(serde_yaml_ng::Value::as_mapping) else {
        return Ok(Vec::new());
    };
    let mut calls = Vec::new();
    for job in jobs.values() {
        if let Some(uses) = job.get("uses").and_then(serde_yaml_ng::Value::as_str)
            && let Some(call) = uses.strip_prefix("DreamWeave-MP/StroggForge/.github/workflows/")
        {
            ensure!(call.contains('@'), "StroggForge call has no ref: {call}");
            calls.push(call.to_owned());
        }
    }
    Ok(calls)
}

pub fn local_file(path: &str) -> Result<()> {
    if path.is_empty()
        || Path::new(path).is_absolute()
        || Path::new(path)
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        bail!("expected a safe repository-relative path, got {path:?}");
    }
    Ok(())
}
