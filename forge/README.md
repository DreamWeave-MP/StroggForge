# The forge

`org.dreamweave.forge`: StroggForge's organization logic as a Jess package. This is the first
slice ([FOR-201](https://github.com/DreamWeave-MP/StroggForge/issues/38)): two commands that read
a checkout and decide something about it. The plan for the rest is
[`docs/vnext/FOR-102-rfc.md`](../docs/vnext/FOR-102-rfc.md).

Nothing in CI runs this yet. The Rust war room (`tools/war-room`) is still what `war-room.yml`
runs, and stays the reference until FOR-202 replaces it.

## Commands

From the checkout's root, with jess 0.3.0 or newer on `PATH`:

```sh
jess workspace run --trusted forge-check                  # validate war-room/ against .github/
jess workspace run --trusted forge-check -- --format json # stroggforge.diagnostics/1 on stdout
jess workspace run --trusted forge-inspect                # stroggforge.inspect/1 JSON Lines
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

One rule differs on purpose: a date is four digits, two and two. Rust's integer parsing also took
`2026-+1-03`.

## Layout

```text
src/diagnostics.jsl   codes, exit statuses, text and JSON reports
src/schema.jsl        closed TOML records
src/model.jsl         the war-room files as records
src/workflows.jsl     the workflow and action YAML
src/inspect.jsl       the stroggforge.inspect/1 contract
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

jess 0.3.0's YAML loader drops the last line break of a block scalar that ends a document. The
suite tolerates exactly that difference on the one golden line it affects, and expects byte
equality on a jess without the defect.
