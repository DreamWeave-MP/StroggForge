//! `inspect`: the workflow and composite action contracts as this checkout's YAML states them,
//! as JSON Lines in the `stroggforge.inspect/1` shape. It is the reference the JSL forge's own
//! `forge inspect` is compared against (FOR-101/FOR-201), so its rules are written for a reader
//! that has only Lua tables:
//!
//! - one compact JSON object per line: a header, then one line per file, ordered by path;
//! - object keys sorted bytewise, which is what `serde_json`'s default map does;
//! - an empty mapping is written `[]`, because a decoded Lua table cannot say which it was;
//! - a step keeps `id`, `name`, `uses`, `if`, `shell`, `with`, `env` and `working-directory`,
//!   and says whether it has a `run` script without carrying the script.
use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde_json::{Map, Value as Json, json};
use serde_yaml_ng::Value;

use crate::workflows::Workflow;

pub const SCHEMA: &str = "stroggforge.inspect/1";

const STEP_KEYS: [&str; 8] = [
    "id",
    "name",
    "uses",
    "if",
    "shell",
    "with",
    "env",
    "working-directory",
];

pub fn inspect(root: &Path, workflows: &[Workflow]) -> Result<String> {
    let mut ordered: Vec<&Workflow> = workflows.iter().collect();
    ordered.sort_by(|left, right| left.path.cmp(&right.path));

    let mut lines = vec![serde_json::to_string(
        &json!({ "schema": SCHEMA, "files": ordered.len() }),
    )?];
    for workflow in ordered {
        let text = fs::read_to_string(root.join(&workflow.path))
            .with_context(|| format!("read {}", workflow.path))?;
        let document: Value =
            serde_yaml_ng::from_str(&text).with_context(|| format!("parse {}", workflow.path))?;
        lines.push(serde_json::to_string(&file(workflow, &document)?)?);
    }
    Ok(lines.join("\n") + "\n")
}

fn file(workflow: &Workflow, document: &Value) -> Result<Json> {
    let mut out = Map::new();
    out.insert("id".into(), workflow.id.clone().into());
    out.insert("path".into(), workflow.path.clone().into());
    out.insert(
        "kind".into(),
        if workflow.action {
            "action"
        } else {
            "workflow"
        }
        .into(),
    );
    out.insert("name".into(), workflow.name.clone().into());
    out.insert("callable".into(), workflow.callable.into());

    if workflow.action {
        let runs = document.get("runs");
        out.insert("inputs".into(), raw(document.get("inputs"))?);
        out.insert("outputs".into(), raw(document.get("outputs"))?);
        out.insert(
            "using".into(),
            raw(runs.and_then(|runs| runs.get("using")))?,
        );
        out.insert(
            "steps".into(),
            steps(runs.and_then(|runs| runs.get("steps")))?,
        );
        return Ok(Json::Object(out));
    }

    let on = document.get("on");
    let call = on.and_then(|on| on.get("workflow_call"));
    out.insert("events".into(), events(on));
    out.insert(
        "inputs".into(),
        raw(call.and_then(|call| call.get("inputs")))?,
    );
    out.insert(
        "secrets".into(),
        raw(call.and_then(|call| call.get("secrets")))?,
    );
    out.insert(
        "outputs".into(),
        raw(call.and_then(|call| call.get("outputs")))?,
    );
    out.insert("permissions".into(), raw(document.get("permissions"))?);
    out.insert("concurrency".into(), raw(document.get("concurrency"))?);
    out.insert("env".into(), raw(document.get("env"))?);

    let mut jobs = Map::new();
    if let Some(Value::Mapping(mapping)) = document.get("jobs") {
        for (name, job) in mapping {
            let name = name.as_str().context("job names are strings")?;
            let mut entry = Map::new();
            if let Value::Mapping(fields) = job {
                for (key, value) in fields {
                    let key = key.as_str().context("job keys are strings")?;
                    let value = if key == "steps" {
                        steps(Some(value))?
                    } else {
                        raw(Some(value))?
                    };
                    entry.insert(key.into(), value);
                }
            }
            jobs.insert(name.into(), empty_as_list(Json::Object(entry)));
        }
    }
    out.insert("jobs".into(), empty_as_list(Json::Object(jobs)));
    Ok(Json::Object(out))
}

