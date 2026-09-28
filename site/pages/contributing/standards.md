+++
title = "Engineering standards"
description = "Small explicit changes, real evidence, deterministic outputs."
weight = 30
+++

## Rust and metadata

Use explicit types and enums, useful error context, full-word names and ordinary control
flow. Avoid generic frameworks and silent omission. Add a relationship only when a
manifest, workflow, content path or migration record supports it. Use unknown for missing
evidence; never manufacture readiness to fill a table.

Run the generator crate's formatting, tests and strict Clippy before changing its contract.
Meaningful validation tests exercise broken references, contradictory readiness, cycles,
capture semantics and deterministic generation. No network or sibling clone is required
for ordinary validation.

## Workflow changes

Keep shell mechanics in `scripts/`; composite actions orchestrate them. Keep release
boundary dependencies and notification `needs` complete. Updating a workflow's input
automatically updates its generated contract; update operational notes when semantics change.
Review self-referential refs intentionally before any StroggForge release tag.

## Documentation

Write in `site/pages`, not generated `site/content`. Keep the shared docs foundation
recognizable and record upstream changes in `site/UPSTREAM.md`. Use stable human-readable
slugs and Zola `@/` links. Every diagram needs a textual equivalent or readable source.
Verify narrow-screen layout, keyboard navigation and JavaScript-disabled reading.

## Release intent

Record a version/date only when there is an actual plan. A done requirement needs evidence;
a blocked requirement needs a useful reason. Configuration completion, successful CI,
artifact inspection and runtime testing are distinct facts.
