# SEC-101 evidence: the three inventories

These are the primary-evidence sweeps behind [`SEC-101-threat-model.md`](SEC-101-threat-model.md),
read on 2026-11-12. Three independent read-only reviewers produced them, one per repository. The
headline claims were re-checked at source before the RFC relied on them (see "Re-checked").

- **V** means verified in source or settings.
- **I** means inferred from documented platform behavior.

Paths are relative to each repository. AsHyAmS's crawler is under `tools/dreamweave-network/src/`.
The first draft cited AsHyAmS line numbers that don't exist; they're corrected below and in the
RFC.

## Re-checked by the RFC's author

| Claim | Check | Result |
| --- | --- | --- |
| A tag name reaches the shell | `git show origin/main:.github/actions/corprus-crucible/action.yml`, lines 233, 245, 262; `rustGlobalBuild.yml:328` (`release_name: ${{ github.ref_type == 'tag' && github.ref_name … }}`) | V: spliced in double quotes |
| `jess-setup` checks nothing | `jess-setup/action.yml:27-33` | V: `curl`, `unzip`, PATH |
| No rulesets; `main` unprotected; SHA pinning not required | `gh api repos/DreamWeave-MP/StroggForge/rulesets` (0), `/branches/main` (`protected: false`), `/actions/permissions` (`allowed_actions: all`, `sha_pinning_required: false`) | V |
| jess's default index is AsHyAmS | jess `src/cli/workspace.rs:13-16` | V |
| jess accepts `http://` URLs | jess `src/lua/stdlib/publisher.jsl:155-157` | V |
| The dev jess binary verifies against its bundle | `cosign verify-blob` on `jess-Linux-X64.zip` from `dev` (`1a4be2f`); certificate SAN `…/rustGlobalBuild.yml@refs/tags/v70`, source `DreamWeave-MP/jess` at `1a4be2f`, `refs/heads/main` | V |

## 1. StroggForge (`72c3050`, tag `v72`)

### 1.1 Jobs, permissions, secrets, privileged operations

**Repository settings** (V, `gh api`):
- rulesets: none;
- `main`: unprotected;
- `default_workflow_permissions: write`;
- `allowed_actions: all`, `sha_pinning_required: false`;
- fork PRs need approval from first-time contributors only;
- immutable releases: off.

**Triggers** (V). No `pull_request_target`, `workflow_run`, `issue_comment` or `repository_dispatch`
anywhere.

**Consumer templates.**
- `rust_template.yaml:3-10` and `lib_template.yaml:3-10` trigger on dispatch, pull requests, pushes
  to `main` and any tag. `mod_template.yaml:6-12` triggers on any branch, `*-[0-9]*` tags, pull
  requests and dispatch.
- Each passes `secrets: inherit` (`:19`, `:19`, `:29`).
- Each grants `contents: write, pages: write, id-token: write` (`rust_template.yaml:14-17`). The mod
  template adds `actions: write`.

**`rustGlobalBuild.yml`** has no workflow-level permissions. Jobs without a block inherit the
caller's grants (I).

| Job | Line | Permissions | Secrets | Privileged operation |
| --- | --- | --- | --- | --- |
| lint | 145 | caller's | — | `cargo audit`, publish dry run |
| test (4 OS) | 228 | caller's | — | runs repository code |
| release (mac/win) | 307 | contents:write, id-token:write (315-317) | VT_API_KEY, NEXUS_API_KEY, NEXUS_GROUP_IDS (327-331) | build, cosign sign, VirusTotal upload |
| release-linux | 336 | same | same | container `…el9-builder:v72` (344) |
| release-android | 365 | same | same | `nttld/setup-ndk@v1` |
| release-portmaster | 425 | same | same | container `…portmaster…:v72` (434) |
| publish | 474 | contents:write (479-480) | GITHUB_TOKEN; DW_BOT_PAT (586) | deletes and recreates the release and the `development` tag (528 → `refresh-github-release.sh:18,22,39`); uploads; opens issues in other repositories (`create-dependent-update-issue.sh:50`) |
| nexus | 594 | caller's | NEXUS_API_KEY (621) | Nexus upload |
| crates | 631 | caller's, no environment | CARGO_REGISTRY_TOKEN (657) | `cargo publish` (665-675) |
| aur | 680 | environment `AUR` (685) | AUR_SSH_PRIVATE_KEY (692) | AUR push |
| docs | 699 | pages:write, id-token:write | — | Pages deploy |
| benchmarks | 738 | contents:write | GITHUB_TOKEN | release asset upload |
| mod-template | 751 | actions, contents, id-token, pages: write | inherit | see `modGlobalBuild` |
| notify | 779 | — | Discord webhooks | Discord post |

