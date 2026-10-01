#!/usr/bin/env bash
# setup-llvm.sh — install clang and lld whose LLVM major matches the given Rust toolchain's LLVM,
# so crates built under cross-language LTO (l3i and its dependents) link. Prints the resolved
# versions and writes `llvm_major`/`llvm_version` to $GITHUB_OUTPUT when set.
#
#   setup-llvm.sh [rust-toolchain] [llvm-version-override]
#
# rust-toolchain: a rustup toolchain name ("1.88", "stable"); empty uses the default `rustc`.
# llvm-version-override: a full LLVM release such as 22.1.8, instead of the one rustc reports.
set -euo pipefail

toolchain=${1:-}
override=${2:-}

rustc_cmd=(rustc)
if [[ -n "$toolchain" ]]; then
  rustc_cmd=(rustc "+$toolchain")
fi
rustc_llvm=$("${rustc_cmd[@]}" -vV | sed -n 's/^LLVM version: \([0-9][0-9.]*\).*/\1/p')
if [[ -z "$rustc_llvm" ]]; then
  echo "setup-llvm: could not read the LLVM version from '${rustc_cmd[*]} -vV'" >&2
  exit 1
fi
llvm_version=${override:-$rustc_llvm}
llvm_major=${llvm_version%%.*}
echo "setup-llvm: rustc (${toolchain:-default}) uses LLVM $rustc_llvm; installing LLVM $llvm_version (major $llvm_major)"

add_path() {
  if [[ -n "${GITHUB_PATH:-}" ]]; then
    echo "$1" >> "$GITHUB_PATH"
  fi
  export PATH="$1:$PATH"
}

# The GitHub release asset for this exact version, or the newest release of the same major.
release_tag() {
  local candidate="llvmorg-$llvm_version"
  if curl -fsSLI "https://github.com/llvm/llvm-project/releases/tag/$candidate" >/dev/null 2>&1; then
    echo "$candidate"
    return
  fi
  echo "setup-llvm: no LLVM release $llvm_version; looking for the newest $llvm_major.x release" >&2
  local newest
  newest=$(curl -fsSL "https://api.github.com/repos/llvm/llvm-project/releases?per_page=100" \
    | sed -n 's/.*"tag_name": *"\(llvmorg-'"$llvm_major"'\.[0-9]*\.[0-9]*\)".*/\1/p' | sort -V | tail -1)
  if [[ -z "$newest" ]]; then
    echo "setup-llvm: no LLVM $llvm_major.x release found" >&2
    exit 1
  fi
  echo "$newest"
}

download_release_asset() {
  local tag=$1 asset=$2 dest=$3
  curl -fSL --retry 3 -o "$dest" "https://github.com/llvm/llvm-project/releases/download/$tag/$asset"
}

install_linux() {
  if command -v apt-get >/dev/null 2>&1; then
    # apt.llvm.org carries every recent major for Ubuntu LTS releases.
    local script
    script=$(mktemp)
    curl -fsSL https://apt.llvm.org/llvm.sh -o "$script"
    sudo bash "$script" "$llvm_major"
    sudo apt-get install -y "lld-$llvm_major" "clang-$llvm_major"
    local tool
    for tool in clang clang++ ld.lld lld lld-link llvm-ar llvm-nm llvm-objcopy; do
      if [[ -x "/usr/bin/$tool-$llvm_major" ]]; then
        sudo ln -sf "/usr/bin/$tool-$llvm_major" "/usr/local/bin/$tool"
      fi
    done
    return
  fi
  if command -v dnf >/dev/null 2>&1; then
    # The EL9 builder image (CentOS Stream 9) ships clang and lld from AppStream; they are
    # installed in the image, and this makes a plain EL9 container work too. The major check at
    # the end catches the stream and rustc moving to different LLVM releases.
    dnf install -y clang lld
    return
  fi
  # Neither apt nor dnf: the official release tarball, which needs glibc 2.34 and GCC 12's
  # libstdc++ (Ubuntu 22.04, Debian 12; no EL release qualifies).
  local tag prefix
  tag=$(release_tag)
  prefix="${RUNNER_TEMP:-/tmp}/llvm-$llvm_major"
  mkdir -p "$prefix"
  download_release_asset "$tag" "LLVM-${tag#llvmorg-}-Linux-X64.tar.xz" "$prefix.tar.xz"
  tar -xJf "$prefix.tar.xz" -C "$prefix" --strip-components=1
  add_path "$prefix/bin"
}

