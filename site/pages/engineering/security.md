+++
title = "Supply-chain security"
description = "What gets signed, which credentials exist, and what the checks actually prove."
weight = 25
+++

## What the Checks Prove

Tests, formatting, pedantic Clippy and RustSec gate every standard release, and MSRV
joins them when configured. None of that makes a binary trustworthy. It makes it
reviewable: every artifact traces back to a run, a commit and a set of checks somebody
can inspect.

Corprus Crucible signs the **binary** with keyless Cosign before it is packaged, and the
verification bundle travels inside the archive. Verify the extracted binary against that
bundle and the expected workflow identity. The archive itself is not signed, so do not
tell anyone it is. VirusTotal then scans the archive and records the links. A clean scan
means no scanner complained. It does not mean the binary is benign.

## Credentials

- `GITHUB_TOKEN`: built in, limited by the caller's and the job's permissions.
- `id-token: write`: the OIDC identity keyless signing and Pages deployment rely on.
- `VT_API_KEY`: needed for every non-PR binary release.
- `CARGO_REGISTRY_TOKEN`: crates.io, only when publishing is on.
- `AUR_SSH_PRIVATE_KEY`: AUR publishing.
- `NEXUS_API_KEY` and `NEXUS_GROUP_IDS`: Nexus uploads, always as a pair.
- `DW_BOT_PAT`: opening update issues in dependent repositories.
- Discord webhooks: notifications only. A missing webhook skips quietly, by design.

The [input reference](@/stroggforge/inputs.md) shows which contract declares what. No
secret value belongs anywhere under `war-room/` or in a plan; this whole site is public.

## Workflow Scripts

A `run:` step receives caller-supplied text, like a notification title, through `env:`,
never spliced into the script. `${{ inputs.title }}` pasted into shell source means a
title containing `$(...)` gets executed. `discord.yml` shows the pattern.

## When Something Goes Wrong

Keep the run URL, commit or tag, caller ref, job, target and artifact identity. If some
channels already published before the failure, find out which before retrying. Then the
[incident desk](@/stroggforge/troubleshooting.md) and the
[recovery rules](@/releases/publishing.md).
