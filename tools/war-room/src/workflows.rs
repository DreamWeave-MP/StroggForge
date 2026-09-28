use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_yaml_ng::Value;

/// A workflow or composite action as it exists in this checkout.
#[derive(Debug)]
pub struct Workflow {
    pub id: String,
    pub path: String,
    pub name: String,
    pub callable: bool,
    pub action: bool,
    pub inputs: BTreeMap<String, Value>,
    pub secrets: BTreeMap<String, Value>,
    pub outputs: BTreeMap<String, Value>,
    pub permissions: Value,
    pub jobs: BTreeMap<String, Job>,
    pub steps: Vec<Value>,
}

#[derive(Debug, Deserialize)]
pub struct Job {
    #[serde(default)]
    pub needs: Value,
    #[serde(rename = "if", default)]
    pub condition: Value,
    #[serde(rename = "runs-on", default)]
    pub runner: Value,
    #[serde(default)]
    pub container: Value,
    #[serde(default)]
    pub strategy: Value,
    #[serde(default)]
    pub uses: Value,
    #[serde(default)]
    pub steps: Vec<Value>,
}

impl Job {
    pub fn dependencies(&self) -> Result<Vec<String>> {
        match &self.needs {
            Value::Null => Ok(Vec::new()),
            Value::String(value) => Ok(vec![value.clone()]),
            Value::Sequence(values) => values
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(str::to_owned)
                        .context("needs must contain job names")
                })
                .collect(),
            _ => anyhow::bail!("unsupported job needs value: {:?}", self.needs),
        }
    }
}

/// What YAML cannot say about the workflows: `war-room/workflows.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operations {
    pub workflow: BTreeMap<String, Note>,
    pub stage: Vec<Stage>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Note {
    pub purpose: String,
    pub intended_for: String,
    pub release_behavior: String,
    pub artifacts: Vec<String>,
    pub assumptions: Vec<String>,
    pub failures: Vec<String>,
    pub example: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    pub id: String,
    pub name: String,
    pub kind: StageKind,
    pub summary: String,
    pub jobs: Vec<String>,
}

/// How a stage is drawn: the node kinds the schematic shortcode styles.
#[derive(Debug, Clone, Copy, Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StageKind {
    Source,
    Tool,
    Boundary,
    Component,
    Service,
}

/// Resolve `<workflow>/<job>` or `<action>/<step name>` against the executable YAML.
pub fn resolve<'a>(
    workflows: &'a [Workflow],
    reference: &'a str,
) -> Result<(&'a Workflow, &'a str)> {
    let (id, name) = reference
        .split_once('/')
        .with_context(|| format!("{reference:?}: expected <workflow>/<job> or <action>/<step>"))?;
    let workflow = workflows
        .iter()
        .find(|workflow| workflow.id == id)
        .with_context(|| format!("{reference:?}: no workflow or action named {id}"))?;
    let found = if workflow.action {
        workflow
            .steps
            .iter()
            .any(|step| step.get("name").and_then(Value::as_str) == Some(name))
    } else {
        workflow.jobs.contains_key(name)
    };
    ensure!(
        found,
        "{reference:?}: {id} has no {} named {name:?}",
        if workflow.action { "step" } else { "job" }
    );
    Ok((workflow, name))
}

fn mapping(value: &Value, name: &str) -> Result<BTreeMap<String, Value>> {
    value.get(name).map_or_else(
        || Ok(BTreeMap::new()),
        |value| {
            serde_yaml_ng::from_value(value.clone()).context("workflow contract must be a mapping")
        },
    )
}

pub fn load(root: &Path) -> Result<Vec<Workflow>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(root.join(".github/workflows"))? {
        let path = entry?.path();
        if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("yml" | "yaml")
        ) {
            files.push(path);
        }
    }
    for entry in fs::read_dir(root.join(".github/actions"))? {
        let path = entry?.path().join("action.yml");
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let value: Value = serde_yaml_ng::from_str(&fs::read_to_string(&path)?)
                .with_context(|| format!("parse workflow {}", path.display()))?;
            let action = path.file_name().unwrap() == "action.yml";
            let id = if action {
                path.parent().unwrap().file_name().unwrap()
            } else {
                path.file_stem().unwrap()
            }
            .to_string_lossy()
            .into_owned();
            let call = value.get("on").and_then(|on| on.get("workflow_call"));
            let contract = if action {
                &value
            } else {
                call.unwrap_or(&Value::Null)
            };
            let jobs: BTreeMap<String, Job> = value.get("jobs").map_or_else(
                || Ok(BTreeMap::new()),
                |jobs| serde_yaml_ng::from_value(jobs.clone()),
            )?;
            for (name, job) in &jobs {
                for needed in job.dependencies()? {
                    ensure!(
                        jobs.contains_key(&needed),
                        "{id}/{name}: missing needed job {needed}"
                    );
                }
            }
            Ok(Workflow {
                id,
                path: path.strip_prefix(root)?.to_string_lossy().into_owned(),
                name: value
                    .get("name")
                    .and_then(Value::as_str)
                    .context("workflow/action requires a name")?
                    .into(),
                callable: call.is_some(),
                action,
                inputs: mapping(contract, "inputs")?,
                secrets: mapping(contract, "secrets")?,
                outputs: mapping(contract, "outputs")?,
                permissions: value.get("permissions").cloned().unwrap_or_default(),
                jobs,
                steps: value
                    .get("runs")
                    .and_then(|runs| runs.get("steps"))
                    .and_then(Value::as_sequence)
                    .cloned()
                    .unwrap_or_default(),
            })
        })
        .collect()
}

pub fn text(value: &Value) -> String {
    if value.is_null() {
        return "—".into();
    }
    if let Some(value) = value.as_str() {
        return value.to_owned();
    }
    serde_json::to_string(value).expect("YAML workflow value is JSON-compatible")
}