Which events each privileged job runs on:
- `publish` excludes only `pull_request` (477), so a `workflow_dispatch` from any branch refreshes
  the `development` release (V).
- `aur` requires a push (683).
- `crates` requires a tag (634).

**`libGlobalBuild.yml`** has the same shape:
- `publish` (210) has contents:write, deletes and recreates the release (239) and uses DW_BOT_PAT
  (254);
- `crates` (261) has no environment and uses CARGO_REGISTRY_TOKEN (287);
- `docs` (308) deploys Pages.

**`modGlobalBuild.yml`** sets `contents: read` at the top (48-49).

- **`site`** (74) has contents:write and id-token:write (78-81). It runs on pull requests too, and
  runs `jess workspace run --trusted` on repository content (118, 159-161). On the default branch
  it also:
  - commits a scaffold (107);
  - pushes `jess.lock` and `mod.lock` commits to the default branch (`sync-pins.sh:73-75`,
    `commit-records.sh:43`);
  - signs with cosign (182);
  - deletes the development release and its tag, then recreates them (198-210).
- **`release`** (233) has actions, contents and id-token write. It signs (314), records `mod.lock`
  (324), creates the release with `--verify-tag` (343) or re-uploads with `--clobber` (339), and
  dispatches a run (379).

**Other workflows:**

| Workflow | Permissions | Secrets and effects |
| --- | --- | --- |
| `createRelease.yml` | none set | deletes and recreates releases and the `development` tag (45) |
| `dependent.yml` | none set | DW_BOT_PAT (33): issues in other repositories |
| `publish-*-el9-builder.yml` | packages:write | pushes container images to GHCR on dispatch, pushes to `main` and `v*` tags (3-16, 64-70) |
| `publishJessImage.yml` | packages:write | pushes the jess image (76-81) |
| `war-room.yml` | contents:read; `deploy` has pages:write | deploys Pages |

### 1.2 Third-party pinning

Only two references are pinned to a SHA:
- `sigstore/cosign-installer@faadad0…` (`corprus-crucible/action.yml:172`);
- `actions/upload-pages-artifact@fc324d3…` (`modGlobalBuild.yml:212`).

**Branch-pinned**, mutable by design:
- `dtolnay/rust-toolchain@master` (`rust-setup/action.yml:40`; used by the crates job);
- `@stable` (`corprus-crucible/action.yml:72`, `toolchain.yml:51`, `war-room.yml:21`, the
  daily-quality template);
- **`Nexus-Mods/upload-action@main`** with the Nexus key (`modGlobalBuild.yml:396`);
- `taiki-e/install-action@cargo-audit` (`rustGlobalBuild.yml:174`, `libGlobalBuild.yml:110`).

**Tag-pinned, with secrets or write grants nearby:**
- `crazy-max/ghaction-virustotal@v5` (VT key, in a job holding contents:write and id-token);
- `varrcan/aur-publish-action@v1` (AUR key);
- `Nexus-Mods/upload-action@v1.0.0-beta.5` (Nexus key);
- `tsickert/discord-webhook@v7.0.0`;
- `docker/login-action@v3` and `docker/build-push-action@v6` (GHCR push);
- `sigstore/cosign-installer@v3` (`modGlobalBuild.yml:174,306`);
- `actions/*` from v5 to v8 throughout.

**Own references:** `@v72` and `ref: v72` throughout (for example `rustGlobalBuild.yml:156,159,322`
and `modGlobalBuild.yml:92,95`).

**Images:**
- builder images by tag (`rustGlobalBuild.yml:344,434`);
- the base `quay.io/centos/centos:stream9` by tag;
- only the jess image is pinned by digest (`modGlobalBuild.yml:56`);
- no image is signed.

