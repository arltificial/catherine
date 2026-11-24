# Purpose: the architecture policy gate's machinery — the shared policy
#   loader, the Cargo.toml extractor, the model builder, the checker runner,
#   the verdict interpreter, and the negative-control self-test (bead
#   catherine-kpn.9).
# Responsibilities:
# - load_policy: the one policy loader (gate + self-test), read-only, schema
#   verbatim from policy/architecture-policy.ct.json — the artifact is never
#   written and the policy is never derived from Cargo.toml; it encodes the
#   design intent of docs/DESIGN.md:156-159, README.md:205, docs/kani-lane.md
# - extract_edges: ACTUAL edges from each workspace member's Cargo.toml
#   [dependencies] ONLY — dev-dependencies never produce a model row — with
#   members discovered from the root Cargo.toml [workspace] section
# - build_document + run_checker + gate: model emitted in the artifact's
#   structure, `catherine check` verdict mapped 0 clean / 3 uncovered / 4
#   dangling (other codes pass through; gate-vs-checker disagreement fails
#   closed, exit 1); failure lines name the offending edge
# - self_test: five checker directions over synthetic models in a temp dir
#   (clean; core>cli; core>genesis_vibes; kani>core; dropped policy edge)
#   plus the [dependencies]-only extractor test
# Rationale: split out of scripts/check-architecture-policy.py for the
#   fail-closed size gate (pretender file_lines <= 300); behavior, exit
#   codes (violation 3, dangling 4), and failure-line formats unchanged.
import json
import os
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
POLICY_RELPATH = Path("policy/architecture-policy.ct.json")
KERNEL_PREFIX = "catherine-"
SUITE = "genesis_vibes"  # README.md:205: suite plumbing, CLI crate only

def load_policy(root: Path) -> dict:
    """The one policy loader (gate and self-test share it)."""
    path = root / POLICY_RELPATH
    try:
        doc = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as err:
        sys.exit(f"architecture-policy: cannot read policy artifact {path}: {err}")
    try:
        rows = doc["instance"]["rows"]["AllowedEdge"]
        allowed_from = doc["instance"]["tables"]["allowed_from"]
        allowed_to = doc["instance"]["tables"]["allowed_to"]
        edges = [(allowed_from[row], allowed_to[row]) for row in rows]
    except (KeyError, TypeError) as err:
        sys.exit(f"architecture-policy: policy artifact malformed ({err}): {path}")
    if len(set(edges)) != len(edges):
        sys.exit(f"architecture-policy: policy lists a duplicate edge: {path}")
    return {"schema": doc["schema"], "edges": edges}

def workspace_members(root: Path) -> list:
    """Workspace member paths, discovered from the root Cargo.toml."""
    manifest_path = root / "Cargo.toml"
    try:
        with open(manifest_path, "rb") as fh:
            members = tomllib.load(fh)["workspace"]["members"]
    except (OSError, tomllib.TOMLDecodeError, KeyError, TypeError) as err:
        sys.exit(f"architecture-policy: cannot read [workspace] members from "
                 f"{manifest_path}: {err}")
    if not all(isinstance(member, str) for member in members):
        sys.exit(f"architecture-policy: [workspace] members must be paths: "
                 f"{manifest_path}")
    return list(members)

def dependency_names(manifest: dict) -> list:
    """[dependencies]-only: a dev-only edge must never yield a row."""
    return [value.get("package", key) if isinstance(value, dict) else key
            for key, value in manifest.get("dependencies", {}).items()]

def extract_edges(root: Path) -> list:
    """Every ACTUAL edge, sorted for deterministic rows."""
    edges = set()
    for member in workspace_members(root):
        manifest_path = root / member / "Cargo.toml"
        try:
            with open(manifest_path, "rb") as fh:
                manifest = tomllib.load(fh)
        except (OSError, tomllib.TOMLDecodeError) as err:
            sys.exit(f"architecture-policy: cannot read manifest "
                     f"{manifest_path}: {err}")
        base = os.path.basename(member.rstrip("/"))
        crate = base[len(KERNEL_PREFIX):] if base.startswith(KERNEL_PREFIX) else base
        for dep in dependency_names(manifest):
            if dep.startswith(KERNEL_PREFIX):
                edges.add((crate, dep[len(KERNEL_PREFIX):]))
            elif dep == "genesis-vibes":
                edges.add((crate, SUITE))
    return sorted(edges)

