"""Build-only output contract: fail if Rust emits different bytes or file sets."""
import hashlib
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def generate():
    subprocess.run(
        ["cargo", "run", "--locked", "-p", "stroggforge-war-room", "--", "build"],
        cwd=ROOT,
        check=True,
    )
    return {
        str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
        for directory in ("site/content", "site/static/generated")
        for path in sorted((ROOT / directory).rglob("*"))
        if path.is_file()
    }


first = generate()
second = generate()
if first != second:
    changed = sorted(key for key in first.keys() | second.keys() if first.get(key) != second.get(key))
    raise SystemExit("Nondeterministic generation: " + ", ".join(changed))
print(f"Deterministic generation: {len(first)} files agree byte-for-byte")
