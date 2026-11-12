//! `inspect`: the workflow and composite action contracts as this checkout's YAML states them,
//! and a digest of every executable helper, as JSON Lines in the `stroggforge.inspect/2` shape. It
//! is the reference the JSL forge's own `forge inspect` is compared against (FOR-101/FOR-201/
//! FOR-202), and its golden is what a migration's review diffs: a change to a trigger, a
//! permission, a step or a helper script changes a line.
//!
//! - one compact JSON object per line: a header, then one line per workflow or action ordered by
//!   path, then one line per executable file ordered by path;
//! - object keys sorted bytewise, which is what `serde_json`'s default map does;
//! - an empty mapping is written `[]`, as the first reader (Lua tables) needed;
//! - a workflow keeps its whole `on`, with every filter, schedule and dispatch input;
//! - a step keeps `id`, `name`, `uses`, `if`, `shell`, `with`, `env`, `working-directory`,
//!   `continue-on-error` and `timeout-minutes`, and its `run` script as `sha256:<hex>` of the
//!   script, or `null` when it has none.
use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde_json::{Map, Value as Json, json};
use serde_yaml_ng::Value;
use sha2::{Digest, Sha256};

use crate::workflows::Workflow;

pub const SCHEMA: &str = "stroggforge.inspect/2";

const STEP_KEYS: [&str; 10] = [
    "id",
    "name",
    "uses",
    "if",
    "shell",
    "with",
    "env",
    "working-directory",
    "continue-on-error",
    "timeout-minutes",
];

/// The folders and files whose contents run in CI or a release: the helpers the workflows call
/// and the images they build in. Each file under them is one `executable` line.
const EXECUTABLES: [&str; 4] = [
    "scripts",
    ".github/scripts",
    ".github/container-images",
    "bump-version.sh",
];

fn sha256(bytes: &[u8]) -> String {
    use std::fmt::Write;
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut hex, byte| {
            write!(hex, "{byte:02x}").expect("writing to a String cannot fail");
            hex
        })
}

/// Every file under `EXECUTABLES`, repository-relative with `/` separators, sorted bytewise.
fn executables(root: &Path) -> Result<Vec<String>> {
    fn walk(root: &Path, relative: &str, out: &mut Vec<String>) -> Result<()> {
        let path = root.join(relative);
        if path.is_file() {
            out.push(relative.to_owned());
        } else if path.is_dir() {
            for entry in fs::read_dir(&path).with_context(|| format!("read {relative}"))? {
                let name = entry?.file_name().to_string_lossy().into_owned();
                walk(root, &format!("{relative}/{name}"), out)?;
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    for relative in EXECUTABLES {
        walk(root, relative, &mut out)?;
    }
    out.sort();
    Ok(out)
}

pub fn inspect(root: &Path, workflows: &[Workflow]) -> Result<String> {
    let mut ordered: Vec<&Workflow> = workflows.iter().collect();
    ordered.sort_by(|left, right| left.path.cmp(&right.path));

    let helpers = executables(root)?;
    let mut lines = vec![serde_json::to_string(
        &json!({ "schema": SCHEMA, "files": ordered.len(), "executables": helpers.len() }),
    )?];
    for workflow in ordered {
        let text = fs::read_to_string(root.join(&workflow.path))
            .with_context(|| format!("read {}", workflow.path))?;
        let document: Value =
            serde_yaml_ng::from_str(&text).with_context(|| format!("parse {}", workflow.path))?;
        lines.push(serde_json::to_string(&file(workflow, &document)?)?);
    }
    for path in helpers {
        let bytes = fs::read(root.join(&path)).with_context(|| format!("read {path}"))?;
        lines.push(serde_json::to_string(
            &json!({ "kind": "executable", "path": path, "sha256": sha256(&bytes) }),
        )?);
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
    out.insert("on".into(), raw(on)?);
    out.insert("defaults".into(), raw(document.get("defaults"))?);
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
            entry.insert("run".into(), script(step.get("run"))?);
            Ok(Json::Object(entry))
        })
        .collect::<Result<Vec<_>>>()
        .map(Json::Array)
}

/// A step's `run` as `sha256:<hex>` of the script, `null` when it has none, or the value itself
/// when it isn't text (which GitHub would refuse).
fn script(value: Option<&Value>) -> Result<Json> {
    match value {
        Some(Value::String(text)) => Ok(format!("sha256:{}", sha256(text.as_bytes())).into()),
        other => raw(other),
    }
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
        let helpers = executables(&root).unwrap().len();
        assert_eq!(
            lines[0],
            json!({ "schema": SCHEMA, "files": count, "executables": helpers })
        );
        assert_eq!(lines.len(), count + helpers + 1);
        let paths: Vec<&str> = lines[1..=count]
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
    fn steps_keep_their_contract_and_a_digest_of_their_scripts() {
        let parse = |text: &str| serde_yaml_ng::from_str::<Value>(text).unwrap();
        let steps = steps(Some(&parse(
            "- name: Build\n  id: build\n  continue-on-error: true\n  run: |\n    echo secret\n- uses: actions/checkout@v6\n",
        )))
        .unwrap();
        assert_eq!(
            steps,
            json!([
                {
                    "continue-on-error": true,
                    "id": "build",
                    "name": "Build",
                    "run": format!("sha256:{}", sha256(b"echo secret\n")),
                },
                { "uses": "actions/checkout@v6", "run": null }
            ])
        );
        assert!(!steps.to_string().contains("secret\\n"));
    }

    #[test]
    fn a_digest_is_sha256_in_lowercase_hex() {
        assert_eq!(
            sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
