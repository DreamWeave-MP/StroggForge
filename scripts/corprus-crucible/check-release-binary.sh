#!/usr/bin/env bash
# check-release-binary.sh — refuse a release binary that needs the machine that built it: one that
# loads a library from the build's toolchain, or searches a folder only the runner had. It runs on
# the runner, where those folders exist, so it starts there and nowhere else; jess 0.1.0's Intel
# macOS release loaded @rpath/libc++.1.dylib from /Users/runner/work/_temp/llvm-23 and shipped.
#
#   check-release-binary.sh <binary> <platform-os>
#
# macOS: every library is the system's (/usr/lib, /System/Library) and there is no LC_RPATH.
# Linux, PortMaster and Android: no RPATH or RUNPATH outside $ORIGIN. Windows names the DLLs it
# loads without a folder, so there is nothing to check.
set -euo pipefail

binary=${1:?usage: check-release-binary.sh <binary> <platform-os>}
platform_os=${2:?usage: check-release-binary.sh <binary> <platform-os>}

if [ ! -f "$binary" ]; then
  echo "::error::Release binary not found: $binary"
  exit 1
fi

refused=0
refuse() {
  echo "::error::$binary $1"
  refused=1
}

# A tool by its own name, else LLVM's port of it; neither is an error, never a pass.
tool() {
  local name
  for name in "$1" "llvm-$1"; do
    if command -v "$name" >/dev/null 2>&1; then
      echo "$name"
      return
    fi
  done
  echo "::error::Neither $1 nor llvm-$1 is on PATH" >&2
  return 1
}

case "$platform_os" in
  macOS)
    # otool reports a file that isn't Mach-O and exits as if it had checked it.
    magic=$(od -An -tx1 -N4 "$binary" | tr -d ' \n')
    case "$magic" in
      cffaedfe | cefaedfe | cafebabe | cafebabf) ;;
      *)
        refuse "is not a Mach-O executable (it starts ${magic:-empty})"
        exit 1
        ;;
    esac
    otool=$(tool otool)
    libraries=$("$otool" -L "$binary")
    commands=$("$otool" -l "$binary")
    # The first line names the file; each after it is a library and its versions.
    while read -r library _; do
      case "$library" in
        /usr/lib/* | /System/Library/*) ;;
        *) refuse "loads $library, which only the build machine has; link the SDK's library instead" ;;
      esac
    done < <(tail -n +2 <<<"$libraries")
    while read -r path; do
      refuse "searches $path for libraries (LC_RPATH), a folder only the build machine has"
    done < <(awk '$2 == "LC_RPATH" { rpath = 1 } rpath && $1 == "path" { print $2; rpath = 0 }' <<<"$commands")
    ;;
  Windows) ;;
  *)
    readelf=$(tool readelf)
    dynamic=$("$readelf" -d "$binary")
    while read -r path; do
      # $ORIGIN is the binary's own folder, written literally, for the loader to expand.
      # shellcheck disable=SC2016
      case "$path" in
        '' | '$ORIGIN'*) ;;
        *) refuse "searches $path for libraries (RPATH/RUNPATH), a folder only the build machine has" ;;
      esac
    done < <(sed -n 's/.*(R\(UN\)\{0,1\}PATH).*\[\(.*\)\]/\2/p' <<<"$dynamic" | tr ':' '\n')
    ;;
esac

if [ "$refused" -ne 0 ]; then
  exit 1
fi
echo "check-release-binary: $binary needs nothing of the machine that built it"
