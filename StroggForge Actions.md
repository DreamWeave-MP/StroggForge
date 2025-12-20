[Home](Readme.md)  

# StroggForge Actions

This repository was originally born to provide GitHub Actions various apps in the org use.
That's still its purpose, of course, and this document exists to explain them to you.
Their usage will be described in sequence of the order in which your workflows should run them (generally).

## [./.github/workflows/createRelease.yml](./.github/workflows/createRelease.yml)

This is a reusable workflow originally developed to help ease shipping development builds of applications to users.
It will search for a release matching the current tag name, or if the job isn't running on a tag, then a release called `development`.

If the job is NOT running on a tag release, then, the `development` tag is also deleted from the target repository.
A new release with the target name is then created on the repository, with auto-generated changelog and a marker indicating which workflow created the release.

This workflow has one output, `release_name`, which can be used to determine whether we're on a tag release or not a *bit* more easily.
This workflow requires access to the GitHub token, so should be called with `secrets: inherit`

## [./.github/actions/corprus-crucible/action.yml](./.github/actions/corprus-crucible/action.yml)

Corprus Crucible is DreamWeave's core Rust build flow that ensures deliverability and verifiability of all releases.
The Crucible tests, builds, and signs an array of Rust apps provided to it, supporting multi-app repositories or monorepos.

The Crucible has no direct outputs, but has a range of required parameters:

1. `binary_name`: String param indicating the executable name to build, sans extension.
1. `include_files`: Optional comma-separated list of additional files to inclue. Paths are relative to the working directory, which is dependent upon whether the target `binary_name` matches a folder name in the root of the repository.
1. `vt_api_key`: A VirusTotal API key. DreamWeave stores is VT API key as an organization-level secret which should be passed into this workflow.
1. `github_token`: As it says on the tin. Not negotiable.
1. `release_name`: Provided to this composite action as an output from the createRelease workflow, or just make it up yourself.

## [./.github/workflows/discord.yml](./.github/workflows/discord.yml)

Action to emit pipeline results into the DreamWeave Discord channels.
Generally called `if: always()` to allow warning about pipeline failures.

Inputs:

1. `avatar_url`: Optional, usually not needed. By default uses the DreamWeave logo.
1. `title`: Optional title string for the embed. Autofills with the repo name if unspecified.
1. `description`: Optional text content for the embed. If unspecified contains a link to the latest release, using the ref name as its text.
1. `footer_text`: Optionally override the timestamp/workflow ids at the bottom of each log message.

Secrets:

1. `webhook_url`: Mandatory. Gives a channel URL to write to. By default, you should use the organization secrets containing this URL, unless you have a special use case.

## [./.github/workflows/dependent.yml](./.github/workflows/dependent.yml)

When one repository depends on another, use this action to notify your dependents so their maintainers don't lose track of things.
Super simple action that needs some more work, specifically in accepting the issue body as an input.

Inputs:

1. `aur_package_name`: Required, used to generate links to AUR packages.
1. `target_repo`: Well, you can't make an issue without saying where to do it. Run the workflow in a matrix to notify multiple repos.

Secrets:

1. `DW_BOT_PAT`: Typically should just use the Org-level PAT created specifically for this action. If you don't use this, you should have a very good reason for not doing so.

## [./.github/action_templates/rust_template.yaml](./.github/action_templates/rust_template.yaml)

This one actually is not meant to be used directly, but rather is a template for Rust repositories to use when creating new CI.
It uses all of the above actions/workflows to provide a standardized build flow for *all* our apps. There's really no reason *not* to use this, except that it needs `cargo publish` functionality.

To set up, you must fill in the following values, in each of the respective jobs and steps:

1. `release.steps.Run Corprus Crucible`: `ENTER_BINARY_NAME_HERE` should be replaced with the name of the app you're building. Add a matrix for multi-project repos.
1. `aur-publish.env.AUR_PACKAGE_NAME`: `ENTER_PACKAGE_NAME_HERE` should be replaced by the AUR package name, if there is one. If you're not posting to the AUR, delete this section, and shame on you.
1. `aur-publish.steps[1].with.git_username/git_email`: both `ENTER_USERNAME_HERE` and `ENTER_EMAIL_HERE` should be replaced by the respective values for publishing.
1. `nag-dependents.strategy.matrix.target_repo`: Replace the fake repository names here with real ones under the DreamWeave org.
