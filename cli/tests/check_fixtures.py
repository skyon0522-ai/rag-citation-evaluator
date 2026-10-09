"""Execute fixed CLI regressions against actual copied-engine retrieval; standard library only."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

binary, output = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
fixtures = Path(__file__).resolve().parents[1] / "fixtures"
cases = json.loads((fixtures / "cases.json").read_text())
output.mkdir(parents=True, exist_ok=False)
results = []
for case in cases:
    command = [str(binary), str(fixtures / case["corpus"]), str(fixtures / case["request"])]
    completed = subprocess.run(command, capture_output=True, text=True, timeout=30)
    (output / (case["name"] + ".stdout.json")).write_text(completed.stdout)
    (output / (case["name"] + ".stderr.log")).write_text(completed.stderr)
    failures = []
    try:
        report = json.loads(completed.stdout)
        codes = sorted(error["code"] for error in report.get("reference_integrity", {}).get("errors", []))
        if report.get("code"):
            codes.append(report["code"])
        if report.get("retrieval_expectation", {}).get("missing_source_ids"):
            codes.append("missing_expected_retrieval_source")
        observed = {"exit": completed.returncode, "status": report["status"], "codes": sorted(codes),
            "semantic_support": report["semantic_support"], "retrieval_evaluated": report.get("retrieval_evaluated"),
            "retrieved_ids": [item["source_id"] for item in report.get("retrieved", [])]}
        for key, value in case["expected"].items():
            if observed.get(key) != value:
                failures.append({"key": key, "expected": value, "observed": observed.get(key)})
        for item in report.get("retrieved", []):
            if item["stored_text_sha256"] != item["source_sha256"]:
                failures.append({"stored_readback": item["source_id"]})
        if "reference_passed" in case:
            if report["reference_integrity"]["passed"] != case["reference_passed"]:
                failures.append({"reference_passed": report["reference_integrity"]["passed"]})
        if "retrieval_passed" in case:
            if report["retrieval_expectation"]["passed"] != case["retrieval_passed"]:
                failures.append({"retrieval_passed": report["retrieval_expectation"]["passed"]})
    except (KeyError, ValueError) as error:
        report, observed = {}, {"exit": completed.returncode}
        failures.append({"invalid_report": str(error)})
    results.append({"name": case["name"], "command": command, "observed": observed,
        "passed": not failures, "failures": failures, "report": report,
        "fixture_sha256": {key: hashlib.sha256((fixtures / case[key]).read_bytes()).hexdigest() for key in ["corpus", "request"]}})
receipt = {"status": "passed" if all(item["passed"] for item in results) else "failed",
    "fixture_tests": len(results), "passed": sum(item["passed"] for item in results),
    "semantic_support": "not_evaluated", "binary": str(binary),
    "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "results": results,
    "network_services_started": 0, "provider_calls": 0}
(output / "fixture-results.json").write_text(json.dumps(receipt, indent=2, ensure_ascii=False) + "\n")
print(json.dumps({key: receipt[key] for key in ["status", "fixture_tests", "passed", "binary_sha256"]}, indent=2))
sys.exit(0 if receipt["status"] == "passed" else 1)
