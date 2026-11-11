#!/usr/bin/env bash
# Name the target cache's partition: the runner image and the environment that changes what Cargo
# builds. runner.os and runner.arch alone put macos-15-intel and macos-26-intel in one cache, and a
# job that changes its profile (CARGO_PROFILE_DEV_DEBUG) would restore the old build on an exact key
# hit, rebuild it all, and never save, since actions/cache saves nothing on a hit.
#
# Writes `partition=<image>-<checksum>` to GITHUB_OUTPUT.
set -euo pipefail

# Hosted runners name their image in ImageOS (ubuntu22, macos26, win25). A container or a
# self-hosted runner names it from the system.
image=${ImageOS:-}
if [ -z "$image" ]; then
  if [ -r /etc/os-release ]; then
    # shellcheck disable=SC1091
    image=$(. /etc/os-release && echo "${ID:-linux}${VERSION_ID:-}")
  elif [ "$(uname -s)" = Darwin ]; then
    image="macos$(sw_vers -productVersion | cut -d. -f1)"
  else
    image=$(uname -s)
  fi
fi

# Profiles, flags, linkers and the build target, wherever they were set: workflow, job, or an
# earlier step's GITHUB_ENV (setup-llvm.sh sets CARGO_BUILD_TARGET on Windows). The target
# directory is where the cache lives, not what it holds.
build_env=$(env | grep -E '^(CARGO_(PROFILE|BUILD|TARGET)_[A-Z0-9_]+|RUSTFLAGS|RUSTDOCFLAGS|CARGO_ENCODED_RUSTFLAGS)=' \
  | grep -v '^CARGO_TARGET_DIR=' | LC_ALL=C sort || true)
checksum=$(printf '%s' "$build_env" | cksum | cut -d' ' -f1)

image=$(printf '%s' "$image" | tr -c 'A-Za-z0-9._-' '_')
echo "cache partition: $image-$checksum"
[ -z "$build_env" ] || printf '%s\n' "$build_env" | sed 's/^/  /'
echo "partition=$image-$checksum" >> "$GITHUB_OUTPUT"
