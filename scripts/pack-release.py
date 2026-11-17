import json
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

root = Path(__file__).resolve().parents[1]
output = Path(sys.argv[1]).resolve()
wrapper = json.loads((root / "package.json").read_text())
version = wrapper["version"]
source = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
notice = (root / "THIRD_PARTY_NOTICES.md").read_text()
for library in ("libheif", "libde265"):
    pin = subprocess.check_output(
        ["git", "rev-parse", f"HEAD:deps/{library}"], cwd=root, text=True
    ).strip()
    if f"https://github.com/strukturag/{library}/tree/{pin}" not in notice:
        sys.exit(f"Update THIRD_PARTY_NOTICES.md for {library} at {pin}")

legal = {
    path: (root / path).read_bytes()
    for path in ("LICENSE", "LICENSES/GPL-3.0.txt", "LICENSES/LGPL-3.0.txt")
}
legal["THIRD_PARTY_NOTICES.md"] = (
    notice.rstrip() + f"\n\nRelease source: https://github.com/photon-hq/heif2jpeg/tree/{source}\n"
).encode()

native = sorted((root / "npm").glob("*/package.json"))
versions = {}
output.mkdir(parents=True, exist_ok=False)
for manifest in [*native, root / "package.json"]:
    pkg = json.loads(manifest.read_text())
    is_wrapper = manifest.parent == root
    if is_wrapper:
        if versions.keys() != pkg["optionalDependencies"].keys():
            sys.exit("Wrapper dependencies must match the native packages")
        pkg["optionalDependencies"] = versions
    else:
        if pkg["version"] != version or pkg["name"] in versions:
            sys.exit(f"Unexpected native package name or version: {manifest}")
        versions[pkg["name"]] = version
    destination = output / ("wrapper" if is_wrapper else "native")
    destination.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        stage = Path(tmp)
        for path in ["README.md", *pkg["files"]]:
            if path in legal or path == "LICENSES":
                continue
            shutil.copy2(manifest.parent / path, stage / path)
        for path, data in legal.items():
            target = stage / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        (stage / "package.json").write_text(json.dumps(pkg, indent=2) + "\n")
        packed = subprocess.check_output(
            ["npm", "pack", "--json", "--ignore-scripts", "--pack-destination", str(destination)],
            cwd=stage, text=True
        )
    filename = json.loads(packed)[0]["filename"]
    with tarfile.open(destination / filename) as archive:
        for path, data in legal.items():
            if archive.extractfile(f"package/{path}").read() != data:
                sys.exit(f"Invalid {path} in {filename}")
        if archive.getmember(f"package/{pkg['main']}").size == 0:
            sys.exit(f"Empty entry point in {filename}")
        if is_wrapper and json.load(archive.extractfile("package/package.json"))["optionalDependencies"] != versions:
            sys.exit(f"Incorrect native dependency versions in {filename}")
    print(f"Verified {filename}")