**Runtime tool installs:**

| Install | Location | Integrity check |
| --- | --- | --- |
| LLVM tarballs | `setup-llvm.sh:53-56,136,159,181` | TLS only |
| micromamba `latest` | `setup-llvm.sh:71` | TLS only |
| conda-forge clang/lld | `setup-llvm.sh:72-74` | major version only |
| apt.llvm.org | `setup-llvm.sh:96-99` | key fetched each run |
| dnf | — | default gpgcheck (I) |
| jess | `jess-setup/action.yml:27-30` | none |
| actionlint | `war-room.yml:40` | none |
| `DREAMWEAVE_BOOTSTRAP_SOURCES` git clones, later run `--trusted` | `sync-pins.sh:39-50` | none |

### 1.3 Signing, scanning, auditing

**Cosign:**
- signs the raw binary (`sign-release-binary.sh:12`), skipped on pull requests (`action.yml:171,178`);
- the bundle goes inside the zip (`create-release-archive.sh:14-15`);
- mod archives are signed at `modGlobalBuild.yml:182,314`, under the identity in
  `DREAMWEAVE_SIGNING_IDENTITY` (71), which `bump-version.sh:48` rewrites;
- **no `cosign verify` or `verify-blob` anywhere** (V).

**VirusTotal:**
- an empty key fails the job (`validate-virustotal-key.sh:6-10`);
- results only become links in the release body (`format-virustotal-links.sh:17-21`;
  `rustGlobalBuild.yml:530-541`); no detection gates anything (V).

**`cargo audit`:**
- runs in `lint` (`rustGlobalBuild.yml:177`);
- `publish` and `crates` require `lint` (477, 634); release builds don't wait for it;
- a missing `Cargo.lock` is generated fresh (168-172).

**`check-release-binary.sh`** checks portability, not security.

**SBOM and SLSA provenance:** none.

### 1.4 Script injection

- **Tag names.** `inputs.release_name` is `github.ref_name` on tags (`rustGlobalBuild.yml:328`). It
  is spliced inside double quotes at `corprus-crucible/action.yml:233,245,262`. `git
  check-ref-format` accepts tag names containing `$(` (V), so a tag name executes in a job holding
  the VT and Nexus keys, contents:write and id-token:write.
- **Secrets spliced into script text:** `action.yml:58` (`vt_api_key`) and `action.yml:220`
  (`nexus_api_key`).
- **Caller inputs in single quotes:**
  - `dependent.yml:35`;
  - `rustGlobalBuild.yml:279,727`;
  - `libGlobalBuild.yml:336`;
  - `action.yml:68,90,134-153,189-195`.
- **Passed through `env`, which is correct:** `discord.yml`, the image workflows, and
  `modGlobalBuild` except where `inputs.*` appears only in an `if:`.

### 1.5 Cross-repository trust

- Consumer templates pin `@v72` with `secrets: inherit` (`rust_template.yaml:18-19` and the others).
- Every script-using job checks out `DreamWeave-MP/StroggForge` at `ref: v72` (for example
  `rustGlobalBuild.yml:153-157`).
- With no tag protection (V), moving `v72` runs attacker code in every consumer, holding:
  - CARGO_REGISTRY_TOKEN, AUR_SSH_PRIVATE_KEY, NEXUS_API_KEY, VT_API_KEY and DW_BOT_PAT;
  - GITHUB_TOKEN with write grants;
  - OIDC.

  Signatures made afterwards carry the same `@refs/tags/v72` identity (I).
- A `workflow_dispatch` at a moved tag re-pushes that tag's builder image.
- `refresh-github-release.sh:16-22,39` deletes tagged releases and the `development` tag, then
  recreates the release without `--target`, so it lands on the default branch head (I).
- `bump-version.sh:45-52,75-77` rewrites references with `sed` and tags locally. It never pins a SHA
  or a digest.

### 1.6 Fork and same-repository pull requests

**Fork PRs** run lint, test and unsigned release builds:
- read-only token, no secrets, no OIDC (I: GitHub policy);
- `publish`, `crates`, `aur`, `docs`, `nexus` and `notify` are skipped by `event_name` conditions
  (477, 634, 683, 702, 781);
