+++
title = "Supply-chain security"
description = "The signing boundary, credentials and what checks actually establish."
weight = 25
+++

## Quality and provenance

Tests, formatting, pedantic Clippy and RustSec block the standard release path. MSRV is
checked when configured. None of these makes an artifact intrinsically trustworthy;
they are reviewable controls with a concrete workflow/run identity.

Corprus Crucible signs the **built binary** with keyless Cosign before packaging. The
verification bundle travels in the archive. Verify the extracted binary against that
bundle and the expected workflow identity/issuer; do not claim the archive itself was
signed. VirusTotal scans the archive and records analysis links. A scan is not a guarantee
that a binary is benign.

## Credential boundaries

- `GITHUB_TOKEN`: built-in token, constrained by caller/job permissions.
- `id-token: write`: OIDC identity for keyless signing and Pages deployment.
- `VT_API_KEY`: required operationally for non-PR binary scans.
- `CARGO_REGISTRY_TOKEN`: crates.io publishing only when enabled.
- `AUR_SSH_PRIVATE_KEY`: configured AUR publishing.
- `NEXUS_API_KEY` and `NEXUS_GROUP_IDS`: configured Nexus upload pair.
- `DW_BOT_PAT`: cross-repository dependent issue creation.
- Discord webhook secrets: notification only; missing webhook is an intentional skip.

See generated contracts for exact required/default declarations. No secret values belong
in source snapshots, plans, failure reports or browser data. The capture command reads
workflow source and explicitly selected evidence files, not credential stores.

## Incident response

Preserve the failing run URL, commit/tag, caller ref, job, target and artifact identity.
If publication occurred before a later failure, determine which channels already changed
before retrying. Use the [incident desk](@/stroggforge/troubleshooting.md) and the
[publishing recovery rules](@/releases/publishing.md).
