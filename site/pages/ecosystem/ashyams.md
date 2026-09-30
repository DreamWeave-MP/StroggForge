+++
title = "AsHyAmS, the index"
description = "Where DreamWeave's projects are found: what AsHyAmS reads, what it publishes, how a release reaches it, and how to keep it running."
weight = 1
+++

Everything this war room builds ends up on a site. [AsHyAmS](https://dreamweave-mp.github.io/AsHyAmS/)
is where somebody finds it.

The DreamWeave network is not a service. It is every site that publishes the DreamWeave protocol:
a `dreamweave.json` and a manifest per project, written by the
[Mod Template](@/ecosystem/projects/mod-template.md) on every build. Nothing sits above those sites.
AsHyAmS is one index of them. It reads every site it has been told about, keeps what each said in
Git, notices what changed, and publishes all of it as static files. Anyone can run another index
over the same sites; this is ours, and it is the one people are pointed at.

The distinction is the design. The sites own the facts. AsHyAmS owns observations of them. If it
disappears, every mod stays discoverable, installable and verifiable from its own site.

## What it shows

- **Projects**, one card per claim: a site's manifest for one project id, labelled with the site
  that published it. Two sites claiming the same id are both shown, and neither wins.
- **Releases** with every artifact and its SHA-256, labelled as the publisher's claim. Programs
  show one archive per platform, desktop, Android and handheld alike. Crates show the checksum
  crates.io serves. Game data shows its components and install layout.
- **Dependencies and capabilities**, resolved release by release, and the gaps where a
  relationship points outside the network.
- **Updates**: a typed diff of every change between one crawl and the next, and an Atom feed of it.
- **Health**: which sites were read on the last crawl, which failed and why, and which claims are
  served from the last good copy.

Nothing is counted. There are no downloads, views or stars to rank by, because AsHyAmS never sits
in the path of a download and has no telemetry to invent numbers from.

## Machine-readable

| File | What |
|---|---|
| [`network-data/catalog.json`](https://dreamweave-mp.github.io/AsHyAmS/network-data/catalog.json) | Every site, claim, relationship and capability, in the DreamWeave Network Catalog v1 format with its own JSON Schema |
| [`network-data/events.json`](https://dreamweave-mp.github.io/AsHyAmS/network-data/events.json) | Every change AsHyAmS has observed |
| [`network-data/updates.xml`](https://dreamweave-mp.github.io/AsHyAmS/network-data/updates.xml) | The same changes as an Atom feed |

A client can read these instead of crawling every site itself. It should still check an archive
against the publisher's own manifest before installing it: the catalog is a cache, not an
authority.

## How a release gets there

{{ schematic(data_path="data/schematics/ashyams-release-path.json") }}

A site does nothing to be re-read. AsHyAmS compares each manifest's digest in `dreamweave.json` and
downloads only the ones that changed. Expect a release on AsHyAmS within six hours of its site
deploying. A site that is down for a crawl keeps its projects listed from the last good read,
marked stale, until it comes back.

## Getting a site indexed

A site is indexed once it is enrolled in `network/sources.toml`, by pull request. From an AsHyAmS
checkout:

```sh
cargo network inspect https://dreamweave-mp.github.io/your-project/
cargo network add https://dreamweave-mp.github.io/your-project/ --note "What it is, in one line."
```

`inspect` reads the site exactly as the crawler will and writes nothing. Fix whatever it reports
before `add`: a project it cannot validate is indexed as invalid until it can. CI inspects every
newly enrolled source on the pull request, so the reviewer sees the same report. The
[join page](https://dreamweave-mp.github.io/AsHyAmS/join/) covers the rest, including the issue
form for people without a Rust toolchain.

Every Mod Template site this war room deploys should be enrolled. A site that is published but not
enrolled exists, and nobody finds it.

## Keeping it running

AsHyAmS has no server to keep up. Two workflows do everything:

| Workflow | Runs | Does |
|---|---|---|
| `network.yml` (Refresh AsHyAmS) | every six hours, every push to `main`, on demand | crawl every source, check the state, build and deploy the site, commit the observations to `network-state` |
| `check.yml` | every pull request | Rust gates, a fixture network built and link-checked, and a live `inspect` of every source the pull request adds |

The observations live on the `network-state` branch; the code, the source list and the manual live
on `main`. Losing `network-state` loses history, not the network: delete it and the next refresh
rebuilds every current claim from the sites. The
[operating manual](https://dreamweave-mp.github.io/AsHyAmS/about/operating/) has the runbook,
including how to read a failed crawl.

### When the protocol changes, AsHyAmS changes too

This is the part that bites. AsHyAmS validates every manifest against the Mod Template's JSON
Schemas, vendored into `tools/dreamweave-network/schemas/`, and its Rust model refuses unknown
fields because the protocol does. When StroggForge or the template adds something a manifest can
say, AsHyAmS does not know it yet, and every site that says it is indexed as invalid.

That already happened once. `android` builds, the `portmaster` and `muos` platform variants, and
`crate` artifacts reached the template's schema before AsHyAmS's copy, and greenmote, dream-net and
every crate site showed up invalid until the schema was re-vendored and the model taught the new
fields.

So a change to what `modGlobalBuild` records, or to the template's manifest schema, is not done
until AsHyAmS reads it:

1. Copy the changed schema from the Mod Template into `tools/dreamweave-network/schemas/` and
   record the revision in its `UPSTREAM.md`.
2. Teach `src/protocol/` any new field or enum value, with a test.
3. Run `cargo network inspect` against a real site that uses it. It must say *Ready*.

Presentation follows the template too: AsHyAmS vendors its tokens, docs shell and schematic, as
this war room does, and its `UPSTREAM.md` records the revision.