/// The event names `on` lists, sorted, whichever of its three spellings it uses.
fn events(on: Option<&Value>) -> Json {
    let mut names: Vec<String> = match on {
        Some(Value::String(name)) => vec![name.clone()],
        Some(Value::Sequence(names)) => names
            .iter()
            .filter_map(|name| name.as_str().map(str::to_owned))
            .collect(),
        Some(Value::Mapping(mapping)) => mapping
            .keys()
            .filter_map(|name| name.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    };
    names.sort();
    names.into()
}

fn steps(value: Option<&Value>) -> Result<Json> {
    let Some(Value::Sequence(steps)) = value else {
        return Ok(Json::Array(Vec::new()));
    };
    steps
        .iter()
        .map(|step| {
            let mut entry = Map::new();
            for key in STEP_KEYS {
                if let Some(value) = step.get(key) {
                    entry.insert(key.into(), raw(Some(value))?);
                }
            }
            entry.insert("run".into(), step.get("run").is_some().into());
            Ok(Json::Object(entry))
        })
        .collect::<Result<Vec<_>>>()
        .map(Json::Array)
}

/// A YAML value as JSON, `null` when absent, every empty mapping written as `[]`.
fn raw(value: Option<&Value>) -> Result<Json> {
    let Some(value) = value else {
        return Ok(Json::Null);
    };
    Ok(empty_as_list(serde_json::to_value(value)?))
}

fn empty_as_list(value: Json) -> Json {
    match value {
        Json::Object(map) if map.is_empty() => Json::Array(Vec::new()),
        Json::Object(map) => Json::Object(
            map.into_iter()
                .map(|(key, value)| (key, empty_as_list(value)))
                .collect(),
        ),
        Json::Array(values) => Json::Array(values.into_iter().map(empty_as_list).collect()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    fn committed() -> (std::path::PathBuf, String) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let workflows = crate::workflows::load(&root).unwrap();
        let text = inspect(&root, &workflows).unwrap();
        (root, text)
    }

    #[test]
    fn one_header_then_one_line_per_file_in_path_order() {
        let (root, text) = committed();
        let lines: Vec<Json> = text
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let count = crate::workflows::load(&root).unwrap().len();
        assert_eq!(lines[0], json!({ "schema": SCHEMA, "files": count }));
        assert_eq!(lines.len(), count + 1);
        let paths: Vec<&str> = lines[1..]
            .iter()
            .map(|line| line["path"].as_str().unwrap())
            .collect();
        let mut sorted = paths.clone();
        sorted.sort_unstable();
        assert_eq!(paths, sorted);
    }

    #[test]
    fn output_is_deterministic_and_has_no_empty_objects() {
        let (_, first) = committed();
        let (_, second) = committed();
        assert_eq!(first, second);
        assert!(!first.contains("{}"));
    }

    #[test]
    fn empty_mappings_become_lists_at_any_depth() {
        let value = json!({ "a": {}, "b": [{ "c": {} }], "d": { "e": 1 } });
        assert_eq!(
            empty_as_list(value),
            json!({ "a": [], "b": [{ "c": [] }], "d": { "e": 1 } })
        );
    }

    #[test]
    fn events_read_every_spelling_of_on() {
        let parse = |text: &str| serde_yaml_ng::from_str::<Value>(text).unwrap();
        assert_eq!(events(Some(&parse("push"))), json!(["push"]));
        assert_eq!(
            events(Some(&parse("[push, pull_request]"))),
            json!(["pull_request", "push"])
        );
        assert_eq!(
            events(Some(&parse("{workflow_call: {}, push: null}"))),
            json!(["push", "workflow_call"])
        );
        assert_eq!(events(None), json!([]));
    }

    #[test]
    fn steps_keep_their_contract_but_not_their_scripts() {
        let parse = |text: &str| serde_yaml_ng::from_str::<Value>(text).unwrap();
        let steps = steps(Some(&parse(
            "- name: Build\n  id: build\n  run: |\n    echo secret\n- uses: actions/checkout@v6\n",
        )))
        .unwrap();
        assert_eq!(
            steps,
            json!([
                { "id": "build", "name": "Build", "run": true },
                { "uses": "actions/checkout@v6", "run": false }
            ])
        );
    }
}
