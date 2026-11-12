# FOR-201 receipt: the forge's first slice

[FOR-201](https://github.com/DreamWeave-MP/StroggForge/issues/38): a JSL workspace, `forge
inspect` and `forge check`, golden legacy fixtures, no CI change. Receipt format is provisional
until DOC-101 ratifies one (see FOR-102, "Coordination").

## Inspect

- **Source:** branch `vnext/for-201-forge`, stacked on FOR-101 and FOR-102; `main` at `72c3050`.
- **Hypothesis:** the war room's validation and the workflow contracts can be owned by a Jess
  package on shipped jess 0.3.0 with no Rust, no network and no new dependency, and agree with the
  Rust war room on every case the Rust decides.
- **Threat boundary:** both commands read a checkout and write to the terminal. They declare
  `filesystem.read` and `process.stdio` and nothing else; `jess workspace run` without `--trusted`
  refuses them. They are U-class (FOR-102): safe on pull-request content.

## Implement

`forge/` (package `org.dreamweave.forge` 0.1.0, no dependencies) and `jess-workspace.toml`.
Library, `commands` and `cli` layers per the jess tool-package rule. Nothing under `.github`,
`scripts`, `site` or `war-room` changed (`git diff --stat main..HEAD -- .github scripts
bump-version.sh site war-room` is empty).

## Strict

| Command (from the checkout root) | Result |
| --- | --- |
| `jess package check forge` | `errors: []`, 13 files |
| `jess package test forge` (jess 0.3.0) | 200 passed, 0 failed (100 cases, source and packed) |
| `jess package test forge` (jess with the local `@jess/yaml` fix, `a07000c`) | 200 passed; the golden comparison takes its byte-exact path |
| `jess workspace run --trusted forge-check` | `Validated 22 projects, 2 plans and 16 workflows.`, exit 0 |
| `jess workspace run --trusted forge-inspect` vs `docs/vnext/golden/workflow-contracts.jsonl` | identical except the one documented `@jess/yaml` byte; identical on the fixed jess |
| broken tree (bad date + missing job) | both findings reported, exit 65 |
| missing root | `FORGE-E001`, exit 66 |
| `forge-check a b`, `--format xml` | usage, exit 64 |
| `jess workspace run forge-check` without a grant | refused, `filesystem.read` denied |
| `actionlint` | clean |

Corpus: 58 refusing cases and 19 accepting ones, every one of the 30 diagnostic codes exercised.
Each refusing case names part of the reason the Rust gave (`rust`), and the suite checks the
recorded Rust message contains it, so no case passes on a refusal for some other reason. The Rust
refuses 57 of the 58 and accepts 10 of the 19; the ten disagreements are recorded `divergence`s:
`signed-date` (the forge's stricter date) and nine trees @jess/yaml decodes to Lua tables that
can't say whether the file wrote `{}` or `[]`, or a mapping keyed 1 to n or a list.

Adversarial review: an independent reviewer ran about 200 edited trees through both
implementations and found the first version accepted shapes the Rust refused (an empty TOML `{}`
for a list, `jobs` as a list or string, steps as a mapping, contracts that weren't mappings), refused
a reusable action the Rust accepted, and crashed `inspect` on an integer past 2^53, an infinity, or
a non-string key. All of those are fixed and in the corpus or the suite; inspect now writes
infinities as null as serde_json did and reports what JSON can't hold as `FORGE-E057`.

The reviewer's second pass confirmed those fixes and found what the first fix broke: any key that
wasn't a string made the document, `on` or `runs` unreadable, where the Rust's `get` ignored such
keys; `inspect` still read steps given as a mapping that `check` no longer did; seven more trees of
the `{}`/`[]` family were unrecorded; and a large integer key was reported for the wrong reason.
All fixed: the forge reads those containers as `get` did (three accept cases), `inspect` and
`check` agree on steps, the seven trees are recorded divergences with the Rust's reasons, and
`inspect`'s remaining differences from serde are pinned by a test. The reviewer also judged the
golden tolerance and the fixed-order test non-vacuous (it broke the job walk's order in a copy and
the test failed).


Mutation check, after the review fixes: eight deliberate defects, each applied alone and then
restored. Seven failed the suite: dropping the blocked-plan rule (4 failing cases), unsorted events
(4), accepting unknown fields (2), treating every workflow as reusable (6), reading TOML tables as
lists (2), not checking the shape of `jobs` (2), letting infinities reach the JSON writer (2). One
survived: walking `[workflow.*]` notes unsorted changes only the order of several notes findings,
which no test produces yet.

Not run: any workflow, any consumer, any release path; Windows and macOS (the commands use
`@dream/fs.join` and no platform calls, but that is a reading, not a run).

## Style

`@jess/style` runs over every `.jsl` in the package as a test case, so the style gate is part of
`jess package test`. It caught single-letter names in `isDate` on the first run.

## Profile

Linux, this workstation, shipped jess 0.3.0, after the review fixes; 3-5 runs each:

| | Wall | Peak RSS |
| --- | --- | --- |
| `forge-check` (jess workspace run) | 0.10-0.11 s | ~30 MB |
| `forge-inspect` | 0.10-0.11 s | ~31 MB |
| Rust `check` (debug build, already compiled) | 0.03 s | ~6 MB |
| `jess package test forge` (200 cases) | 0.51-0.52 s | ~39 MB |

The forge is slower per run and needs no compile. Whether that removes the Rust toolchain step
from `war-room.yml` and what it saves is measured in FOR-202, when CI runs it; it is not claimed
here.
