# The forge

`org.dreamweave.forge`: StroggForge's organization logic as a Jess package. This is the first
slice ([FOR-201](https://github.com/DreamWeave-MP/StroggForge/issues/38)): two commands that read
a checkout and decide something about it. The plan for the rest is
[`docs/vnext/FOR-102-rfc.md`](../docs/vnext/FOR-102-rfc.md).

Nothing in CI runs this yet. The Rust war room (`tools/war-room`) is still what `war-room.yml`
runs, and stays the reference until FOR-202 replaces it.

## Commands

From the checkout's root, with a jess whose `@jess/yaml` has `decodeWithLayout` on `PATH`
([jess#30](https://github.com/DreamWeave-MP/jess/pull/30); no released jess has it yet, and on
one that doesn't the forge stops with that message, exit 1):

```sh
jess workspace run --trusted forge-check                  # validate war-room/ against .github/
jess workspace run --trusted forge-check -- --format json # stroggforge.diagnostics/1 on stdout
jess workspace run --trusted forge-inspect                # stroggforge.inspect/2 JSON Lines
```

Both take an optional root as their first argument and default to the working folder. `--trusted`
grants what the targets declare, which is `filesystem.read` and `process.stdio`: they read the
checkout, write to the terminal, and touch nothing else. No network, no environment, no processes.

| Exit | Means |
| --- | --- |
| 0 | valid |
| 1 | the forge crashed: a bug, not a verdict |
| 64 | the forge's own arguments are wrong |
| 65 | findings: the report says what and where |
| 66 | an input is missing or unreadable |

Diagnostic codes (`FORGE-E0xx`) are listed with their meanings in `src/diagnostics.jsl`. The code is
the contract; the message is for people and may change.

## What `check` checks

What the Rust war room's `validate.rs` checked, rule for rule: closed records (no unknown fields,
no missing ones, enums from their closed sets, strings that are strings and not TOML datetimes,
integers that weren't written as floats), unique IDs, references that resolve to projects, plans,
platforms, workflows, jobs and composite steps, plan states that agree with their requirements and
prerequisites, and no cycles. It reports every violation, where the Rust stopped at the first.

It also holds the workflow YAML to the shapes serde made the Rust insist on: `jobs` a mapping of
named jobs, each job a mapping, `steps` a list, `needs` a name or a list of names, and an
`inputs`, `secrets` or `outputs` contract a mapping (`FORGE-E056`). A folder named like a workflow
is an unreadable workflow, as it was. `@jess/yaml`'s layout says whether the file wrote a mapping
or a sequence, so `jobs: []`, `steps: {}` and `needs: {1: test}` are refused as serde refused them,
and a tag is read through everywhere serde read through it, and nowhere else (`needs`).

It reads YAML within serde_yaml_ng's limits: 128 nested collections, and alias jumps over the
expanded document of at most 100 per parser event (a billion-laughs file is refused, as the Rust
refused it). An alias to a scalar decodes to a plain value, so it counts as an event but not a jump;
that can move the repetition threshold, never past a document the Rust refused by a wide margin.

Its first finding is the error the Rust stopped on: TOML tables are walked in sorted key order, as
toml 0.9 visited them, with missing fields after; the files in the order the Rust read them; actions
in Rust's path order (`a/` before `a-b/`); a workflow's jobs, needs, name and contract in its
loader's order. `tests/differential.jsl` holds it to that on every pair of corpus faults.

Where it decides differently, it says so: `tests/fixtures/cases.toml` with a `divergence`, and
`tests/fixtures/differential.toml` for the differential's trees.

- A date is exactly `YYYY-MM-DD` in digits. Rust checked the length and parsed each part as an
  integer, so it also took `2026-+1-03` and `2024-2-029`.
- YAML scalars resolve by YAML 1.2's core schema, where serde_yaml_ng kept YAML 1.1 habits: `0b101`
  and `-0x1` are strings, `007` is 7, `!t 12` holds the string `12`, `!!null ""` is null. Where such a
  value is a name, a job key or a needs entry, the verdict changes with it.
- An integer from 2^63 to 2^64-1, or `.nan` as a key, can't be held in Luau and is refused.
- A byte order mark at the start of a file is allowed, as YAML 1.2 allows it.
- Four `@jess/yaml` defects the differential found are fixed in
  [jess#31](https://github.com/DreamWeave-MP/jess/pull/31) and recorded until the forge runs on it.

`inspect` (`stroggforge.inspect/2`) is the golden a migration's review diffs. Beside each
workflow's contract it keeps the whole `on` (every branch, path and tag filter, schedule and
dispatch input), `defaults`, every job's fields including `permissions`, each step's
`continue-on-error` and `timeout-minutes`, and `sha256:<hex>` of each step's `run` script; then one
line per file under `scripts/`, `.github/scripts/`, `.github/container-images/` and
`bump-version.sh` with its SHA-256, and likewise every file under `.github/actions/` and
`.github/action_templates/`, `package.json` and `package-lock.json`; and `run-name`, and an action's
whole `runs` besides its steps. A changed trigger, permission, step or helper is a changed line.
File modes aren't recorded: `@dream/fs` doesn't expose them, and git shows a mode change in review.

`inspect` writes JSON as serde_json wrote it: floats as floats (`1.0`, `1e+300`), integers past 2^53
exactly, a tag `!t` as `{"!t": value}`. Two differences remain, both pinned by the suite: `010` is
the integer 10, as YAML 1.2 reads it, where serde_yaml read a string; and a key written `1.0` is
`"1"`, which the layout can't record. A key that is null or a collection is `FORGE-E057`, where
the Rust stopped with an error. Unlike the Rust, `inspect` reads only `.github`, so it works on a
repository with no war room.

## Layout

```text
src/diagnostics.jsl   codes, exit statuses, text and JSON reports
src/schema.jsl        closed TOML records
src/model.jsl         the war-room files as records
src/workflows.jsl     the workflow and action YAML
src/inspect.jsl       the stroggforge.inspect/2 contract
src/check.jsl         the structural rules
src/commands.jsl      typed requests and results: what another tool calls
src/cli/              argv, printing and exit statuses
tests/                the suite, the fixture checkout, the negative corpus and the Rust oracle's verdicts
```

## Tests

```sh
jess package check forge   # strict types, every .jsl
jess package test forge    # the suite, from source and packed; @jess/style over every source
```

Run both from the checkout's root: the suite compares this checkout's workflows with
`docs/vnext/golden/workflow-contracts.jsonl`, which the Rust war room wrote.

`tests/fixtures/cases.toml` is the negative corpus, one broken rule per case. Each case must fail
with its code, and agree with the Rust war room's verdict recorded in `tests/fixtures/oracle.json`
unless it names a `divergence`. After changing the corpus or the Rust validator, re-record:

```sh
cargo build --locked -p stroggforge-war-room
jess run --trusted forge/tests/oracle.jsl target/debug/stroggforge-war-room
```

The golden comparison is byte for byte: any jess with `decodeWithLayout` also has the block-scalar
chomping fix (jess#28) that jess 0.3.0 lacks.
