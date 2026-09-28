use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::workflows::{Operations, Workflow};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ecosystem {
    pub schema: u32,
    pub reviewed: String,
    pub scope: String,
    pub updates: Vec<Update>,
    pub projects: Vec<Project>,
    pub relationships: Vec<Relationship>,
    pub retired: Vec<Retired>,
}

/// An infrastructure change that alters what consumers get; Git holds the full history.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Update {
    pub date: String,
    pub title: String,
    pub detail: String,
    pub commits: Vec<String>,
    pub plan: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub kind: Kind,
    pub domain: Domain,
    pub summary: String,
    /// Absent when the project has no public repository yet.
    pub repository: Option<String>,
    pub languages: Vec<String>,
    pub lifecycle: Lifecycle,
    pub maturity: Maturity,
    pub owners: Vec<String>,
    pub docs: Vec<String>,
    pub packages: Vec<String>,
    pub distribution: Vec<Distribution>,
    /// StroggForge workflows this project's repository calls.
    pub stroggforge: Vec<String>,
    pub approach: Approach,
    pub contribution_areas: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    Foundation,
    Application,
    DeveloperTool,
    Runtime,
    Infrastructure,
    Documentation,
    Content,
}

impl Kind {
    pub const ALL: [Self; 7] = [
        Self::Foundation,
        Self::Application,
        Self::DeveloperTool,
        Self::Runtime,
        Self::Infrastructure,
        Self::Documentation,
        Self::Content,
    ];
}

/// The side of DreamWeave a component belongs to. StroggForge serves the Rust side; the
/// Lua(u)/OpenMW content side and the web sites are documented here without joining it.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
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
    Source,
}

/// A reviewed relationship. Arrows point consumer -> dependency.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relationship {
    pub from: String,
    pub to: String,
    pub kind: RelationshipKind,
    pub label: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum RelationshipKind {
    Rust,
    Documentation,
    Contains,
    Successor,
}

/// Retired infrastructure whose replacement explains the current architecture.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Retired {
    pub id: String,
    pub name: String,
    pub kind: RetiredKind,
    pub summary: String,
    pub location: String,
    pub successor: Option<Successor>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RetiredKind {
    BuildEnvironment,
    Site,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Successor {
    Project(String),
    Platform(String),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plans {
    pub plans: Vec<Plan>,
}

#[derive(Debug, Deserialize)]
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

impl State {
    pub fn finished(self) -> bool {
        matches!(self, Self::Released | Self::Done)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub id: String,
    pub kind: RequirementKind,
    pub label: String,
    pub state: TaskState,
    pub detail: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Toolchains {
    pub reviewed: String,
    pub rust_policy: String,
    pub llvm_policy: String,
    pub reference: String,
    pub platforms: Vec<Platform>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Platform {
    pub id: String,
    pub name: String,
    pub target: String,
    pub workflow: String,
    pub job: String,
    pub baseline: String,
    pub compiler: String,
    pub validation: Validation,
    pub notes: String,
}

/// Whether a configured platform has been proven, as opposed to merely configured.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Validation {
    Unknown,
    Blocked,
    Ready,
}

pub struct Model {
    pub ecosystem: Ecosystem,
    pub plans: Vec<Plan>,
    pub toolchains: Toolchains,
    pub operations: Operations,
    pub workflows: Vec<Workflow>,
}

impl Model {
    pub fn load(root: &Path) -> Result<Self> {
        let plans: Plans = read_toml(&root.join("war-room/plans.toml"))?;
        Ok(Self {
            ecosystem: read_toml(&root.join("war-room/ecosystem.toml"))?,
            plans: plans.plans,
            toolchains: read_toml(&root.join("war-room/toolchains.toml"))?,
            operations: read_toml(&root.join("war-room/workflows.toml"))?,
            workflows: crate::workflows::load(root)?,
        })
    }

    pub fn project(&self, id: &str) -> &Project {
        self.ecosystem
            .projects
            .iter()
            .find(|project| project.id == id)
            .expect("project references are validated before generation")
    }

    pub fn platform(&self, id: &str) -> &Platform {
        self.toolchains
            .platforms
            .iter()
            .find(|platform| platform.id == id)
            .expect("platform references are validated before generation")
    }

    pub fn workflow(&self, id: &str) -> &Workflow {
        self.workflows
            .iter()
            .find(|workflow| workflow.id == id)
            .expect("workflow references are validated before generation")
    }

    /// Current projects grouped by the StroggForge workflow their repository calls.
    pub fn callers(&self) -> BTreeMap<&str, Vec<&Project>> {
        let mut callers: BTreeMap<&str, Vec<&Project>> = BTreeMap::new();
        for project in self.current_projects() {
            for workflow in &project.stroggforge {
                callers.entry(workflow).or_default().push(project);
            }
        }
        callers
    }

    pub fn current_projects(&self) -> impl Iterator<Item = &Project> {
        self.ecosystem
            .projects
            .iter()
            .filter(|project| !project.lifecycle.historical())
    }
}

fn read_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

/// The kebab-case spelling of a closed-set value, as written in the TOML.
pub fn label(value: &impl Serialize) -> String {
    serde_json::to_value(value)
        .expect("enum serialization")
        .as_str()
        .expect("string enum")
        .to_owned()
}
