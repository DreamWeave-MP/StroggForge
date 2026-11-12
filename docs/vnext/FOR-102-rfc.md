# FOR-102: the forge as a Jess application

Status: **proposed RFC** for [FOR-102](https://github.com/DreamWeave-MP/StroggForge/issues/18).
It authorizes nothing. Decisions marked **open** need a human.

Builds on the [FOR-101 audit](FOR-101-audit.md); its inventory and goldens are the evidence base.
Every API named here was checked against jess 0.3.0's generated types (`jess luau-types`) and run
where a receipt says so.

## What this RFC decides

StroggForge's organization logic moves into one Jess package, `org.dreamweave.forge`, run as
command targets. GitHub Actions keeps what only it can do: events, runners, job permissions,
environments and secrets. Native tools stay where they are the trust or the toolchain: rustc,
Cargo, clang/lld, Git, Docker, Cosign, `gh`. The forge calls them; it does not become a script that
assembles shell lines for them.

The test for "is this a wrapper": if a forge command's logic is "build an argv, run it, print what
came back", it is a wrapper and stays a YAML step. A forge command owns a decision: what to build,
which files make a release, whether a release may proceed, what the manifest says.

## Package and modules

One package, in `forge/`, its own Package Source; the repository root becomes a jess workspace
whose one member is `forge` (the Rust war room keeps the root `Cargo.toml` until FOR-202 retires
it).

```text
forge/
  jess.toml                package org.dreamweave.forge, jess_api 1, no dependencies at first
  src/
    diagnostics.jsl        Diagnostic, codes, severities, exit statuses, JSON rendering
    schema.jsl             strict TOML records: closed field sets, enums, dates, IDs
    model.jsl              war-room/*.toml loaded through schema into a typed model
    workflows.jsl          .github workflows and actions: contracts read from YAML
    inspect.jsl            the stroggforge.inspect/1 JSON Lines contract
    check.jsl              structural validation (today's validate.rs)
    commands.jsl           typed requests in, typed results out; no argv, no printing
    cli/<command>.jsl      argv, output, exit status: one per target
  tests/
    run.jsl                the @jess/test suite
    fixtures/              valid and broken trees, oracle results
```

Later modules, each landing with its own fixture parity: `site/` (the war room's pages, FOR-203),
`release/` (archive, checksums, manifest, changelog, muxapp, docs index, crate order),
`nexus/`, `benchmarks/`, `verify/` (binary portability, digests, provenance). The three layers
(library, `commands`, `cli`) follow the jess tool-package rule, so a later caller (an editor, an
agent, a test) uses `commands` and never parses CLI text.

## Commands

| Command | Does | Capabilities | Network | Privilege class |
| --- | --- | --- | --- | --- |
| `forge inspect` | prints workflow/action contracts (`stroggforge.inspect/1`) | `filesystem.read` | none | U |
| `forge check` | validates war-room data against the workflows | `filesystem.read` | none | U |
| `forge build` | writes the war room site (FOR-203) | `filesystem.read`, `filesystem.write` | none | U |
| `forge verify` | checks staged release files: names, digests, binary portability, provenance bundle shape | `filesystem.read`, `process.spawn` (otool/readelf until JSL reads load commands) | none | V |
| `forge release plan` | freezes a release manifest from verified staged files | `filesystem.read`, `filesystem.write` | none | V |
| `forge release apply` | performs a frozen manifest through `gh`/`cargo publish`, refusing anything not in it | `filesystem.read`, `process.spawn`, `process.environment` | via native tools | P |
| `forge doctor` | reports toolchains, versions, capability grants, bootstrap digests | `filesystem.read`, `process.spawn`, `process.environment` | none | U |
| `forge jess-release` | operator-local: prepares and checks an official Jess release; never signs | see below | none | Operator |

Classes: **U** untrusted (may run on pull-request code; no secrets, `contents: read`), **V**
validation (runs on build outputs, never on source; no secrets), **P** promotion (runs only on a
frozen manifest from V, in a job with an `environment:` gate and the narrowest write permission).
Signing is not a forge class: Sigstore keyless signing stays a Cosign step, and official Jess
signing stays offline with a human.

`inspect` and `check` are FOR-201. The rest are named so their contracts exist before their code.

## Contracts

### Output

- Diagnostics: one JSON document on stdout with `--format json`, text on stderr otherwise.
  `{"schema": "stroggforge.diagnostics/1", "command", "ok", "diagnostics": [{"code", "severity",
  "path", "where", "message"}], "summary"}`. `code` is stable (`FORGE-E0xx`), `message` is not:
  tests match codes, people read messages.
- `inspect`: JSON Lines, a header then one line per file in bytewise path order, keys sorted, an
  empty mapping written `[]` (a decoded Lua table cannot say which it was). The Rust oracle writes
  the same bytes; the golden is `docs/vnext/golden/workflow-contracts.jsonl`.
- Determinism: no timestamps, no absolute paths, no hash-table order. Two runs over one tree are
  byte-identical.

### Exit status

Jess gives a command target's first returned number as the exit status, prints an uncaught error
as `error: …` with status 1, and uses 2 for a command line it can't parse. The forge stays out of
both:

| Status | Meaning |
| --- | --- |
| 0 | done, nothing wrong |
| 1 | the forge crashed (an uncaught error): a bug, never a verdict |
| 64 | usage: the forge's own arguments are wrong |
| 65 | the input is invalid: findings were reported |
| 66 | an input is missing or unreadable |
| 74 | an output could not be written |
| 77 | refused: a gate or authority check said no |

A CI step that sees 65 knows the data is wrong; one that sees 1 knows the tool is. Today both are
`anyhow`'s 1.

### Time

Offline commands are bounded by their input. A forge command that starts a native tool needs a
deadline, and `@dream/process.run` in jess 0.3.0 has none. Until a jess release carries l3i 2.8's
bounded runs, every job calling such a command sets `timeout-minutes`, and the forge does not ship
`verify`'s native half or `release apply` before that release (**constraint N1**).

## Getting jess onto a runner

Today `jess-setup` downloads a release zip from GitHub over TLS and runs it, with no digest check.
The forge then runs on whatever that URL served. Options:

| | How | Trust root | Offline | Cost |
| --- | --- | --- | --- | --- |
| A | per-platform SHA-256 of each jess release asset, pinned in StroggForge; the bootstrap step checks with `sha256sum`/`shasum`/`Get-FileHash` **before** running jess | the StroggForge commit | after first fetch, from cache keyed by digest | one table per jess release |
| B | the jess image, already pinned by digest in `modGlobalBuild` | the StroggForge commit + GHCR | yes, once pulled | Linux only; ~seconds to pull |
| C | `cargo install jess --locked --version X` | crates.io + Cargo.lock | no | minutes of compile per job |
| D | Sigstore: `cosign verify-blob` of the release asset against the jess release workflow's identity | Fulcio/Rekor + pinned identity | no | needs the jess release to publish bundles |
| E | official Jess offline signature checked against a pinned public key | the operator's key | yes | needs a verifier that is not the jess being verified |

**Recommendation: A, with D added once jess publishes Sigstore bundles.** A never asks a
downloaded jess whether it is genuine; the check runs in the runner's own shell before jess
executes. D adds who-built-it to what-it-is. B stays for container jobs. E is SEC's design to own:
a verification key can be pinned in StroggForge, but the verifier must be native or already
trusted, since jess 0.3.0 can sign with RSA/ECDSA and cannot verify anything (**constraint N2**).

The forge package itself is not downloaded: a job runs it from the StroggForge checkout at the
pinned tag, so the package's bytes are the tag's bytes. A P-class job checks the package digest
(`jess package inspect`) against one recorded in the tag before running it.

## Privilege separation

FOR-101 found the release jobs compile consumer code (build scripts, proc macros, every
dependency) in the job that holds `contents: write` and `id-token: write`, then sign in that same
job. Any build script can read `ACTIONS_ID_TOKEN_REQUEST_TOKEN` and mint a Sigstore identity as
StroggForge's workflow. The vNext shape:

1. **Build (U).** Compile, package, upload an artifact. `contents: read`, no `id-token`, no
   secrets.
2. **Verify and plan (V).** Download the artifacts, run `forge verify` and `forge release plan`:
   portability, digests, names, the frozen manifest. No consumer code runs.
3. **Sign (Cosign step).** `id-token: write`, signs exactly the digests in the manifest. No
   consumer code runs.
4. **Promote (P).** `forge release apply` performs the manifest, behind an `environment:` gate,
   with only the write permission it needs.

SEC-102 owns this split; the forge's job is to make step 2 and 4 small enough to review. Nothing
here changes until SEC-102 is ratified.

## `forge jess-release`

Runs on the operator's machine, never in CI.

- **Does:** checks the tree is clean and tagged, rebuilds or fetches the CI-built archives, checks
  their digests against the CI run's manifest and Sigstore bundles, writes the exact bytes to be
  signed (a canonical release statement), and after signing, checks the signature with the pinned
  public key before anything is published.
- **Does not:** read, reconstruct, or ask for the paper-held secret; call the signer; publish. The
  operator signs with their offline tool between `prepare` and `check`.
- CI and agents get only the public key and the signed statement. A CI job that finds no valid
  human signature on an official Jess release stops before any publish step.

The statement format and the signature scheme are SEC-103's. This RFC fixes only where the
boundary sits.

## Replacement table

From `inventory/components.json`, grouped. "Native" means the forge starts the tool; the decision
around it moves.

| Today | vNext owner | Native boundary kept | Lands with |
| --- | --- | --- | --- |
| `tools/war-room` validate/model/workflows | `forge check`, `forge inspect` | — | FOR-201, FOR-202 |
| `tools/war-room` generate, `check-site.py`, Zola, `render-diagrams.mjs` | `forge build` over `@dreamweave/site` | Mermaid: **open** (FOR-103) | FOR-203 |
| `resolve-build-context`, `binary-suffix`, `feature-args`, `resolve-crate-manifest`, `resolve-msrv` | `forge/release` planning | `cargo metadata` | FOR-204 |
| `create-release-archive`, `package-muxapp`, `prepare-github-release-artifact`, `jess-image/prepare-context` | `forge/release` archive | `7z` until JSL writes the zip format bytes identically | FOR-204 |
| `check-release-binary` | `forge verify` | `otool`/`readelf` until JSL reads load commands | FOR-204 |
| Nexus scripts (6) | `forge/nexus` | the upload action | FOR-204 |
| `changelog.sh`, `docs-index.sh`, `gen_benchmarks.py`, `generate-benchmark-docs.sh` | forge modules | `git log`, `cargo bench` | FOR-204 |
| `publish_workspace.py` | crate order, skip, retry in JSL | `cargo publish` | FOR-204 |
| `refresh-github-release`, `publish-github-release-artifacts`, dependent issues | `forge release apply` | `gh` | FOR-204, after SEC-102 |
| `sign-release-binary` | stays a Cosign step | `cosign` | — |
| `setup-llvm.sh`, `rust-setup`, `jess-setup` | stay native bootstrap; `forge doctor` reports | rustup, apt/dnf/conda, curl + digest | — |
| builder images, Dockerfiles | stay | Docker | — |
| mod-template `commit-records`, `scaffold`, `sync-pins` | the Mod Template's own targets, not the forge | `git push` | coordinate with the template |
| `bump-version.sh` | `forge release bump` | `git` | FOR-204 |
| `createRelease.yml`, `dependent.yml` | retired: their only caller is deprecated (FOR-101) | — | FOR-104 notice, then removal |

## Cross-platform limits

- Jess releases exist for Linux X64, macOS ARM64, macOS X64 and Windows X64. PortMaster and
  Android jobs cross-compile from Linux, so the forge runs on the host, never the target.
- The forge never builds a path by string concatenation with `/`: `@dream/fs.join` everywhere.
  Fixtures use no case twins and no non-UTF-8 names (jess's own test rule).
- Windows `CARGO_BUILD_TARGET` pinning (`83eba44`) stays in `setup-llvm.sh`; the forge reads the
  resulting layout, it does not reproduce the reason.

## Unresolved native constraints

| # | Constraint | Executable alternative until resolved |
| --- | --- | --- |
| N1 | no process timeout in jess 0.3.0 | job `timeout-minutes`; ship spawn-using commands only on a jess with bounded runs |
| N2 | no signature verification in JSL | native verifier (`cosign verify-blob`, `ssh-keygen -Y verify` or SEC's choice) in the bootstrap shell |
| N3 | `@jess/yaml` drops a final line break (FOR-101), and mishandles some `>+` keep chomping mid-document | fixed on a local jess branch (`fix/yaml-clip-eof`, unpushed, awaiting the maintainer); until a release carries it, `inspect` omits `run` bodies and the parity test tolerates exactly the one known byte |
| N4 | `@dreamweave/site` expands shortcodes inside inline code | fix in the generator or escape in `forge build`; decided in FOR-103 |
| N5 | no Mermaid renderer in JSL | FOR-103: schematic JSON, reviewed SVG, or build-only Node |
| N6 | Mach-O/ELF load-command reading | `otool -L`/`readelf -d` from `forge verify` |
| N7 | `@jess/yaml` has no layout: an empty `{}` and `[]`, `{1: a}` and `[a]`, `30.0` and `30`, and a tag are indistinguishable once decoded | the forge records each such divergence from the Rust; a `positions`/layout option like `@jess/data/toml`'s would close it, in jess |

## Coordination with EP-DOC, JECP and DWWP

The forge consumes their contracts and implements none of them.

- **Receipts (DOC-101).** Until DOC-101 ratifies a schema, forge PRs carry receipts as Markdown
  tables: source SHA, exact command, observed result, and what was not run. The forge emits
  `stroggforge.diagnostics/1`, which a receipt can embed as-is. Proposed for DOC-101: a receipt
  references a diagnostics document by digest rather than restating it.
- **Gates (JECP).** A forge exit status is a gate result: 0 pass, 65/77 fail with a reproducible
  input, 1 tool fault. JECP decides what a stage does with each.
- **Cross-repository changes (DWWP).** Consumer cutover PRs (FOR-205) are DWWP's to sequence. The
  forge supplies `inspect` diffs between two StroggForge refs as the compatibility evidence.

## PR sequence and removal list

1. FOR-101: oracle `inspect`, inventory, goldens (this RFC's base).
2. FOR-102: this document.
3. FOR-201: `forge/` with `inspect` and `check`, fixtures, parity tests. No CI change.
4. FOR-202: complete `check`, negative corpus, then `war-room.yml` runs `forge check`; Rust
   `validate.rs`/`model.rs`/`workflows.rs` removed after a release where both ran.
5. FOR-203: `forge build`; Zola, `check-site.py`, Node removed after the site matches.
6. FOR-204: release modules one helper at a time, each with golden and injection tests; shell
   removed per helper after parity, not in bulk.
7. FOR-205: consumer cutover per FOR-104.

Rollback for every step: the previous StroggForge tag still exists and still works; no step
deletes a tag, and the Rust/Python/shell implementation is removed only in a later release than
the one that made the forge authoritative.

## Profile

Measured on this machine (receipts in FOR-101 and FOR-201):

- Rust `check`: 0.03 s, ~6 MB, after a Rust toolchain and a ~1 s incremental build (cold CI:
  `dtolnay/rust-toolchain` plus a crate build, about a minute).
- JSL `inspect` prototype: 0.06-0.07 s, ~28 MB, on an installed jess, no compile step.

The CI saving is the toolchain, not the run: `war-room.yml` installs Rust, Node, Zola and Python to
check 1,000 lines of TOML. With a digest-checked jess download (~10 MB) the validation job should
fall to seconds; the claim gets measured in FOR-202 before it is repeated anywhere.

Maintenance: four languages (Rust, Python, Node, Bash) plus YAML become JSL plus YAML plus the
native tools listed above. Bash stays only inside bootstrap steps that must run before jess exists.

## Open decisions

1. Who ratifies this RFC, and who is final approver for FOR-201 merge (CIV-000 leaves it open).
2. ~~Whether the unlisted callers may be named~~: settled 2026-11-12. jess and Starwind-Builder are listed; a deprecated caller is left out (FOR-101).
3. Whether the AUR job's push-to-main behavior is intended (FOR-101 defect 2).
4. Bootstrap option A now, or wait for D.
5. Whether `forge` lives in this repository long-term or becomes its own package repository
   published like the Mod Template.