- caches are restore-only (V).

**Same-repository branch PRs** run the same jobs, but GITHUB_TOKEN carries the caller's grants, and
the release jobs have contents:write and id-token:write. PR code then runs:
- `build.rs`, tests and proc macros;
- the `.stroggforge/cargo-build-args.sh` hook (`feature-args.sh:12-29`);
- `jess … --trusted`.

No checkout sets `persist-credentials: false` (V).

## 2. jess (`1a4be2f`)

### 2.1 Release and distribution

**`release.yaml`:**
- calls `StroggForge/.github/workflows/rustGlobalBuild.yml@v70` (`:20`), a tag pin;
- passes `secrets: inherit` (`:21`);
- grants actions, contents, pages and id-token write (`:15-19`);
- runs on pull requests, pushes to `main`, any tag and dispatch (`:3-8`);
- calls `publishJessImage.yml@v70` on tags (`:43-51`).

**What it publishes:**
- pull requests produce workflow artifacts;
- pushes to `main` refresh the rolling `dev` release, which is deleted and recreated, so its assets
  are mutable;
- tags produce a release and `cargo publish` (`releasing.md:41-45`).

**The image** is built `FROM quay.io/centos/centos:stream9` by tag (`Dockerfile:13`) and is not
signed.

**Sigstore:**
- the bundle covers the binary inside the zip;
- nothing in jess verifies it;
- `site/content/docs/getting-started/install.md:35-37` documents `--certificate-identity-regexp '^https://github.com/DreamWeave-MP/'`,
  which accepts any workflow in the org.

**Consumers get jess** through `jess-setup` (no check) or the image by digest (`modGlobalBuild.yml:56`).

**`docs.yaml`** uses tag-pinned third-party actions (`:58,60`).

### 2.2 Package install

**Index:**
- default `https://dreamweave-mp.github.io/AsHyAmS/network-data/jess-packages-v1.json`
  (`src/cli/workspace.rs:16`);
- overridable by `--index` or the `JESS_REGISTRY_URL` variable (`src/cli/install.rs:56`);
- the index decides the manifest for an id not named with `--publisher` (`publisher.jsl:668-688`);
- two claimants are an error (`:661-682`).

**Manifest:** unsigned JSON (`publisher.jsl:168-187`).

**URLs:** `^https?://`, so plain `http://` is accepted (`publisher.jsl:155-157`).

**Artifacts, checked in order and failing closed:**
- size;
- SHA-256 against the manifest (`publisher.jsl:562-600`);
- id, version and BLAKE3 content digest against the manifest (`workspace.jsl:1740-1770`).

**`jess.lock`** pins the BLAKE3 digest, the manifest URL and the artifact's SHA-256. It holds no
signature or key (`jess.lock:3-12`). `--locked` refuses drift (`workspace.jsl:1553-1556`).

**`jess install`:**
- checks the `.jsw` index against the manifest (`install.jsl:488-530`);
- prompts only when commands or capabilities grow (`install.jsl:673-695`), so an upgrade that changes
  code under the same capabilities installs without a prompt;
- checks no continuity of the manifest URL (I).

**Not present:**
- signatures, publisher identity, trust roots, rotation;
- calls to l3i's `fs.openAuthenticated` (compiled in, unused).

### 2.3 Capabilities

- **Coarse names.** `filesystem.*` is not path-scoped. `network.tcp.connect` reaches any address,
  loopback included (`capabilities.md:15-35`; l3i `src/tcp/mod.rs:49-50`).
- **What grants what:**
  - `jess script.jsl` and `jess run --trusted` hold every capability (`src/cli/run.rs:101-133`);
  - `.jsw` apps and standalone executables grant all declared capabilities without a prompt
    (`src/cli/workspace.rs:188-219`);
  - package targets hold what they declare (`src/lua/package.rs:1395-1420`).
- **No per-package isolation** (`src/lua/package.rs:13-16`).
- **Install runs no package code** (`src/cli/install.rs:188-191`).

### 2.4 Rust supply chain

