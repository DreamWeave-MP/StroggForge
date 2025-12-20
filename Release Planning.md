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