def build_document(actual: list, policy: dict) -> dict:
    """Every ACTUAL edge in exactly one injection image (coverage), agreeing
    with its policy edge on both endpoints (equations); schema verbatim."""
    allowed = policy["edges"]
    actual_set = set(actual)
    dep_from = {f"dep_{s}_{t}": s for s, t in actual}
    dep_to = {f"dep_{s}_{t}": t for s, t in actual}
    allowed_from = {f"ae_{s}_{t}": s for s, t in allowed}
    allowed_to = {f"ae_{s}_{t}": t for s, t in allowed}
    inject = {f"ae_{s}_{t}": f"dep_{s}_{t}"
              for s, t in allowed if (s, t) in actual_set}
    crates = sorted({end for edge in list(actual) + list(allowed) for end in edge})
    return {
        "format_version": "1.5",
        "schema": policy["schema"],
        "instance": {
            "rows": {"Crate": crates, "Dep": sorted(dep_from),
                     "AllowedEdge": sorted(allowed_from)},
            "tables": {"dep_from": dep_from, "dep_to": dep_to,
                       "allowed_from": allowed_from, "allowed_to": allowed_to,
                       "inject": inject},
        },
    }

def coverage(actual: list, policy: dict) -> tuple:
    """Actual edges no policy edge covers (-> 3), missing policy edges (-> 4)."""
    actual_set, allowed_set = set(actual), set(policy["edges"])
    return sorted(actual_set - allowed_set), sorted(allowed_set - actual_set)

def witness_lines(uncovered: list, dangling: list) -> list:
    """One line per offending edge — failure messages must name edges."""
    lines = [f"uncovered: actual dependency {src}>{tgt} (row "
             f"dep_{src}_{tgt}) is permitted by no policy edge"
             for src, tgt in uncovered]
    lines += [f"dangling: policy edge {src}>{tgt} (row ae_{src}_{tgt}) "
              f"has no counterpart in the tree's [dependencies]"
              for src, tgt in dangling]
    return lines

def find_binary(root: Path, override: str):
    """Locate the catherine CLI, or die with the build instruction."""
    candidates = [override, os.environ.get("CATHERINE_BIN"),
                  str(root / "target" / "debug" / "catherine"),
                  str(root / "target" / "release" / "catherine")]
    for candidate in candidates:
        if candidate and os.path.isfile(candidate) and os.access(candidate, os.X_OK):
            return candidate
    on_path = shutil.which("catherine")
    if on_path:
        return on_path
    looked = ", ".join(c for c in candidates if c)
    sys.exit(f"architecture-policy: catherine CLI binary not found (looked in "
             f"{looked} and $PATH); build it first: cargo build -p catherine-cli "
             f"(or `just check-architecture-policy`)")

def run_checker(binary: str, model_path: Path, cwd: Path) -> tuple:
    """One `catherine check` run; the verdict is its exit code."""
    proc = subprocess.run([binary, "check", str(model_path)], cwd=str(cwd),
                          capture_output=True, text=True, encoding="utf-8")
    return proc.returncode, proc.stdout, proc.stderr

def relay(out: str, err: str) -> None:
    """Pass the checker's streams through on their own channels."""
    if out:
        sys.stdout.write(out)
    if err:
        sys.stderr.write(err)

def write_model(directory: Path, name: str, doc: dict) -> Path:
    """Write a generated model into a (temp) directory — never the repo."""
    path = directory / f"{name}.ct.json"
    path.write_text(json.dumps(doc, indent=1) + "\n", encoding="utf-8")
    return path

def gate(root: Path, bin_override: str) -> int:
    """The real tree: generate, check, interpret. Returns the exit code."""
    policy = load_policy(root)
    actual = extract_edges(root)
    binary = find_binary(root, bin_override)
    with tempfile.TemporaryDirectory() as tmp:
        model = write_model(Path(tmp), "architecture-actual",
                            build_document(actual, policy))
        code, out, err = run_checker(binary, model, root)
    uncovered, dangling = coverage(actual, policy)
    relay(out, err)
    if code == 0:
        if uncovered or dangling:
            sys.exit("architecture-policy: checker passed a model the pairing "
                     "tables call broken — gate and checker disagree")
        print(f"architecture-policy: clean — {len(actual)} actual edge(s) "
              f"covered by {len(policy['edges'])} policy edge(s)")
        return 0
    if code in (3, 4):
        for line in witness_lines(uncovered, dangling):
            print(line)
        if not uncovered and not dangling:
            sys.exit(f"architecture-policy: checker exit {code} but the pairing "
                     "tables name no witness — gate and checker disagree")
        print(f"architecture-policy: FAIL (checker exit {code})", file=sys.stderr)
        return code
    sys.exit(f"architecture-policy: catherine check exited {code} (neither "
             "clean, violation, nor dangling) — see its output above")

def row_spelling(actual: list, policy: dict, row_name: str):
    """Transcript spelling `Object#dense-id` (the renderer cannot print names)."""
    rows = build_document(actual, policy)["instance"]["rows"]
    for object_name, names in rows.items():
        if row_name in names:
            return f"{object_name}#{names.index(row_name)}"
    return None

