#!/usr/bin/env bash
# Idempotently fetch local ruff + pytest into .tools/ when missing (A03).
# Prefer system installs when already on PATH.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS="${ROOT}/.tools"
BIN="${TOOLS}/bin"
PY="${TOOLS}/py"

mkdir -p "${BIN}" "${PY}"

need_ruff=0
if ! command -v ruff >/dev/null 2>&1 && [[ ! -x "${BIN}/ruff" ]]; then
  need_ruff=1
fi

need_pytest=0
if ! command -v pytest >/dev/null 2>&1 && ! python3 -m pytest --version >/dev/null 2>&1; then
  if [[ ! -d "${PY}/pytest" ]]; then
    need_pytest=1
  fi
fi

if [[ "${need_ruff}" -eq 0 && "${need_pytest}" -eq 0 ]]; then
  exit 0
fi

python3 - <<PY
import io, json, pathlib, sys, urllib.request, zipfile

tools = pathlib.Path(r"""${TOOLS}""")
bin_dir = tools / "bin"
py_dir = tools / "py"
bin_dir.mkdir(parents=True, exist_ok=True)
py_dir.mkdir(parents=True, exist_ok=True)

def fetch_json(url: str):
    with urllib.request.urlopen(url) as resp:
        return json.load(resp)

def download(url: str) -> bytes:
    with urllib.request.urlopen(url) as resp:
        return resp.read()

if ${need_ruff}:
    meta = fetch_json("https://pypi.org/pypi/ruff/json")
    wheels = [
        u for u in meta["urls"]
        if u["packagetype"] == "bdist_wheel"
        and "manylinux" in u["filename"]
        and "x86_64" in u["filename"]
    ]
    if not wheels:
        sys.exit("error: no manylinux x86_64 ruff wheel found on PyPI")
    wheel = wheels[0]
    print(f"ensure-dev-tools: fetching {wheel['filename']}", flush=True)
    data = download(wheel["url"])
    extract = tools / "ruff_extract"
    if extract.exists():
        import shutil
        shutil.rmtree(extract)
    extract.mkdir()
    with zipfile.ZipFile(io.BytesIO(data)) as zf:
        zf.extractall(extract)
    candidates = [
        p for p in extract.rglob("ruff")
        if p.is_file() and p.stat().st_mode & 0o111
    ]
    if not candidates:
        candidates = [p for p in extract.rglob("ruff") if p.is_file()]
    if not candidates:
        sys.exit("error: ruff binary not found inside wheel")
    target = bin_dir / "ruff"
    target.write_bytes(candidates[0].read_bytes())
    target.chmod(0o755)
    print(f"ensure-dev-tools: installed {target}", flush=True)

if ${need_pytest}:
    for name in ("pytest", "iniconfig", "packaging", "pluggy", "pygments"):
        meta = fetch_json(f"https://pypi.org/pypi/{name}/json")
        wheel = next(
            (
                u for u in meta["urls"]
                if u["packagetype"] == "bdist_wheel"
                and u["filename"].endswith("py3-none-any.whl")
            ),
            None,
        )
        if wheel is None:
            wheel = next(u for u in meta["urls"] if u["packagetype"] == "bdist_wheel")
        print(f"ensure-dev-tools: fetching {wheel['filename']}", flush=True)
        data = download(wheel["url"])
        with zipfile.ZipFile(io.BytesIO(data)) as zf:
            zf.extractall(py_dir)
    print(f"ensure-dev-tools: pytest available via PYTHONPATH={py_dir}", flush=True)
PY
