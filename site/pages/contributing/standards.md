+++
title = "Engineering standards"
description = "Small explicit changes, fail-fast code and honest status."
weight = 30
+++

## Rust

Explicit types, enums for closed sets, full-word names, ordinary control flow and errors
that say what to fix. No framework for a problem that has one implementation. If an
invariant matters, fail when it breaks; a generator that quietly renders bad data is not
being tolerant, it is lying to you.

Run the generator's formatting, tests and pedantic Clippy before changing it. Validation
covers structure: IDs, references, plan states and cycles. Taste belongs in review.

## Relationships and Status

Add a relationship when it is real: a Cargo dependency, a workflow call, a content path,
a migration. Use `unknown` when nothing is recorded. An empty table cell is more useful
than a manufactured green one.

## Workflows

Shell logic lives in `scripts/`; composite actions call it. Keep release-boundary
ordering and the Discord `needs` list complete. A new input documents itself on the site;
update `war-room/workflows.toml` when a workflow's behavior changes. Check every
self-referential ref before tagging a StroggForge release.

## Writing

Write in `site/pages`, never the generated `site/content`. Use stable slugs and Zola's `@/`
links so the build catches breakage. Every diagram needs a table or its source next to it.
Check narrow screens, keyboard navigation and reading with JavaScript off.

## Release Intent

Record a version or date only when someone decided on it. Link the run or artifact that
finished a requirement, and give a blocked requirement a reason somebody can act on.
Configured, passing CI, inspected artifact and tested on hardware are four different
facts. Keep them apart.