def clean_direction_ok(code: int, want_code: int, out: str, err: str,
                       lines: list) -> bool:
    """Clean direction: expected class code, total silence everywhere."""
    return (code == want_code and not out.strip() and not err.strip()
            and not lines)

def failing_direction_ok(code: int, want_code: int, out: str, spelling,
                         lines: list) -> bool:
    """Failing direction: expected code, one witness, checker names the
    row in its `Object#dense-id` transcript spelling."""
    return (code == want_code and spelling is not None
            and f"at row {spelling}" in out and len(lines) == 1)

def report_failure(name: str, code: int, want_code: int, spelling, out: str,
                   err: str, lines: list) -> None:
    """Failure transcript: expectation, relayed checker streams, witnesses."""
    print(f"  {name}: FAIL — exit {code}, wanted {want_code} (expected "
          f"`at row {spelling}` in the checker's report)", file=sys.stderr)
    relay(out, err)
    for line in lines:
        print(f"  {line}", file=sys.stderr)

def run_case(root: Path, directory: Path, name: str, actual: list,
             policy: dict, binary: str, want_code: int, want_row) -> bool:
    """One negative-control direction: the checker must return the expected
    class code and its own report must pin the offending row."""
    model = write_model(directory, name, build_document(actual, policy))
    code, out, err = run_checker(binary, model, root)
    lines = witness_lines(*coverage(actual, policy))
    spelling = row_spelling(actual, policy, want_row) if want_row else None
    if want_row is None:  # the clean direction: silence is the contract
        ok = clean_direction_ok(code, want_code, out, err, lines)
    else:
        ok = failing_direction_ok(code, want_code, out, spelling, lines)
    if ok:
        detail = f", checker report names {spelling}" if spelling else ", silent"
        print(f"  {name}: exit {code} as expected{detail}")
        return True
    report_failure(name, code, want_code, spelling, out, err, lines)
    return False

def extractor_test(root: Path) -> bool:
    """A dev-dependencies edge yields NO row; a [dependencies] edge does."""
    manifests = {
        "pos": '[package]\nname = "catherine-pos"\n\n[dependencies]\n'
               'catherine-core = "0.4"\n',
        "neg": '[package]\nname = "catherine-neg"\n\n[dependencies]\n'
               'serde = "1"\n\n[dev-dependencies]\n'
               'catherine-core = { path = "../catherine-core" }\n',
    }
    with tempfile.TemporaryDirectory() as tmp:
        tree = Path(tmp)
        for crate, text in manifests.items():
            crate_dir = tree / "crates" / f"catherine-{crate}"
            crate_dir.mkdir(parents=True)
            (crate_dir / "Cargo.toml").write_text(text)
        (tree / "Cargo.toml").write_text(
            '[workspace]\nmembers = ["crates/catherine-pos", "crates/catherine-neg"]\n')
        edges = extract_edges(tree)
        rows = build_document(edges, load_policy(root))["instance"]["rows"]["Dep"]
        ok = edges == [("pos", "core")] and rows == ["dep_pos_core"]
    if ok:
        print("  extractor test: dev-only edge produced no row, "
              "[dependencies] edge produced dep_pos_core")
    else:
        print(f"  extractor test: FAIL — edges={edges} dep_rows={rows}",
              file=sys.stderr)
    return ok

def self_test(root: Path) -> int:
    """Five checker directions + the extractor test; red until correct."""
    failures = []
    if not extractor_test(root):
        failures.append("extractor-test")
    policy = load_policy(root)
    binary = find_binary(root, None)
    allowed = policy["edges"]
    cases = [
        ("clean", sorted(allowed), 0, None),
        ("inject_core_cli", sorted(set(allowed) | {("core", "cli")}), 3, "dep_core_cli"),
        ("inject_core_genesis_vibes", sorted(set(allowed) | {("core", SUITE)}), 3, "dep_core_genesis_vibes"),
        ("inject_kani_core", sorted(set(allowed) | {("kani", "core")}), 3, "dep_kani_core"),
        ("drop_cli_analysis", sorted(e for e in allowed if e != ("cli", "analysis")), 4, "ae_cli_analysis"),
    ]
    with tempfile.TemporaryDirectory() as tmp:
        for name, actual, want_code, want_row in cases:
            if not run_case(root, Path(tmp), name, actual, policy, binary,
                            want_code, want_row):
                failures.append(name)
    if failures:
        print("architecture-policy self-test: FAIL — " + ", ".join(failures),
              file=sys.stderr)
        return 1
    print("architecture-policy self-test: 5 directions + extractor test pass")
    return 0