install_macos() {
  # Neither LLVM build below is Apple's clang, so point it at the Xcode SDK for the rest of the
  # job; without SDKROOT it finds no system headers or libraries.
  local sdk
  sdk=$(xcrun --show-sdk-path)
  if [[ -n "${GITHUB_ENV:-}" ]]; then
    echo "SDKROOT=$sdk" >> "$GITHUB_ENV"
  fi
  export SDKROOT=$sdk

  local prefix="${RUNNER_TEMP:-/tmp}/llvm-$llvm_major"
  mkdir -p "$prefix"
  case "$(uname -m)" in
    arm64)
      # LLVM's own release, the same build every time and current the day it ships; Homebrew's
      # bottles lag it.
      local tag
      tag=$(release_tag)
      download_release_asset "$tag" "LLVM-${tag#llvmorg-}-macOS-ARM64.tar.xz" "$prefix.tar.xz"
      tar -xJf "$prefix.tar.xz" -C "$prefix" --strip-components=1
      ;;
    *)
      # LLVM publishes no Intel macOS build and Homebrew no longer bottles current LLVM for Intel,
      # so Intel takes conda-forge's clang, clang++ and lld, through a standalone micromamba,
      # pinned to rustc's LLVM major.
      local mamba="${RUNNER_TEMP:-/tmp}/micromamba"
      mkdir -p "$mamba"
      curl -fsSL --retry 3 https://micro.mamba.pm/api/micromamba/osx-64/latest | tar -xj -C "$mamba" bin/micromamba
      "$mamba/bin/micromamba" create --yes --quiet --root-prefix "$mamba/root" --prefix "$prefix" \
        --channel conda-forge --override-channels \
        "clang=$llvm_major" "clangxx=$llvm_major" "lld=$llvm_major"
      ;;
  esac
  add_path "$prefix/bin"
}

install_windows() {
  local tag prefix arch
  tag=$(release_tag)
  case "$(uname -m)" in
    aarch64|arm64) arch=aarch64 ;;
    *) arch=x86_64 ;;
  esac
  prefix="${RUNNER_TEMP:-/tmp}/llvm-$llvm_major"
  mkdir -p "$prefix"
  download_release_asset "$tag" "clang+llvm-${tag#llvmorg-}-$arch-pc-windows-msvc.tar.xz" "$prefix.tar.xz"
  7z x "$prefix.tar.xz" -so | 7z x -si -ttar -o"$prefix" >/dev/null
  local inner
  inner=$(find "$prefix" -maxdepth 1 -mindepth 1 -type d | head -1)
  add_path "$(cygpath -w "$inner/bin" 2>/dev/null || echo "$inner/bin")"
}

# On Windows, pin Cargo to an explicit --target (the host triple) for the rest of the job. Without
# one, Cargo applies the [target] rustflags to build scripts and proc macros too, and rustc refuses
# -Clinker-plugin-lto on proc macros for MSVC targets, because Cargo builds them with
# -Cprefer-dynamic. With an explicit target the rustflags only reach the target's own artifacts.
pin_windows_target() {
  local host
  host=$("${rustc_cmd[@]}" -vV | sed -n 's/^host: //p')
  if [[ -n "${GITHUB_ENV:-}" ]]; then
    echo "CARGO_BUILD_TARGET=$host" >> "$GITHUB_ENV"
  fi
  echo "setup-llvm: CARGO_BUILD_TARGET=$host"
}

case "${RUNNER_OS:-$(uname -s)}" in
  Linux) install_linux ;;
  macOS|Darwin) install_macos ;;
  Windows|MINGW*|MSYS*|CYGWIN*) install_windows; pin_windows_target ;;
  *)
    echo "setup-llvm: unsupported runner OS '${RUNNER_OS:-$(uname -s)}'" >&2
    exit 1
    ;;
esac

installed=$(clang --version | sed -n 's/.*clang version \([0-9][0-9.]*\).*/\1/p' | head -1)
installed_major=${installed%%.*}
if [[ "$installed_major" != "$llvm_major" ]]; then
  echo "setup-llvm: clang on PATH is $installed, not LLVM $llvm_major (rustc's); PATH order is wrong" >&2
  exit 1
fi
echo "setup-llvm: clang $installed at $(command -v clang)"
if command -v ld.lld >/dev/null 2>&1; then
  lld_banner=$(ld.lld --version | head -1)
elif command -v lld-link >/dev/null 2>&1; then
  lld_banner=$(lld-link --version | head -1)
else
  echo "setup-llvm: lld is not on PATH" >&2
  exit 1
fi
lld_version=$(sed -n 's/.*LLD \([0-9][0-9.]*\).*/\1/p' <<<"$lld_banner" | head -1)
if [[ "${lld_version%%.*}" != "$llvm_major" ]]; then
  echo "setup-llvm: lld on PATH is '$lld_banner', not LLVM $llvm_major (rustc's); PATH order is wrong" >&2
  exit 1
fi
echo "setup-llvm: $lld_banner"
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  {
    echo "llvm_major=$llvm_major"
    echo "llvm_version=$installed"
  } >> "$GITHUB_OUTPUT"
fi
