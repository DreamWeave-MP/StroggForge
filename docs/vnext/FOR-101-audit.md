# FOR-101: what StroggForge actually does, and who depends on it

Status: **proposed** for review under [FOR-101](https://github.com/DreamWeave-MP/StroggForge/issues/17).
Nothing here authorizes implementation, merge, release or deployment.

This is the inventory the vNext rewrite is held to. It counts behavior, not files: what a
consumer can observe, what can mutate a release, and which language should own it afterwards.

## Evidence

| What | Value |
| --- | --- |
| StroggForge | `main` at `72c3050` (tag `v72`). This branch adds only the war room's `inspect` command and its root argument. |
| Rust oracle | `stroggforge-war-room`, rustc 1.99.1, built from this branch |
| Jess | `jess 0.3.0` (`~/.cargo/bin/jess`), the shipped release |
| Consumers | default branches read through the GitHub API at 2026-11-12T17:41:17Z, HEAD SHAs recorded per repository |

Machine-readable results:

- [`inventory/components.json`](inventory/components.json): 64 executable components (8 reusable
  workflows, 4 other workflows, 4 composite actions, 5 templates, 3 images, 35 shell scripts,
  3 Python, 1 Node, the Rust war room), each with its trigger, inputs, outputs, reads, writes,
  network, secrets, permissions, native tools, callers, hard-coded StroggForge refs, mutable refs,
  and an intended owner. Six owners were overridden on review; those entries carry
  `reviewed_owner` and `review_note`.
- [`inventory/consumers.json`](inventory/consumers.json): every caller found, the exact ref each
  pins, the inputs and secrets it passes, and the permissions it grants.
- [`golden/workflow-contracts.jsonl`](golden/workflow-contracts.jsonl): the events, inputs,
  secrets, outputs, permissions, concurrency, env, jobs and step contracts of all 16 workflows and
  actions, as the Rust war room reads them. SHA-256 `303bb8db…602e7d`.
- [`golden/war-room-build.sha256`](golden/war-room-build.sha256): digests of all 93 files the Rust
  war room generates. Two clean builds produced identical manifests.

Regenerate the goldens with:

```sh
cargo run --locked -p stroggforge-war-room -- inspect > docs/vnext/golden/workflow-contracts.jsonl
rm -rf site/content site/static/generated
cargo run --locked -p stroggforge-war-room -- build
(cd site && find content static/generated -type f -print0 | LC_ALL=C sort -z | xargs -0 sha256sum) \
  > docs/vnext/golden/war-room-build.sha256
```

The goldens are snapshots for parity, not CI gates: no workflow change has to update them, and no
CI job reads them. The JSL forge's tests (FOR-201) compare against them.

## Ownership, by count

| Intended owner | Components |
| --- | --- |
| YAML entrypoint (events, runners, permissions, environments) | 13, plus `createRelease` and `dependent` kept for an external caller |
| JSL forge modules | 33: release 10, nexus 6, check 4, archive 3, mod-template 3, war room 3, benchmarks 2, changelog 1, docs 1 |
| Native boundary | 15: Docker images 6, LLVM install 2, `gh release` 2, rustup 1, jess install 1, cargo build 1, Cosign 1, cargo publish 1 |
| Retire | `scripts/cargo-build-args.example.sh` (superseded by the template's documentation) |
| Undecided | `render-diagrams.mjs`: there is no JSL Mermaid renderer (FOR-103) |

23 components are privileged: they can mutate a release, tag, registry, Pages, a default branch or
an issue, sign through OIDC, or use a write token. They are listed with `privileged: true`.

## Consumers and their pins

Callers named in `war-room/ecosystem.toml`, by the ref they pin:

| Ref | Caller → workflow |
| --- | --- |
| v72 | l3i → `libGlobalBuild`, `benchmarks`, `actions/setup-llvm` |
| v70 | dream-soft-render → `libGlobalBuild`, `actions/setup-llvm` |
| v69 | dream_archive, vfstool, Openmw_Config, S3LightFixes, dream-ini, dream_archivetool → `modGlobalBuild` |
| v68 | DreamWeave-Mod-Template → `modGlobalBuild` |
| v55 | dream-net → `libGlobalBuild` |
| v53 | Morrobroom, greenmote → `rustGlobalBuild`; dream_path → `libGlobalBuild`; S3ctors-S3cret-St4sh and dream-soft-render → `modGlobalBuild` |

Three more callers are not in `ecosystem.toml` (`unlisted-1..3`, pinning v6, v70 and v6). Their
names are withheld until the maintainer says which organization repositories may be listed.
One of them still calls `createRelease` and `dependent` at v6.

Uncertainty: GitHub code search missed five callers we already knew, so it cannot prove there are
no others. Only enumerating every organization repository would.

Seven refs are in use at once (v6, v53, v55, v68, v69, v70, v72). The "no permanent v70
compatibility tax" goal in EP-FOR has to start from v53, not v70.

### Where the war room disagrees with the callers

- dream_archive and Openmw_Config are recorded as `libGlobalBuild` callers; vfstool, S3LightFixes,
  dream-ini and dream_archivetool as `rustGlobalBuild`. Each one's only StroggForge call is a
  `build_site.yml` calling `modGlobalBuild@v69` directly. Either their Rust CI was dropped or the
  records are stale; the maintainer knows which.
- dream-soft-render calls `modGlobalBuild@v53` directly, besides `libGlobalBuild@v70`, and its two
  pins disagree with each other.
- l3i also calls `benchmarks` and `actions/setup-llvm`, which the record leaves out.
- `AGENTS.md` says St4sh and the Mod Template call only `createRelease`. Both call
  `modGlobalBuild`, which is what `ecosystem.toml` says.

## Defects found on the way

Read from the code at `72c3050`; none of these was executed. They are reported, not fixed: fixing
them changes production CI and is not FOR-101's business.

1. **A consumer's feature hook is erased before the release-feature lint.** `rustGlobalBuild`'s
   `test` job checks StroggForge out into `.stroggforge` (`rustGlobalBuild.yml:257-261`), the same
   folder a consumer's `.stroggforge/cargo-build-args.sh` lives in (`feature-args.sh:12`), then
   runs `clippy-release.sh` (`:278`). The checkout cleans the folder, so "Clippy with the
   features the release ships" lints default features. greenmote carries that hook. Release
   builds are unaffected: they don't check StroggForge out there. The `crates` job's
   `rm -rf .stroggforge` (`:664`, `:668`) would also delete a tracked hook and dirty the tree
   `cargo publish` refuses. Present at v53 as well.
2. **The AUR job runs on every push.** `aur` (`rustGlobalBuild.yml:683`) is gated on
   `github.event_name == 'push'`, so development pushes to the default branch publish too.
   Confirm whether that is the intent (a `-git` package) or a missing tag check.
3. **Caller input and secrets are spliced into script text**, against the `AGENTS.md` rule:
   caller inputs at `rustGlobalBuild.yml:277-280` and `:727`, `libGlobalBuild.yml:336` and
   `dependent.yml:35`; secrets at `corprus-crucible/action.yml:58` (the VirusTotal key) and `:220`
   (the Nexus key). Step and matrix outputs are spliced the same way at `rustGlobalBuild.yml:546-551`
   and `:617`.
4. **A downloaded artifact is sourced as shell.** `read-nexusmods-upload-artifact.sh:20` runs
   `source` on a `manifest.env` from an artifact. Its writer escapes values, but the reader trusts
   whatever the artifact holds.
5. **Unverified executables.** `jess-setup` downloads jess (v0.2.2) with no digest check;
   `setup-llvm.sh` takes micromamba `latest` and LLVM tarballs unchecked; `war-room.yml` pipes the
   actionlint installer into bash. Builder images are pulled by tag (`:v72`), and all three
   Dockerfiles start from the floating `quay.io/centos/centos:stream9`. Only the jess image is
   pinned by digest.
6. **Mutable third-party refs.** Of 28 distinct `uses:` refs, two are SHA pins. Four follow a
   branch: `Nexus-Mods/upload-action@main`, `dtolnay/rust-toolchain@master` and `@stable`,
   `taiki-e/install-action@cargo-audit`. Two different Nexus upload actions are in use
   (`@main` in `modGlobalBuild.yml:396`, `@v1.0.0-beta.5` in `rustGlobalBuild.yml:619`).
7. **The EL9 LLVM mismatch does not fail.** `AGENTS.md` says a stream/rustc LLVM major mismatch
   fails loudly, then that conda-forge is used when the stream trails. `setup-llvm.sh:119-128` does
   the second. `AGENTS.md` should drop the first claim.
8. **Smaller ones.** `NEXUS_API_KEY` and `NEXUS_GROUP_IDS` are declared secrets that `AGENTS.md`'s
   secret contract omits. `AGENTS.md` says `modGlobalBuild` runs the consumer's `./buildSite`; it
   runs `jess workspace run --trusted dreamweave-*`. `discord.yml`'s `webhook_url` input is unused.
   `corprus-crucible` installs stable Rust itself and so ignores the caller's MSRV and toolchain.
   `modGlobalBuild` uses `actions/checkout@v7` where everything else uses `@v6`, and its
   `cosign-installer@v3` pins no Cosign version where `corprus-crucible` pins both.

## Jess findings that bound the rewrite

- **`@jess/yaml` drops a final line break.** A clip-chomped block scalar that ends the document
  loses its last newline: `a: |\n  x\n` decodes to `"x"`, where YAML 1.2.2, serde_yaml and PyYAML
  give `"x\n"`. It is the one byte where a JSL reading of all 16 workflows differs from the Rust
  oracle (the last step of `rust-setup`, `restore-keys`). Cause: the line splitter appends a
  synthetic empty line at end of input with `ended = false`, and the block scalar loop takes its
  `ended`. A fix belongs in Jess, not in a forge workaround.
- **`@dreamweave/site` reads shortcodes inside inline code.** Building the generated war room with
  `org.dreamweave.site` 0.1.1 stops at `` `${{ fromJson(inputs.binary_names) }}` `` in a generated
  page: it looks for a `fromJson` shortcode. Zola leaves code spans alone. FOR-103/FOR-203 must
  settle this before the site moves.
- **No signature verification primitive.** Jess 0.3.0 has SHA-256, BLAKE3, streaming hashers and
  RSA/ECDSA *signing*, but nothing that verifies a signature. Verifying an official Jess release
  inside JSL needs one, in l3i.
- **`@dream/process.run` in 0.3.0 has no timeout.** Bounded process runs arrive with l3i 2.8 on an
  unreleased Jess branch. Until a release carries them, the job's `timeout-minutes` is the only
  bound on a native tool the forge starts.

## Receipt

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | pass |
| `cargo test --locked --workspace --all-features` | pass, 13 tests (5 new for `inspect`) |
| `cargo clippy --locked --workspace --all-targets --all-features -- -W clippy::pedantic -D warnings` | pass |
| `stroggforge-war-room check` ×3 | pass, 0.03 s, ~6 MB |
| `stroggforge-war-room build` ×2 into a clean tree | 0.04 s; identical 93-file manifests |
| JSL prototype of `inspect` (`jess run --capability filesystem.read`) ×3 | 0.06-0.07 s, ~28 MB; identical to the oracle except the `@jess/yaml` byte above |
| `jess run` probe of `@dreamweave/site` over `site/` | fails at the `fromJson` shortcode, as described |

Not run: `actionlint` (no workflow changed), any consumer workflow, any release path.