- `Cargo.lock` is committed: 407 registry packages, all with checksums.
- l3i is at 2.9.0.
- `vendor/` holds only miniaudio.
- `build.rs` compiles C, with no downloads.
- l3i's `build.rs` compiles Luau from a pinned submodule and checks the toolchain. It reads
  `CARGO_ENCODED_RUSTFLAGS`, `LUAU_CXXFLAGS` and `L3I_UNVERIFIED_TOOLCHAIN`.
- Proc macros: clap derive, serde derive.
- No `deny.toml`, `audit.toml`, Dependabot or `SECURITY.md`.
- Release builds don't pass `--locked` (StroggForge v70 `build-release-binary.sh:36-54`).

### 2.5 Official release signing

Code has none: only keyless cosign on the runner. No design document for a paper key, offline
signing or enrollment exists in the repository or its handoffs.

### 2.6 Network

`@jess/http`:
- follows cross-origin redirects by default, stripping headers (`http/init.jsl:55-58,308-320,862`);
- refuses an https-to-http downgrade (`:1076-1079`).

No destination filtering, no DNS pinning (I).

## 3. AsHyAmS (`b955dc87`)

### 3.1 What it is

**It describes itself as** "Not a registry", "not a host", "not required" (`Readme.md:3-20`).

**What it crawls and when:**
- the `network/sources.toml` sites (`:16-59`; additions by PR);
- on cron every six hours, on push to `main` and on dispatch (`network.yml:10-16`).

**Where its state goes:** the `network-state` branch, pushed by the bot on the default branch only
(`scripts/network-state.sh:50-58`).

**What it publishes:**
- Pages;
- `network-data/` exports including `jess-packages-v1.json` (`src/catalog.rs:9-14,37`), which calls
  itself non-authoritative (`:40-41`).

**How jess uses it:** as its default index (above). So the index is in practice the authority for
which publisher an unpinned id resolves to.

### 3.2 Networking controls (the model for jess)

**SSRF:** global unicast only. Every private, loopback, link-local, CGNAT, documentation and
multicast range is refused, IPv4-mapped and NAT64 addresses are judged as IPv4, and tests cover the
metadata endpoints (`tools/dreamweave-network/src/address.rs:24-174`).

**DNS:**
- a name is refused if any address it resolves to is refused;
- connections go only to the checked addresses;
- no proxy (`tools/dreamweave-network/src/fetch.rs:146-186`).

**Redirects:** manual, at most five, and each hop re-checked (`fetch.rs`, after `check_url` at 193; `policy.rs:25`).

**Gaps:**
- plain `http://` is allowed, including https-to-http redirects (`fetch.rs` `check_url`, 193);
- no port restriction.

**Limits:** sizes, time and concurrency are capped; there is no decompression (`tools/dreamweave-network/src/policy.rs:20-54`).

**Media:** the declared type must match the sniffed type, from an allowlist without SVG.

### 3.3 Trust and injection

**Validation:**
- schema validation with `deny_unknown_fields`;
- the manifest's hash is checked against the index's, both from the same host (`src/crawl.rs:335-366`).

**What it doesn't verify:**
- signatures or artifact digests (`pages/about/trust.md:25-33`);
- that the manifest URL is on the index's host (`src/protocol/parse.rs:268-271`).

**Injection defenses:**
- publisher Markdown goes through an `ammonia` allowlist (`src/markdown.rs:14-64`);
- templates mark as safe only sanitized or self-escaped HTML.

**Injection gap:** the gaps page offers `cargo network inspect {url}` for copying without shell
quoting (`src/site/pages.rs:1590`, I).

### 3.4 CI

**Permissions:**
- `network.yml` is `contents: read` at the top;
- `refresh` has contents:write, `deploy` has pages:write and id-token:write;
- `check.yml` runs on `pull_request` with read-only grants.

**Pinning:** every action by tag. Three third-party actions run in the writing job. Checkouts keep
their credentials.

**Downloads:** actionlint by `curl | bash` (`check.yml:51`).

**Settings:** `main` and `network-state` unprotected; no rulesets; `default_workflow_permissions: write`.

### 3.5 JSL port

There is none in this repository. FED-103 tracks it.
