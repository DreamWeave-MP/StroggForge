mod graphs;
mod history;
mod pages;
mod pipelines;

use std::{fs, path::Path};

use anyhow::{Context, Result, ensure};

use crate::model::{Model, Project};

pub fn build(root: &Path, model: &Model) -> Result<()> {
    let site = root.join("site");
    for directory in ["content", "static/generated"] {
        let directory = site.join(directory);
        if directory.exists() {
            fs::remove_dir_all(&directory)?;
        }
        fs::create_dir_all(directory)?;
    }
    copy_pages(&site.join("pages"), &site.join("content"))?;
    copy_pages(&site.join("maps"), &site.join("static/generated/maps"))?;
    write(
        &site,
        "static/generated/LICENSE-AGPL-3.0.txt",
        &model.sources["mod-template"].evidence["LICENSE"],
    )?;
    write(
        &site,
        "static/generated/model.json",
        &format!("{}\n", serde_json::to_string_pretty(model)?),
    )?;
    pages::build(&site, model)?;
    graphs::build(&site, model)?;
    history::build(&site, model)?;
    pipelines::build(&site, root, model)?;
    forbidden_terms(root, &site, model)
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Terminology {
    forbidden: Vec<String>,
}

/// Fail if a forbidden name or an excluded repository reached any generated file.
fn forbidden_terms(root: &Path, site: &Path, model: &Model) -> Result<()> {
    let terminology: Terminology = crate::read_toml(&root.join("war-room/terminology.toml"))?;
    let terms: Vec<&String> = terminology
        .forbidden
        .iter()
        .chain(&model.census_exclusions)
        .collect();
    let mut pending = vec![site.join("content"), site.join("static/generated")];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            for term in &terms {
                ensure!(
                    !contains_word(&text, term),
                    "{} contains the forbidden name {term:?}; fix the canonical record it came from (war-room/terminology.toml, war-room/census-exclusions.toml)",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

fn copy_pages(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    let mut entries = fs::read_dir(source)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort();
    for entry in entries {
        let destination = target.join(entry.file_name().unwrap());
        if entry.is_dir() {
            copy_pages(&entry, &destination)?;
        } else {
            fs::copy(&entry, &destination).with_context(|| format!("copy {}", entry.display()))?;
        }
    }
    Ok(())
}

fn write(site: &Path, path: &str, text: &str) -> Result<()> {
    let path = site.join(path);
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, text).with_context(|| format!("write {}", path.display()))
}

fn page(site: &Path, path: &str, title: &str, description: &str, body: &str) -> Result<()> {
    let frontmatter = format!(
        "+++\ntitle = {}\ndescription = {}\nweight = 10\n+++\n\n",
        serde_json::to_string(title)?,
        serde_json::to_string(description)?
    );
    write(
        site,
        &format!("content/{path}.md"),
        &format!("{frontmatter}{body}"),
    )
}

fn section(site: &Path, path: &str, title: &str, weight: u32, body: &str) -> Result<()> {
    write(
        site,
        &format!("content/{path}/_index.md"),
        &format!(
            "+++\ntitle = {}\nsort_by = \"title\"\nweight = {weight}\ntemplate = \"docs/section.html\"\npage_template = \"docs/page.html\"\n+++\n\n{body}",
            serde_json::to_string(title)?
        ),
    )
}

fn cell(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "&#124;")
        .replace(['\n', '\r'], " ")
}

fn project_link(project: &Project) -> String {
    let lane = if project.lifecycle.historical() {
        "archaeology/projects"
    } else {
        "ecosystem/projects"
    };
    format!("[{}](@/{lane}/{}.md)", cell(&project.name), project.id)
}

fn diagram(id: &str) -> String {
    format!("\n{{{{ diagram(name=\"{id}\") }}}}\n\n")
}

/// Whole-word match, so a forbidden name inside a longer legitimate one (TES3MP) is allowed.
/// A word boundary is only required where the name itself starts or ends alphanumerically.
fn contains_word(text: &str, term: &str) -> bool {
    let bounded_start = term.chars().next().is_some_and(char::is_alphanumeric);
    let bounded_end = term.chars().next_back().is_some_and(char::is_alphanumeric);
    text.match_indices(term).any(|(start, _)| {
        let before = text[..start].chars().next_back();
        let after = text[start + term.len()..].chars().next();
        let joined_before = bounded_start && before.is_some_and(char::is_alphanumeric);
        let joined_after = bounded_end && after.is_some_and(char::is_alphanumeric);
        !joined_before && !joined_after
    })
}

#[cfg(test)]
mod tests {
    use super::contains_word;

    #[test]
    fn forbidden_names_match_whole_words_only() {
        assert!(contains_word("core functionality in S3MP.", "S3MP"));
        assert!(!contains_word("derived from TES3MP", "S3MP"));
        assert!(contains_word("(x.brand)", ".brand"));
        assert!(!contains_word("docs-brand", ".brand"));
        assert!(!contains_word("x.branding", ".brand"));
    }
}
