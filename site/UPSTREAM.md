# Imported DreamWeave site components

Two pieces of this site come from other DreamWeave repositories. They are copied in, not
fetched at build time, so a clean checkout always has everything.

## Docs shell: DreamWeave-Mod-Template

Imported at `208857655d07903844fe55335da13396b39024de` (branch `V5`) from
<https://github.com/DreamWeave-MP/DreamWeave-Mod-Template>, licensed **AGPL-3.0**. The full
license text ships as `site/static/LICENSE-AGPL-3.0.txt` and is linked from every page's
footer. Attribution stays with the Mod Template's contributors.

Files: `templates/docs/{base,breadcrumbs,page,section,sidebar,toc}.html`, `sass/docs.sass`,
`static/docs/docs.js`. Only `docs/base.html` differs from upstream.

`docs.sass` and `docs.js` work as a pair. Three columns from 1200px; below that the page's
contents fold into a drawer, and below 768px the navigation does too, the two buttons making
one bar across the screen. `docs.js` closes the drawers and marks the shell
`.docs-shell--ready`; without it both panels stay open in the page's flow. The shell reads
`--dw-*` tokens, falling back to `--accent`, `--border-color` and friends; `war-room.sass`
defines both on `.war-room`. The manual switcher shows only on a site with more than one docs
root, so it never appears here.

Local changes, all in `docs/base.html`:

- It extends `index.html`, a minimal host shell supplying the block contract the docs
  templates expect, instead of the template's `base.html`. It replaces the Mod Template's
  storefront, not its docs architecture.
- Its own header: the circuit mark, the engineering links, and a search input and results
  container that `war-room.js` drives (`aria-controls`, a live status line, `hidden`). The
  template's header and its `search_scope` block live in its `base.html`, which this site
  does not use.
- The extra stylesheets, `schematic.css` and `war-room.css`.
- `<main id="main" tabindex="-1">` on the page column, so the skip link lands past the
  navigation. Upstream puts `<main>` in its `base.html`, around the whole shell.
- No comments include: the war room has no discussions.
- Its own footer links, including the license.

Dropped at `20885765`: the semantic mobile sidebar, because upstream now wraps the navigation
in the same `details`/`summary`/`nav` panel and makes it the phone drawer (labelled with the
docs root's `docs_project_name`), and the `docs.js` Escape change, because upstream `docs.js`
no longer touches search. Code copying and the `/` shortcut left `docs.js` too; `war-room.js`
does both, and closes the search results on Escape.

The StroggForge look lives in `sass/war-room.sass`, `templates/war-room/` and the other
shortcodes. `war-room.sass` also styles what the shell stopped carrying: the header, the
search, code blocks and their copy buttons. None of the Mod Template's initializer, storefront
taxonomy or download automation came along.

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

Its rules sit under `.war-room`, so they outrank the shell's own media queries. Restyle the
shell there, but leave its columns, breakpoints, positions and panel margins to `docs.sass`,
or the drawers break. Check a page at phone and tablet widths after an update: open both
drawers, and search while they are in view.
