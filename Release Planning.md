## [Home](Readme.md)  

# How To Do Releases

Most applications within the DreamWeave ecosystem manage themselves.
However, no process is perfect or without steps to remember.
Please make sure to reference this document when creating releases to ensure we get it right - *every* time!

## Rust Apps

1. Pick a version number
2. Make sure the version number in Cargo.toml matches the one you wish to tag.
3. Run the following build command for all affected crates: `cargo test --all-targets --all-features -- --show-output && cargo build --release`
4. If *all* tests passed and the app built, it's okay to make a tag now.
5. Push the tag to the target repository. StroggForge will handle building, testing, signing, and packaging your release for all platforms.
6. Check whether the app you just updated is available on crates.io:
  - S3LightFixes
  - VFSTool
  - VFSTool_lib
7. If it IS available on crates.io, run a `cargo publish`

## AUR PKGBUILDs

Our workflow here could be better! The issue is that not all of DreamWeave's hosted PKGBUILDs are associated with repositories we control directly.

Apps which will update themselves upon the GitHub repository updating:
  - s3lightfixes-git
  - vfstool-git

Ones which require manual updates:
  - momw-tools-pack-git
  - delta-plugin-git
  - umo-git
  - groundcoverify-git

Preferably, it'd be nice if we could run a cron workflow in a repository once every week or so to track updates in the independent repositories.
Since they're -git repos, there's little to do, but if you find somehow that the PKGBUILDs are out of date, contact someone on [the AUR team](./AUR-TEAM.md)

## [DW-Tools](https://github.com/DreamWeave-MP/DW-tools)

Repositories which DW-Tools depends upon will automatically notify DW-Tools when it's necessary to update.
To update DW-Tools:

1. Grab the release link out of the created issue and update the build script with the listed `sha256` for each release variant.
1. Just copy the shas and remove the `sha256:` bit at the front.
1. Make sure you got the archive names right.

## [DreamWeave-High-End-Resources](https://github.com/DreamWeave-MP/dreamweave-high-end-resources)

Updating the resources repository can be tricky. It depends upon *both* OpenMW stable and dev, plus DW-Tools.
Additionally, wareya's shaders can potentially break between OpenMW versions.
***ALWAYS*** test the contents of this repository before updating it.

1. Make sure it actually works. That means at *least* the water and PBR shaders.
2. Grab the build sha and date of the latest OpenMW build available at: https://redfortune.de/openmw/nightly/?C=M;O=D
3. Write both the build sha and date into [get-dev-resources](https://github.com/DreamWeave-MP/dreamweave-high-end-resources/blob/main/get-dev-resources) in the variables `ref` and `date`
4. Check [get-stable-resources](https://github.com/DreamWeave-MP/dreamweave-high-end-resources/blob/main/get-stable-resources) and make sure the latest stable OpenMW version there is correct.

When the resources repository is updated, this will trigger a notification on [Sixth-House-Mod-Cache](https://github.com/DreamWeave-MP/Sixth-House-Mod-Cache).

## [Sixth-House-Mod-Cache](https://github.com/DreamWeave-MP/Sixth-House-Mod-Cache)

This is the main modlist repo at this time.
Probably, this will change later as DreamWeave-web matures and generates the lists for us.
For now, we manually track our config directories here.
This way works best for ensuring reliability of storage configurations, key bindings, etc.

The workflow for Sixth-House-Mod-Cache will do *most* of the work in getting the release out for you.
Of course, it's a modlist repo, so the only real way to know what's happening is to playtest the shit out of it.
Further documentation on debugging modlists and triaging issues to follow.
