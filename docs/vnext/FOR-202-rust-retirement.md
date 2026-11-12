# Retiring the Rust war room's validator

[FOR-202](https://github.com/DreamWeave-MP/StroggForge/issues/39) asks for the Rust validator's
removal "after parity", with "Rust check no longer required for ordinary CI when cutover passes".
Parity on the corpus is demonstrated (see the [receipt](FOR-202-receipt.md)). Cutover is not
done, and this plan doesn't do it: each step below names the gate it waits on and who opens it.
Nothing here changes a workflow.

## What the Rust crate still does

`tools/war-room` is three things, and only the first is replaced:

| Part | Files | Replaced by | When it can go |
| --- | --- | --- | --- |
| `check`: model, schema, references, cycles | `model.rs`, `validate.rs`, `workflows.rs` | `forge check` | step 3 below |
| `inspect`: the contract golden | `inspect.rs` | `forge inspect` | step 3 below |
| `build`: the site's generated content and graphs | `generate.rs`, `graphs.rs` (which read `model.rs`) | `forge build`, FOR-203 | after FOR-203 |

`generate.rs` reads the model through `model.rs`, so the crate as a whole outlives FOR-202. What
FOR-202 can retire is the crate's role as the validator.

## Steps

1. **A jess that runs the forge exists as a release.** The forge needs `@jess/yaml`'s
   `decodeWithLayout` ([jess#30](https://github.com/DreamWeave-MP/jess/pull/30)); no released
   jess has it, and on jess 0.3.0 the forge stops and says so. Gate: jess#30 reviewed and merged by
   a human, then an official jess release, which is always a human, offline-signed act (SEC-105,
   SEC-206, SEC-207). No agent opens this gate.
2. **CI can fetch that jess safely.** `war-room.yml` running the forge means a jess download in CI.
   Doing that without an independently authenticated bootstrap is the thing SEC-201 exists to stop.
   Gate: SEC-201 (blocked by SEC-102, itself blocked by SEC-101).
3. **Shadow, then switch.** With 1 and 2 open, and with separate authorization to change
   `war-room.yml` (a workflow that deploys Pages), one PR adds a job that runs, from the same
   checkout:
   - `jess package test forge`,
   - `jess workspace run --trusted forge-check`,
   - `forge-inspect` compared byte for byte with `docs/vnext/golden/workflow-contracts.jsonl`,
   - `forge/tests/differential.jsl` against the Rust binary the job already builds, failing on any
     unexplained disagreement.

   The Rust `check` keeps running beside it for one StroggForge release. In the next release, the
   Rust `check` step goes and the forge's is the gate; `cargo run -- build` stays for the site.
4. **Freeze the oracle.** When the Rust `check` is gone from CI, the differential stops being run
   against a live Rust binary. Its last receipt, `forge/tests/fixtures/oracle.json` and both
   goldens stay as recorded evidence; the suite already reads only those, never the Rust.
   `validate.rs` and `workflows.rs`'s validation paths are deleted then. `model.rs` stays for
   `generate.rs` until FOR-203.
5. **After FOR-203**, the crate, the root `Cargo.toml`, `Cargo.lock`, `clippy.toml` and the Rust
   toolchain step in `war-room.yml` go, in a release after the one where `forge build` became
   authoritative.

Rollback at every step: the previous StroggForge tag still exists and still works, and the Rust is
deleted only in a release after the one that made the forge authoritative (FOR-102).

## What changes for consumers

Nothing. `war-room.yml` is this repository's own CI; no reusable workflow or template that a
consumer calls is touched by any step above.
