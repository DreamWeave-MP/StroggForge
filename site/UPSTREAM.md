# DreamWeave docs foundation

Imported from the local **DreamWeave-Mod-Template** checkout at
`d86d2cb50fe6539132a9f6e1d37d13b9699e8a17` (2026-09-28 inspection).
Repository: <https://github.com/DreamWeave-MP/DreamWeave-Mod-Template>.
The upstream code is licensed **AGPL-3.0**. Its complete, verbatim license is included as
the `evidence.LICENSE` field of `war-room/sources/mod-template.json`, and emitted as
`site/static/generated/LICENSE-AGPL-3.0.txt` for readers of the published site.
Attribution remains with the DreamWeave Mod Template contributors.

The import consists of `templates/docs/{base,breadcrumbs,page,section,sidebar,toc}.html`,
`sass/docs.sass`, and `static/docs/docs.js`. St4sh's use of the same docs architecture
was inspected at `86cbc26146de7daf30f5fbf57d791d847dd8ac9b`, including Cod3x's docs-root frontmatter.

## Local delta

- `docs/base.html`: engineering header/footer links, circuit mark, scoped skin link,
  semantic mobile sidebar panel, skip-link target, accessible search result container.
- The rest of the docs templates and `docs.sass` preserve the upstream implementation.
- `docs.js`: upstream copy/TOC/navigation behavior; Escape uses the native hidden attribute
  to match the local search container. Local search lives in `war-room.js`.
- `templates/index.html` is a minimal host shell supplying the upstream block contract;
  it replaces the storefront/Terminimal host, not the docs architecture.
- `sass/war-room.scss`, `templates/war-room/` and shortcodes own the tooling variant.
  No Mod Template initializer, storefront taxonomy or download automation is imported.

## Updating

Compare these paths against an explicitly reviewed upstream revision. Apply the upstream
changes, reapply the small base-template delta above, update this receipt and run the site
gate. Do not overwrite the tooling skin or hand-authored `site/pages`. The upstream
repository is not fetched during rendering; a clean checkout has the full foundation.
