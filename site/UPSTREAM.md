# Imported DreamWeave site components

Two pieces of this site come from other DreamWeave repositories. They are copied in, not
fetched at build time, so a clean checkout always has everything.

## Docs shell: DreamWeave-Mod-Template

Imported at `d86d2cb50fe6539132a9f6e1d37d13b9699e8a17` from
<https://github.com/DreamWeave-MP/DreamWeave-Mod-Template>, licensed **AGPL-3.0**. The full
license text ships as `site/static/LICENSE-AGPL-3.0.txt` and is linked from every page's
footer. Attribution stays with the Mod Template's contributors.

Files: `templates/docs/{base,breadcrumbs,page,section,sidebar,toc}.html`, `sass/docs.sass`,
`static/docs/docs.js`.

Local changes, all small:

- `docs/base.html`: engineering header and footer links, the circuit mark, the extra
  stylesheets, a semantic mobile sidebar, the skip-link target and an accessible search
  results container.
- `docs.js`: Escape uses the native `hidden` attribute to match the search container.
  Search itself lives in `war-room.js`.
- `templates/index.html` is a minimal host shell supplying the block contract the docs
  templates expect. It replaces the Mod Template's storefront, not its docs architecture.

The StroggForge look lives in `sass/war-room.sass`, `templates/war-room/` and the other
shortcodes. None of the Mod Template's initializer, storefront taxonomy or download
automation came along.

## Schematic shortcode: S3ctors-S3cret-St4sh

Imported at `86cbc26146de7daf30f5fbf57d791d847dd8ac9b` from
<https://github.com/DreamWeave-MP/S3ctors-S3cret-St4sh>.

- `templates/shortcodes/schematic.html`: unchanged.
- `sass/schematic.sass`: every `docs-schematic` rule from St4sh's `sass/docs.sass`,
  including its responsive `@media` blocks, and nothing else.

Schematic data is JSON in the shortcode's format. Hand-written ones live in
`site/data/schematics/`; the generator writes the supply-line ones to
`site/static/generated/schematics/`.

## Updating either

Diff the listed files against a newer upstream revision you have actually read, apply what
matters, reapply the local changes above, and update the revision here. Do not edit the
imported files to restyle them; override in `war-room.sass`, which loads last.
