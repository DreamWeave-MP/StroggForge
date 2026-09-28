use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{
    sources::{Organization, Snapshot},
    workflows::{Operations, Workflow},
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    pub schema: u32,
    pub organization: String,
    pub reviewed: String,
    pub scope: String,
    pub updates: Vec<Update>,
    pub sources: Vec<Source>,
    pub projects: Vec<Project>,
    #[serde(default)]
    pub relationships: Vec<Edge>,
    pub retired: Vec<Retired>,
}

/// A reviewed infrastructure change worth surfacing on the war room; Git holds the full history.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Update {
    pub date: String,
    pub title: String,
    pub detail: String,
    pub commits: Vec<String>,
    pub plan: Option<String>,
}

/// Retired infrastructure that has no current source checkout: archived repositories,
/// replaced build environments and replaced sites. Superseded components that still have
/// a checkout remain ordinary projects with a superseded lifecycle.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Retired {
    pub id: String,
    pub name: String,
    pub kind: RetiredKind,
    pub summary: String,
    pub location: String,
    pub successor: Option<Successor>,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum RetiredKind {
    Repository,
    Organization,
    BuildEnvironment,
    Site,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Successor {
    Project(String),
    Platform(String),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub checkout: String,
    pub repository: String,
    pub manifests: Vec<String>,
    pub evidence_files: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub kind: Kind,
    pub domain: Domain,
    pub summary: String,
    pub source: String,
    pub path: String,
    pub languages: Vec<String>,
    pub lifecycle: Lifecycle,
    pub maturity: Maturity,
    pub owners: Vec<String>,
    pub docs: Vec<String>,
    pub packages: Vec<String>,
    pub distribution: Vec<Distribution>,
    pub approach: Approach,
    pub contribution_areas: Vec<String>,
    pub tags: Vec<String>,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    Foundation,
    Application,
    DeveloperTool,
    Infrastructure,
    Documentation,
    Runtime,
    Content,
}

/// The side of DreamWeave a component belongs to. StroggForge serves the Rust side; the
/// Lua(u)/OpenMW content side and the web sites are documented here without joining it.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Domain {
    Rust,
    OpenmwLua,
    Web,
}

impl Domain {
    pub const ALL: [Self; 3] = [Self::Rust, Self::OpenmwLua, Self::Web];

    pub fn title(self) -> &'static str {
        match self {
            Self::Rust => "Rust ecosystem",
            Self::OpenmwLua => "Lua(u) / OpenMW",
            Self::Web => "Web and documentation sites",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Lifecycle {
    Active,
    Experimental,
    Maintenance,
    Superseded,
    Archived,
}

impl Lifecycle {
    pub fn historical(self) -> bool {
        matches!(self, Self::Superseded | Self::Archived)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Maturity {
    Evolving,
    Established,
    Historical,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Approach {
    Introductory,
    Intermediate,
    Specialist,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Distribution {
    CratesIo,
    GithubReleases,
    GithubPages,
    Aur,
    Nexus,
    Source,
    Unknown,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: Relationship,
    pub label: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Relationship {
    Rust,
    Build,
    Distribution,
    Documentation,
    Contains,
    Successor,
    Runtime,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plans {
    pub plans: Vec<Plan>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub id: String,
    pub kind: PlanKind,
    pub name: String,
    pub projects: Vec<String>,
    pub version: Option<String>,
    pub state: State,
    pub reviewed: String,
    pub target_date: Option<String>,
    pub depends_on: Vec<String>,
    pub platforms: Vec<String>,
    pub distribution: Vec<Distribution>,
    pub notes: String,
    pub links: Vec<String>,
    pub requirements: Vec<Requirement>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PlanKind {
    Release,
    Campaign,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    Unknown,
    Preparing,
    Active,
    Blocked,
    Ready,
    Released,
    Done,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub id: String,
    pub kind: RequirementKind,
    pub label: String,
    pub state: TaskState,
    pub detail: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum RequirementKind {
    Task,
    Ci,
    Documentation,
    Artifact,
    Platform,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TaskState {
    Todo,
    Active,
    Blocked,
    Done,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Toolchains {
    pub reviewed: String,
    pub rust_policy: String,
    pub llvm_policy: String,
    pub reference: String,
    pub platforms: Vec<Platform>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Platform {
    pub id: String,
    pub name: String,
    pub target: String,
    pub workflow: String,
    pub job: String,
    pub baseline: String,
    pub compiler: String,
    pub validation: State,
    pub notes: String,
    pub assertions: Vec<Assertion>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Assertion {
    pub file: String,
    pub contains: String,
}

#[derive(Debug, Serialize)]
pub struct Model {
    pub inventory: Inventory,
    pub sources: BTreeMap<String, Snapshot>,
    /// Raw observation, including GitHub's own descriptions; the census page publishes a
    /// reviewed view of it instead.
    #[serde(skip_serializing)]
    pub organization: Organization,
    #[serde(skip_serializing)]
    pub census_exclusions: Vec<String>,
    pub edges: Vec<Edge>,
    /// Current projects grouped by the StroggForge workflow their repository calls.
    pub callers: BTreeMap<String, BTreeSet<String>>,
    pub plans: Plans,
    pub toolchains: Toolchains,
    pub workflows: Vec<Workflow>,
    pub operations: Operations,
}

impl Model {
    pub fn project(&self, id: &str) -> &Project {
        self.inventory
            .projects
            .iter()
            .find(|project| project.id == id)
            .expect("project references validated before generation")
    }

    pub fn platform(&self, id: &str) -> &Platform {
        self.toolchains
            .platforms
            .iter()
            .find(|platform| platform.id == id)
            .expect("platform references validated before generation")
    }

    pub fn repository(&self, project: &Project) -> &str {
        &self
            .inventory
            .sources
            .iter()
            .find(|source| source.id == project.source)
            .expect("source reference validated")
            .repository
    }
}

pub fn load(root: &Path, inventory: Inventory) -> Result<Model> {
    let mut sources = BTreeMap::new();
    for source in &inventory.sources {
        let path = root.join(format!("war-room/sources/{}.json", source.id));
        let snapshot: Snapshot =
            serde_json::from_str(&fs::read_to_string(&path).with_context(|| {
                format!(
                    "read {}; use capture to refresh this source",
                    path.display()
                )
            })?)
            .with_context(|| format!("parse {}", path.display()))?;
        sources.insert(source.id.clone(), snapshot);
    }
    let edges = crate::sources::derive_edges(&inventory, &sources)?;
    let callers = crate::sources::callers(&inventory, &sources)?;
    Ok(Model {
        inventory,
        sources,
        organization: {
            let path = root.join("war-room/sources/organization.json");
            serde_json::from_str(
                &fs::read_to_string(&path).with_context(|| {
                    format!("read {}; run capture-organization", path.display())
                })?,
            )
            .with_context(|| format!("parse {}", path.display()))?
        },
        census_exclusions: crate::sources::census_exclusions(root)?,
        edges,
        callers,
        plans: crate::read_toml(&root.join("war-room/plans.toml"))?,
        toolchains: crate::read_toml(&root.join("war-room/toolchains.toml"))?,
        workflows: crate::workflows::load(root)?,
        operations: crate::read_toml(&root.join(".github/war-room-workflows.toml"))?,
    })
}

pub fn label(value: &impl Serialize) -> String {
    serde_json::to_value(value)
        .expect("enum serialization")
        .as_str()
        .expect("string enum")
        .to_owned()
}
