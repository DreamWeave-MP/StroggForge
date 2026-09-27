#!/usr/bin/env python3
"""Publish every publishable Cargo workspace member to crates.io, dependencies first.

`cargo publish --workspace` orders crates too, but it cannot be rerun once part of the workspace is
on crates.io, and crates.io only lets an account publish a handful of *new* crates before it answers
429 for a while. This walks the same order one crate at a time instead:

- members with `publish = false` (or a `publish` list without `crates-io`) are left out;
- a crate whose version is already in the crates.io index is skipped, so a failed run can be rerun;
- a 429 waits out the window crates.io names (or ten minutes) and tries that crate again;
- after each upload it waits for the crate to show up in the index before the next crate builds
  against it (`cargo publish` gives up waiting after a minute and only warns).

Run it from the workspace root with CARGO_REGISTRY_TOKEN set.

Usage: publish_workspace.py [--list | --dry-run] [--metadata FILE]

  --list           print the publish order, then exit (no network access)
  --dry-run        walk the order and report what would be uploaded, without uploading
  --metadata FILE  read `cargo metadata --no-deps --format-version 1` output from FILE
"""

import argparse
import email.utils
import json
import re
import subprocess
import sys
import time
import urllib.error
import urllib.request

USER_AGENT = "StroggForge workspace publish (https://github.com/DreamWeave-MP/StroggForge)"
RATE_LIMIT_RETRIES = 30
DEFAULT_RATE_LIMIT_WAIT = 600
INDEX_WAIT_TIMEOUT = 900
INDEX_POLL_INTERVAL = 10
INDEX_REQUEST_RETRIES = 5


def load_metadata(path):
    if path:
        with open(path, encoding="utf-8") as handle:
            return json.load(handle)
    return json.loads(
        subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )


def publishable(package):
    # `publish = false` shows up as an empty list; `publish = ["other"]` names the allowed registries.
    registries = package.get("publish")
    return registries is None or "crates-io" in registries


def blocking_dependency(dep):
    # Cargo strips path-only dev-dependencies (no version, so `req` is `*`) when it packages a crate,
    # so they never have to be on crates.io first. Every other dependency kind does.
    return not (dep.get("kind") == "dev" and dep.get("req") == "*")


def workspace_order(metadata):
    members = set(metadata["workspace_members"])
    packages = {
        package["name"]: package
        for package in metadata["packages"]
        if package["id"] in members and publishable(package)
    }
    deps = {
        name: sorted(
            {
                dep["name"]
                for dep in package["dependencies"]
                if dep["name"] in packages and dep["name"] != name and blocking_dependency(dep)
            }
        )
        for name, package in packages.items()
    }

    order, done, visiting = [], set(), []

    def visit(name):
        if name in done:
            return
        if name in visiting:
            cycle = " -> ".join(visiting[visiting.index(name) :] + [name])
            sys.exit(f"::error::workspace dependency cycle: {cycle}")
        visiting.append(name)
        for dep in deps[name]:
            visit(dep)
        visiting.pop()
        done.add(name)
        order.append(packages[name])

    for name in sorted(packages):
        visit(name)
    return order


def index_path(name):
    name = name.lower()
    if len(name) <= 2:
        return f"{len(name)}/{name}"
    if len(name) == 3:
        return f"3/{name[0]}/{name}"
    return f"{name[:2]}/{name[2:4]}/{name}"


def is_published(name, version):
    request = urllib.request.Request(
        f"https://index.crates.io/{index_path(name)}",
        headers={"User-Agent": USER_AGENT, "Cache-Control": "no-cache"},
    )
    for attempt in range(1, INDEX_REQUEST_RETRIES + 1):
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                lines = response.read().decode().splitlines()
            return any(json.loads(line)["vers"] == version for line in lines if line.strip())
        except urllib.error.HTTPError as error:
            if error.code == 404:
                return False
            if attempt == INDEX_REQUEST_RETRIES or error.code < 500:
                raise
        except (urllib.error.URLError, TimeoutError):
            if attempt == INDEX_REQUEST_RETRIES:
                raise
        time.sleep(5 * attempt)
    return False


def wait_for_index(name, version):
    deadline = time.time() + INDEX_WAIT_TIMEOUT
    while not is_published(name, version):
        if time.time() > deadline:
            sys.exit(f"::error::{name} {version} never showed up in the crates.io index")
        print(f"Waiting for {name} {version} to appear in the crates.io index...", flush=True)
        time.sleep(INDEX_POLL_INTERVAL)


def rate_limit_wait(output):
    match = re.search(r"try again after (.+?GMT)", output)
    if match:
        try:
            retry_at = email.utils.parsedate_to_datetime(match.group(1)).timestamp()
            return max(int(retry_at - time.time()) + 15, 30)
        except (TypeError, ValueError):
            pass
    return DEFAULT_RATE_LIMIT_WAIT


def already_uploaded(output):
    return "already exists" in output or "already uploaded" in output


def rate_limited(output):
    return "429" in output or "Too Many Requests" in output


def publish(package):
    name, version = package["name"], package["version"]
    for attempt in range(1, RATE_LIMIT_RETRIES + 1):
        print(f"::group::cargo publish -p {name} ({version}), attempt {attempt}", flush=True)
        result = subprocess.run(
            ["cargo", "publish", "-p", name], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True
        )
        print(result.stdout, flush=True)
        print("::endgroup::", flush=True)
        if result.returncode == 0:
            wait_for_index(name, version)
            return
        if already_uploaded(result.stdout):
            print(f"{name} {version} is already on crates.io", flush=True)
            return
        if rate_limited(result.stdout):
            wait = rate_limit_wait(result.stdout)
            print(f"::notice::crates.io rate limited {name}; retrying in {wait}s", flush=True)
            time.sleep(wait)
            continue
        sys.exit(f"::error::cargo publish -p {name} failed")
    sys.exit(f"::error::{name} was still rate limited after {RATE_LIMIT_RETRIES} attempts")


def main():
    parser = argparse.ArgumentParser(description="Publish a Cargo workspace to crates.io in dependency order.")
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--list", action="store_true", help="print the publish order and exit")
    mode.add_argument("--dry-run", action="store_true", help="report what would be uploaded without uploading")
    parser.add_argument("--metadata", help="read cargo metadata JSON from this file instead of running cargo")
    args = parser.parse_args()

    order = workspace_order(load_metadata(args.metadata))
    if not order:
        print("::notice::No publishable workspace members found; nothing to publish", flush=True)
        return

    if args.list:
        for package in order:
            print(f"{package['name']} {package['version']}", flush=True)
        return

    pending = 0
    for package in order:
        name, version = package["name"], package["version"]
        if is_published(name, version):
            print(f"{name} {version} is already on crates.io; skipping", flush=True)
            continue
        pending += 1
        if args.dry_run:
            print(f"would publish {name} {version}", flush=True)
            continue
        publish(package)

    if args.dry_run:
        print(f"{pending} of {len(order)} crates would be published", flush=True)


if __name__ == "__main__":
    main()
