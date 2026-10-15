"""Check the actual review binary against the unchanged direct-engine fixture run."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

if len(sys.argv) != 4:
    raise SystemExit("usage: check_report_bundles.py REVIEW_BINARY NEW_OUTPUT_DIR DIRECT_RECEIPT")
binary, output, baseline = (Path(value).resolve() for value in sys.argv[1:])
fixtures = Path(__file__).resolve().parents[1] / "fixtures"
root = fixtures.parents[1]
# Hash the engine/fixtures and copied component, not the manifest containing the hashes.
protected = json.loads((root / "cli/protected-source-manifest.json").read_text(encoding="utf-8"))
copied = json.loads((root / "source-copy-manifest.json").read_text(encoding="utf-8"))
for base, entries in ((root / "cli", protected["files"].items()),
                      (root / "components/tantivy", ((row["path"], row) for row in copied["files"]))):
    for relative, expected in entries:
        data = (base / relative).read_bytes()
        assert len(data) == expected["bytes"] and hashlib.sha256(data).hexdigest() == expected["sha256"], relative
cases = json.loads((fixtures / "cases.json").read_text(encoding="utf-8"))
direct = json.loads(baseline.read_text(encoding="utf-8"))
assert direct["status"] == "passed" and direct["passed"] == 22
original = {row["name"]: row for row in direct["results"]}
output.mkdir(parents=True, exist_ok=False)
results = []
for case in cases:
    directory = output / case["name"]
    process = subprocess.run([str(binary), str(fixtures / case["corpus"]),
                              str(fixtures / case["request"]), str(directory)],
                             capture_output=True, timeout=30)
    assert process.returncode == case["expected"]["exit"], (case["name"], process.stderr)
    report = json.loads((directory / "engine-result.json").read_text(encoding="utf-8"))
    assert report == original[case["name"]]["report"], case["name"]
    run = json.loads((directory / "run.json").read_text(encoding="utf-8"))
    assert run["wrapper_exit"] == process.returncode and run["protocol_verified"] is True
    for filename, field in (("corpus.json", "corpus"), ("request.json", "request")):
        snapshot = (directory / filename).read_bytes()
        assert snapshot == (fixtures / case[field]).read_bytes()
        assert run["input_sha256"][filename] == hashlib.sha256(snapshot).hexdigest()
    html = (directory / "report.html").read_text(encoding="utf-8")
    assert "Not evaluated" in html and "<script" not in html
    if os.name == "posix":
        assert directory.stat().st_mode & 0o777 == 0o700
        assert all(path.stat().st_mode & 0o777 == 0o600 for path in directory.iterdir())
    results.append({"name": case["name"], "exit": process.returncode,
                    "status": report["status"], "same_as_direct_engine": True,
                    "semantic_support": report["semantic_support"]})

# An intact reference can accompany unsafe answer text; the viewer must show it as text.
request = json.loads((fixtures / "valid_retrieved_quote.json").read_text(encoding="utf-8"))
request["answer"] = '<img src=x onerror="alert(1)"> ' + "x" * 400 + " <script>alert(1)</script>"
unsafe_request = output / "unsafe-text-request.json"
unsafe_request.write_text(json.dumps(request, ensure_ascii=False) + "\n", encoding="utf-8")
directory = output / "unsafe_text_control"
process = subprocess.run([str(binary), str(fixtures / "corpus.json"), str(unsafe_request), str(directory)],
                         capture_output=True, timeout=30)
assert process.returncode == 0, process.stderr
report = json.loads((directory / "engine-result.json").read_text(encoding="utf-8"))
assert report["answer"] == request["answer"] and report["semantic_support"] == "not_evaluated"
html = (directory / "report.html").read_text(encoding="utf-8")
assert "&lt;img" in html and "&lt;script&gt;" in html and "<img" not in html and "<script" not in html
examples = []
for name in ("purchase-policy", "false-answer", "rejected-quote"):
    recorded = root / "examples" / name
    directory = output / name
    process = subprocess.run([str(binary), str(recorded / "corpus.json"),
                              str(recorded / "request.json"), str(directory)],
                             capture_output=True, timeout=30)
    expected = json.loads((recorded / "run.json").read_text(encoding="utf-8"))
    assert process.returncode == expected["wrapper_exit"], (name, process.stderr)
    for filename in ("corpus.json", "request.json", "engine-result.json", "engine-stderr.log", "run.json", "report.html"):
        assert (directory / filename).read_bytes() == (recorded / filename).read_bytes(), (name, filename)
    report = json.loads((directory / "engine-result.json").read_text(encoding="utf-8"))
    assert report["semantic_support"] == "not_evaluated"
    examples.append({"name": name, "status": report["status"], "exit": process.returncode,
                     "recorded_bundle_reproduced": True})
receipt = {"status": "passed", "original_fixture_reports": len(results),
           "all_machine_results_equal_direct_engine": True, "unsafe_text_control": "passed",
           "recorded_examples": examples, "protected_files_verified": len(protected["files"]),
           "copied_component_files_verified": len(copied["files"]),
           "semantic_support": "not_evaluated", "results": results,
           "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
           "scope": "Actual Rust engine through presentation wrapper; no semantic evaluation, network service or provider call"}
(output.parent / "report-results.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
print(json.dumps({key: receipt[key] for key in ("status", "original_fixture_reports", "unsafe_text_control", "recorded_examples", "protected_files_verified", "copied_component_files_verified")}))
