# FOR-202 receipt: the forge's validator, held to the Rust

[FOR-202](https://github.com/DreamWeave-MP/StroggForge/issues/39): the JSL `forge check` and
`forge inspect` complete, a negative and differential corpus, deterministic order and exit behavior
compared with the Rust war room, and a plan for retiring the Rust validator.

## DOC-101: a provisional receipt, not a waiver

DOC-101 (#25) is to define what a five-stage receipt is. It is blocked by SEC-101 (#21) and not
ratified. FOR-201 depends on DOC-101, and FOR-202 on FOR-201, so this receipt cannot satisfy that
dependency. It uses the provisional format FOR-102 describes ("Coordination with EP-DOC"): source
SHA, exact command, observed result, and what was not run. The integration contract proposed to
DOC-101 is the comment on #25.

**The DOC-101 dependency is open, and stays open.** Neither FOR-201 nor FOR-202 should merge on the
strength of this format alone. When DOC-101 is ratified, this receipt and FOR-201's are either
re-issued in its format or accepted by an explicit DOC-101 decision. Merging before that is a human
call that has to be recorded as an exception.

## Inspect

- **Source:** branch `vnext/for-202-check`, stacked on `vnext/for-201-forge` (FOR-201, #66), which
  is stacked on FOR-102 (#65) and FOR-101 (#64), all current with `main` at `72c3050`. FOR-101 and
  the FOR-102 RFC were ratified by the maintainer on 2026-11-12; none of the four is merged.
- **jess:** the forge needs `@jess/yaml`'s `decodeWithLayout`, added in
  [jess#30](https://github.com/DreamWeave-MP/jess/pull/30), merged as `33d0d62`. Every command
  below ran on jess's `dev` release built from `main` at `1a4be2f`. That is `jess-Linux-X64.zip`,
  SHA-256 `9a0877f1…69ff`; the binary inside is `6ab640e2…c57b`. Its Sigstore bundle verified with
  `cosign verify-blob`: signed by `StroggForge/.github/workflows/rustGlobalBuild.yml@refs/tags/v70`
  for `DreamWeave-MP/jess` at `1a4be2f`, `refs/heads/main`, on a GitHub-hosted runner. The same
  gates passed earlier on the branch build `af28987`.
- **Hypothesis:** the forge can replace the Rust validator without changing any verdict, any first
  finding or any exit class, apart from divergences each recorded with a reason. Where a divergence
  comes from a jess defect, the defect gets fixed in jess, not worked around.
- **Threat boundary:** unchanged from FOR-201. Both commands declare `filesystem.read` and
  `process.stdio` only. The differential adds `process.spawn` to run the Rust oracle and jess; it is
  a test script and isn't shipped.

## Implement

Commits on `vnext/for-202-check` after the FOR-201 merge (`0921f8e`):

| Change | Commits |
| --- | --- |
| Read YAML through the layout: the nine `{}`/`[]` divergences become agreements; tags are read through where serde read through them | `9d47617`, `993f79f`, `f9f490c` |
| First finding equals the Rust's error: TOML keys sorted as toml 0.9 visits them, Rust's file and loader order | `694d5b7` |
| Differential harness | `bb75e79`, `dcd45ab` |
| `stroggforge.inspect/2`, in the Rust (with `sha2`) and the forge, and both goldens regenerated | `b57ce65`, `f2cafa1`, `6510a91`, `0c46ff8`, `592bf2a`, `31b237d` |
| Review fixes: duplicate ids, PathBuf order, serde_yaml_ng's depth and repetition limits | `27291a2` |
| Docs | `dd19ea3`, and this receipt |

Upstream in jess:

- [jess#30](https://github.com/DreamWeave-MP/jess/pull/30), merged as `33d0d62`: the layout side
  channel, plus fixes for the silent mis-decodes it exposed. These covered `--- [1]`, own-line tags,
  compact-mapping nesting, quotes inside plain scalars, `: ` in plain scalars, control characters,
  and integers that were clamped or wrapped.
- [jess#31](https://github.com/DreamWeave-MP/jess/pull/31), open: implicit flow entries, `%YAML 1.1`,
  empty tag suffixes, and compact block-scalar indentation.

Nothing under `.github`, `scripts`, `site` or `war-room` changed:
`git diff --stat 0921f8e..HEAD -- .github scripts bump-version.sh site war-room` is empty. No workflow, release, signing or consumer path was touched.

## Strict

From the checkout's root at `2ec5e9d`; `J` is the `dev` jess above.

| Command | Result |
| --- | --- |
| `$J package check forge` | `errors: []` |
| `$J package test forge` | 246 passed, 0 failed (source and packed) |
| `cargo fmt --all --check`; `cargo clippy --locked --workspace --all-targets --all-features -- -W clippy::pedantic -D warnings`; `cargo test --locked --workspace --all-features` | clean; clean; 14 passed |
| `actionlint` | clean |
| `target/debug/stroggforge-war-room inspect .` vs `docs/vnext/golden/workflow-contracts.jsonl`, and `forge-inspect` vs the same | both byte-identical: 70 lines, 16 workflows and actions, 53 executable files |
| `$J run --trusted forge/tests/differential.jsl target/debug/stroggforge-war-room $J` | exit 0 in 37 s; table below |
| jess on a release without `decodeWithLayout` | `the forge needs a jess whose @jess/yaml has decodeWithLayout`, exit 1 |

The differential, every tree built from `forge/tests/fixtures/repo`:

| Set | What it compares | Agree | Explained | Disagree | Skipped |
| --- | --- | --- | --- | --- | --- |
| single | 97 corpus trees: verdict and exit class (0 / 65 / 66 against the Rust's 0 / 1 and whether its error was the OS's) | 97 | 0 | 0 | 0 |
| pair | every two faults both sides refuse alone: the forge's first finding is the fault the Rust stopped on | 2488 | 0 | 0 | 68, edits that can't both be made |
| mutation | each fixture file with a line deleted, duplicated, re-indented or tabbed, cut short, or given a BOM, CRLF, NUL or non-UTF-8 byte | 900 | 3 | 0 | 0 |
| inspect | both `inspect`s byte for byte, on every tree above the Rust accepts and 19 crafted YAML files | 347 | 8 | 0 | 0 |
| cli | the forge's command line against the forge in process | 97 | 0 | 0 | 0 |

Explained, each with its reason in `forge/tests/fixtures/differential.toml` or `cases.toml`:

- **Spec divergences:**
  - a BOM at the start of a file is allowed (3 mutations);
  - YAML 1.2 core-schema scalars, where serde kept YAML 1.1 habits: `0b101`, `-0x1`, `007`, `!t 12`, `!!null ""`;
  - integers from 2^63 to 2^64-1, which Luau can't hold.
- **jess#31 defects,** to be removed when the forge runs on a jess with #31: `{a, b}`, `%YAML 1.1`, `!! x`, and `- run: |2`.
- **`signed-date`:** the forge's stricter date check.

## Review

Two independent adversarial reviews; findings and dispositions:

1. **jess#30, the layout side channel.** The reviewer used about 90 hand cases, a 4000-document
   generator, and a comparison of plain `decode` with `main`. Fixed in `c7da82a`:
   - own-line tags resolved the scalar before the tag;
   - stale layout entries;
   - the compact-mapping nesting misparse;
   - `- &x a: 1` read as a string;
   - clamped integers;
   - wrong docs claims.

   Left alone and listed in the PR: `- ? a`, anchor redefinition, flow continuations.

   The rerun found 0 layout failures in 2318 parsed documents.
2. **The forge (FOR-202).**
   - Fixed in `27291a2`, `31b237d` and `dcd45ab`:
     - two workflows sharing an id flipped the verdict;
     - actions loaded in string rather than PathBuf order;
     - billion laughs and depth went unbounded;
     - the golden missed `run-name`, an action's `runs`, files beside actions, consumer templates and the Node manifests;
     - the differential skipped pairs whose first finding matched neither fault, filtered pairs with a vacuous test, and never compared `inspect`.
   - Upstreamed in jess#31: four YAML defects.
   - Recorded as divergences: the YAML 1.1 scalar habits, unsigned 64-bit integers and NaN keys.
   - Not fixed: file modes, because `@dream/fs` has no mode bits (stated in the README), and non-UTF-8 helper file names, where the Rust skips the file and the forge lists it.

Mutation checks:
- In FOR-201, seven of eight planted defects failed the suite.
- Dropping `on` from inspect fails six tests, among them the new `trigger` golden check, whose event names don't change, so only `on` can catch it.

Not run:
- any workflow, consumer or release path;
- Windows and macOS: the forge has no platform calls, but that comes from reading the code, not running it;
- the differential with jess#31 applied.

## Style

`@jess/style` runs over every `.jsl` in the package as a suite case (part of the 246), so the style
gate is in `jess package test`. Rust: `cargo fmt` and pedantic Clippy, as above. The goldens are
deterministic: two runs are byte-identical, and the Rust regenerates the committed file exactly.

## Profile

Linux x86-64, this workstation, the `dev` jess above (an LTO release build) and the Rust war room
built `--release`. Five runs each, wall time from fastest to slowest, peak RSS:

| | Wall | Peak RSS |
| --- | --- | --- |
| `forge-check` (`jess workspace run`) | 0.11-0.12 s | ~30 MB |
| `forge-inspect` | 0.12-0.15 s | ~31 MB |
| `jess package test forge` (246 cases, source and packed) | 0.82-0.85 s | ~39 MB |
| Rust `check`, release | under 0.01 s | ~4 MB |
| Rust `inspect`, release | 0.01 s | ~4 MB |
| Rust release rebuild after touching `main.rs` (toolchain already installed) | 4.5 s | |

The forge is about ten times slower per run, and the difference is all jess's startup. What CI
would trade is the toolchain, not the run. `war-room.yml` installs Rust (`dtolnay/rust-toolchain`,
then a crate build: about a minute cold). The forge needs one 13 MB download, `jess-Linux-X64.zip`
(13,058,301 bytes). That saving is an estimate until a CI job measures it, and it can't be measured
before SEC-201's authenticated bootstrap exists. It is not claimed here.

## Retirement

[`FOR-202-rust-retirement.md`](FOR-202-rust-retirement.md). Parity on the corpus is demonstrated.
CI cutover waits on gates nobody here can open:

- a jess release with #30, which takes human offline signing;
- the authenticated jess bootstrap (SEC-201);
- separate authorization to change `war-room.yml`.
