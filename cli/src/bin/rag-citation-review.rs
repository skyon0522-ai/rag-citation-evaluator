// Copyright (c) 2026 skyon0522-ai. SPDX-License-Identifier: MIT
// Offline presentation around the unchanged sibling reference-contract executable.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::{env, error::Error};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn show(value: &Value) -> String {
    match value {
        Value::Null => "Not reported".into(),
        Value::String(text) => escape(text),
        _ => escape(&value.to_string()),
    }
}

fn array(value: &Value) -> &[Value] {
    value.as_array().map(Vec::as_slice).unwrap_or(&[])
}

fn outcome(value: &Value) -> &'static str {
    match value.as_bool() {
        Some(true) => "Passed",
        Some(false) => "Failed",
        None => "Not reported",
    }
}

fn validate_response(report: &Value, exit: i32) -> std::result::Result<(), String> {
    let expected = match exit {
        0 => "accepted",
        1 => "rejected",
        2 => "invalid_fixture",
        3 => "execution_error",
        _ => return Err(format!("Unexpected engine exit code: {exit}")),
    };
    if report["status"] != expected || report["semantic_support"] != "not_evaluated" {
        return Err(
            "The engine status/exit code or semantic-support contract is inconsistent.".into(),
        );
    }
    if exit <= 1 {
        let retrieval = report["retrieval_expectation"]["passed"].as_bool();
        let reference = report["reference_integrity"]["passed"].as_bool();
        if report["retrieval_evaluated"] != true
            || retrieval.is_none()
            || reference.is_none()
            || !report["retrieved"].is_array()
            || !report["reference_integrity"]["errors"].is_array()
            || (retrieval == Some(true) && reference == Some(true)) != (exit == 0)
        {
            return Err("The engine's retrieval/reference result fields are inconsistent.".into());
        }
    } else if !report["detail"].is_string() || (exit == 2 && report["retrieval_evaluated"] != false)
    {
        return Err("The engine error result is incomplete.".into());
    }
    Ok(())
}

fn render(report: &Value, request: &Value, run: &Value) -> String {
    let absent = Value::Null;
    let data = if run["protocol_verified"] == true {
        report
    } else {
        &absent
    };
    let title = match data["status"].as_str() {
        Some("accepted") => "Reference contract accepted",
        Some("rejected") => "Reference contract rejected",
        Some("invalid_fixture") => "Input is invalid",
        Some("execution_error") => "Engine execution failed",
        _ => "Execution could not be verified",
    };
    let mut issues = String::new();
    for value in [&run["error"], &data["code"], &data["detail"]] {
        if !value.is_null() {
            issues.push_str(&format!("<li>{}</li>", show(value)));
        }
    }
    for error in array(&data["reference_integrity"]["errors"]) {
        if error["citation_index"].is_null() {
            issues.push_str(&format!("<li>{}</li>", show(error)));
        }
    }
    if !issues.is_empty() {
        issues =
            format!("<section class='panel'><h2>Run diagnostics</h2><ul>{issues}</ul></section>");
    }
    let mut retrieved = String::new();
    for item in array(&data["retrieved"]) {
        retrieved.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td><code>{}</code></td><td>{}</td></tr>",
            show(&item["source_id"]),
            show(&item["source_version"]),
            show(&item["source_sha256"]),
            show(&item["score"])
        ));
    }
    if retrieved.is_empty() {
        retrieved.push_str("<tr><td colspan='4'>No retrieval results reported</td></tr>");
    }
    let mut citations = String::new();
    for (index, citation) in array(&request["citations"]).iter().enumerate() {
        let codes = array(&data["reference_integrity"]["errors"])
            .iter()
            .filter(|error| error["citation_index"].as_u64() == Some(index as u64))
            .map(|error| show(&error["code"]))
            .collect::<Vec<_>>()
            .join(", ");
        let result = if !codes.is_empty() {
            codes
        } else if data["reference_integrity"].is_object() {
            "No reference error reported".into()
        } else {
            "Reference checks not completed".into()
        };
        citations.push_str(&format!(
            r#"<article class="citation"><h3>Citation {}</h3>
            <p><strong>{}</strong> · version {}</p><blockquote>{}</blockquote>
            <p>UTF-8 byte span: {}–{}<br>Expected source SHA-256: <code>{}</code></p>
            <p class="result">{}</p></article>"#,
            index + 1,
            show(&citation["source_id"]),
            show(&citation["source_version"]),
            show(&citation["quote"]),
            show(&citation["start"]),
            show(&citation["end"]),
            show(&citation["source_sha256"]),
            result
        ));
    }
    if citations.is_empty() {
        citations.push_str("<p>No citation list could be displayed from the request.</p>");
    }
    let query = request.get("query").unwrap_or(&data["query"]);
    let answer = request.get("answer").unwrap_or(&data["answer"]);
    let raw = escape(&serde_json::to_string_pretty(report).expect("serializable JSON value"));
    format!(
        r##"<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'">
<title>RAG citation review</title><style>
body{{font:16px/1.55 system-ui,sans-serif;margin:0;background:#f4f7fb;color:#182736}}
main{{max-width:70rem;margin:auto;padding:2rem 1rem}}h1{{line-height:1.2}}p{{max-width:75ch}}
.panel{{background:white;border:1px solid #d1dce8;border-radius:.6rem;padding:1.5rem;margin:1.2rem 0;overflow-wrap:anywhere}}
.grid{{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:1rem}}
.grid section{{padding:1rem;background:#eaf1f8}}.grid h2{{font-size:1rem;margin:0 0 .5rem}}
.grid p{{font-weight:600;margin:0}}.scroll{{overflow:auto;max-width:100%}}
.scroll:focus-visible,a:focus-visible,summary:focus-visible{{outline:3px solid #155b87;outline-offset:2px}}
table{{border-collapse:collapse;width:100%;min-width:46rem}}th,td{{text-align:left;padding:.7rem;border-bottom:1px solid #d1dce8}}
th,td:first-child{{white-space:nowrap}}
thead{{background:#eaf1f8}}code,pre{{overflow-wrap:anywhere}}pre,blockquote{{white-space:pre-wrap}}
pre{{padding:1rem;background:#eaf1f8}}a{{color:#155b87}}summary{{cursor:pointer;font-weight:600}}
.citation{{padding:.5rem 0 1rem;border-bottom:1px solid #d1dce8}}blockquote{{margin:1rem 0;padding-left:1rem;border-left:3px solid #d1dce8}}
.result{{font-weight:600}}.answer{{white-space:pre-wrap}}
@media(max-width:600px){{main{{padding:1rem .5rem}}.panel{{padding:1rem}}.grid{{grid-template-columns:1fr}}}}
@media print{{body{{background:white}}main{{max-width:none;padding:0}}.panel{{border:0;padding:0}}}}
</style></head><body><main><h1>RAG citation review</h1>
<p>This checks retrieval and citation references. It does not evaluate answer truth or whether a quotation supports a claim.</p>
<section class="panel"><h2>{title}</h2><p>Engine exit code: {exit}</p>
<div class="grid"><section><h2>Expected retrieval IDs</h2><p>{retrieval}</p></section>
<section><h2>Reference integrity</h2><p>{reference}</p></section>
<section><h2>Answer support</h2><p>Not evaluated</p></section></div></section>
{issues}<section class="panel"><h2>Query and answer</h2><h3>Query</h3><p>{query}</p>
<h3>Answer supplied to the check</h3><p class="answer">{answer}</p>
<p>Structural acceptance can coexist with a false answer. No semantic support score was computed.</p></section>
<section class="panel"><h2>Retrieved sources</h2><p>Missing expected IDs: {missing}</p>
<div class="scroll" role="region" aria-label="Retrieved sources" tabindex="0"><table>
<thead><tr><th scope="col">Source</th><th scope="col">Version</th><th scope="col">Source SHA-256</th><th scope="col">Search score</th></tr></thead>
<tbody>{retrieved}</tbody></table></div><p>On small screens, swipe the table sideways or focus it and use the arrow keys.</p></section>
<section class="panel"><h2>Requested citations</h2>{citations}</section>
<details class="panel"><summary>Engine result and reproducibility</summary><pre>{raw}</pre>
<p><a href="engine-result.json">Unmodified engine result</a> · <a href="run.json">Run receipt</a> ·
<a href="corpus.json">Corpus snapshot</a> · <a href="request.json">Request snapshot</a> · <a href="engine-stderr.log">Engine stderr</a></p>
<p>This bundle contains the full input corpus, query, answer and quotations. Inspect its contents before sharing.</p></details>
</main></body></html>"##,
        exit = show(&run["engine_exit"]),
        retrieval = outcome(&data["retrieval_expectation"]["passed"]),
        reference = outcome(&data["reference_integrity"]["passed"]),
        missing = show(&data["retrieval_expectation"]["missing_source_ids"]),
        query = show(query),
        answer = show(answer)
    )
}

fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)?.write_all(bytes)
}

fn review_with_engine(
    engine: &Path,
    corpus: &Path,
    request: &Path,
    directory: &Path,
) -> Result<i32> {
    match directory.symlink_metadata() {
        Ok(_) => {
            return Err("Choose a new output directory; existing paths are not replaced.".into())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => (),
        Err(error) => return Err(error.into()),
    }
    let engine = engine.canonicalize().map_err(|error| {
        format!(
            "Cannot locate the sibling engine {}: {error}",
            engine.display()
        )
    })?;
    let corpus_bytes = fs::read(corpus)
        .map_err(|error| format!("Cannot read corpus {}: {error}", corpus.display()))?;
    let request_bytes = fs::read(request)
        .map_err(|error| format!("Cannot read request {}: {error}", request.display()))?;
    if let Some(parent) = directory.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    let mut builder = DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(directory)?;
    let directory = directory.canonicalize()?;
    write_new(&directory.join("corpus.json"), &corpus_bytes)?;
    write_new(&directory.join("request.json"), &request_bytes)?;
    let output = Command::new(engine)
        .args(["corpus.json", "request.json"])
        .current_dir(&directory)
        .output()
        .map_err(|error| format!("Cannot run the reference engine: {error}"))?;
    write_new(&directory.join("engine-result.json"), &output.stdout)?;
    write_new(&directory.join("engine-stderr.log"), &output.stderr)?;
    let parsed = serde_json::from_slice::<Value>(&output.stdout);
    let error = match (&parsed, output.status.code()) {
        (Ok(report), Some(exit)) => validate_response(report, exit).err(),
        (Err(error), _) => Some(format!("The engine did not return JSON: {error}")),
        (_, None) => Some("The engine terminated without a normal exit code.".into()),
    };
    let exit = if error.is_none() {
        output.status.code().expect("checked exit code")
    } else {
        4
    };
    let report = parsed.unwrap_or(Value::Null);
    let request = serde_json::from_slice::<Value>(&request_bytes).unwrap_or(Value::Null);
    let run = json!({"schema_version":1,"engine_binary":"rag-reference-contract","engine_exit":output.status.code(),
        "wrapper_exit":exit,"protocol_verified":error.is_none(),"error":error,
        "input_sha256":{"corpus.json":format!("{:x}",Sha256::digest(&corpus_bytes)),
                        "request.json":format!("{:x}",Sha256::digest(&request_bytes))},
        "semantic_support":"not_evaluated"});
    write_new(
        &directory.join("report.html"),
        render(&report, &request, &run).as_bytes(),
    )?;
    write_new(
        &directory.join("run.json"),
        &serde_json::to_vec_pretty(&run)?,
    )?;
    println!(
        "Saved {} (engine exit {}, report exit {exit})",
        directory.join("report.html").display(),
        show(&run["engine_exit"])
    );
    Ok(exit)
}

fn run(args: Vec<OsString>) -> Result<i32> {
    if args.len() == 2 && args[1] == "--help" {
        println!("rag-citation-review CORPUS.json REQUEST.json NEW_OUTPUT_DIRECTORY\nKeeps engine exit codes 0/1/2/3; wrapper setup/output/protocol errors return 4.");
        return Ok(0);
    }
    if args.len() != 4 {
        return Err(
            "usage: rag-citation-review CORPUS.json REQUEST.json NEW_OUTPUT_DIRECTORY".into(),
        );
    }
    let engine: PathBuf = env::current_exe()?
        .with_file_name(format!("rag-reference-contract{}", env::consts::EXE_SUFFIX));
    review_with_engine(
        &engine,
        Path::new(&args[1]),
        Path::new(&args[2]),
        Path::new(&args[3]),
    )
}

fn main() {
    let exit = run(env::args_os().collect()).unwrap_or_else(|error| {
        eprintln!("rag-citation-review: {error}\nA partial new bundle may remain; inspect it and choose a new directory before retrying.");
        4
    });
    process::exit(exit);
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let path = env::temp_dir().join(format!(
                "rag-review-test-{}-{}",
                process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn engine(&self, response: &Value, exit: i32, extra: &str) -> PathBuf {
            let path = self.0.join("engine");
            fs::write(
                self.0.join("engine.json"),
                serde_json::to_vec(response).unwrap(),
            )
            .unwrap();
            fs::write(&path, format!("#!/bin/sh\n/bin/cat \"$0.json\"\nprintf ran >\"$0.marker\"\n{extra}\nexit {exit}\n")).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
            path
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            assert!(self.0.starts_with(env::temp_dir()));
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    fn accepted() -> Value {
        json!({"status":"accepted","semantic_support":"not_evaluated","retrieval_evaluated":true,
            "retrieved":[],"retrieval_expectation":{"passed":true,"missing_source_ids":[]},
            "reference_integrity":{"passed":true,"errors":[]}})
    }
    #[test]
    fn escaping_and_semantic_limit_are_visible() {
        let html = render(
            &accepted(),
            &json!({"query":"<script>bad()</script>","answer":"A false answer",
            "citations":[{"quote":"<&\"'", "source_id":"<img src=x>"}]}),
            &json!({"protocol_verified":true,"engine_exit":0}),
        );
        assert!(html.contains("A false answer"));
        assert!(html.contains("Not evaluated"));
        assert!(html.contains("Structural acceptance can coexist with a false answer"));
        assert!(html.contains("&lt;script&gt;bad()&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<img src=x>"));
    }
    #[test]
    fn status_and_transport_must_agree() {
        assert!(validate_response(&accepted(), 0).is_ok());
        assert!(validate_response(&accepted(), 1).is_err());
        let mut bad = accepted();
        bad["semantic_support"] = json!("evaluated");
        assert!(validate_response(&bad, 0).is_err());
        let mut contradictory = accepted();
        contradictory["reference_integrity"]["passed"] = json!(false);
        assert!(validate_response(&contradictory, 0).is_err());
    }
    #[test]
    fn valid_engine_failure_codes_are_kept_separate_from_wrapper_errors() {
        let mut rejected = accepted();
        rejected["status"] = json!("rejected");
        rejected["reference_integrity"]["passed"] = json!(false);
        for (code, response) in [
            (1, rejected),
            (
                2,
                json!({"status":"invalid_fixture","detail":"invalid input",
                "retrieval_evaluated":false,"semantic_support":"not_evaluated"}),
            ),
            (
                3,
                json!({"status":"execution_error","detail":"engine failed",
                "semantic_support":"not_evaluated"}),
            ),
        ] {
            let temp = Temp::new();
            let engine = temp.engine(&response, code, "");
            let input = temp.0.join("input");
            fs::write(&input, b"{}").unwrap();
            let output = temp.0.join("result");
            assert_eq!(
                review_with_engine(&engine, &input, &input, &output).unwrap(),
                code
            );
            let run: Value =
                serde_json::from_slice(&fs::read(output.join("run.json")).unwrap()).unwrap();
            assert_eq!(run["wrapper_exit"], code);
            assert_eq!(run["protocol_verified"], true);
        }
    }
    #[test]
    fn snapshots_result_and_private_modes_are_preserved() {
        let temp = Temp::new();
        let engine = temp.engine(&accepted(), 0, "");
        let corpus = temp.0.join("source-corpus");
        let request = temp.0.join("source-request");
        fs::write(&corpus, b"source UTF-8 bytes").unwrap();
        fs::write(&request, b"{}").unwrap();
        let output = temp.0.join("result");
        assert_eq!(
            review_with_engine(&engine, &corpus, &request, &output).unwrap(),
            0
        );
        assert_eq!(
            fs::read(output.join("corpus.json")).unwrap(),
            fs::read(corpus).unwrap()
        );
        assert_eq!(
            fs::read(output.join("engine-result.json")).unwrap(),
            serde_json::to_vec(&accepted()).unwrap()
        );
        assert_eq!(
            output.metadata().unwrap().permissions().mode() & 0o777,
            0o700
        );
        for name in [
            "corpus.json",
            "request.json",
            "engine-result.json",
            "engine-stderr.log",
            "run.json",
            "report.html",
        ] {
            assert_eq!(
                output.join(name).metadata().unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
    #[test]
    fn existing_output_is_refused_before_running_engine() {
        let temp = Temp::new();
        let engine = temp.engine(&accepted(), 0, "");
        let output = temp.0.join("result");
        fs::create_dir(&output).unwrap();
        fs::write(output.join("sentinel"), b"previous").unwrap();
        assert!(
            review_with_engine(&engine, Path::new("missing"), Path::new("missing"), &output)
                .is_err()
        );
        assert!(!temp.0.join("engine.marker").exists());
        assert_eq!(fs::read(output.join("sentinel")).unwrap(), b"previous");
    }
    #[test]
    fn broken_output_symlink_is_refused() {
        let temp = Temp::new();
        let output = temp.0.join("dangling");
        std::os::unix::fs::symlink("missing", &output).unwrap();
        assert!(review_with_engine(
            Path::new("missing"),
            Path::new("missing"),
            Path::new("missing"),
            &output
        )
        .is_err());
    }
    #[test]
    fn protocol_error_cannot_appear_as_acceptance() {
        let temp = Temp::new();
        let mut bad = accepted();
        bad["semantic_support"] = json!("evaluated");
        let engine = temp.engine(&bad, 0, "");
        let input = temp.0.join("input");
        fs::write(&input, b"{}").unwrap();
        let output = temp.0.join("result");
        assert_eq!(
            review_with_engine(&engine, &input, &input, &output).unwrap(),
            4
        );
        let html = fs::read_to_string(output.join("report.html")).unwrap();
        assert!(html.contains("Execution could not be verified"));
        assert!(!html.contains("Reference contract accepted"));
    }
    #[test]
    fn truncated_json_cannot_appear_as_acceptance() {
        let temp = Temp::new();
        let engine = temp.engine(&accepted(), 0, "");
        fs::write(temp.0.join("engine.json"), b"{truncated").unwrap();
        let input = temp.0.join("input");
        fs::write(&input, b"{}").unwrap();
        let output = temp.0.join("result");
        assert_eq!(
            review_with_engine(&engine, &input, &input, &output).unwrap(),
            4
        );
        let run: Value =
            serde_json::from_slice(&fs::read(output.join("run.json")).unwrap()).unwrap();
        assert_eq!(run["protocol_verified"], false);
        assert!(run["error"]
            .as_str()
            .unwrap()
            .contains("did not return JSON"));
    }
    #[test]
    fn signaled_engine_cannot_appear_as_acceptance_even_after_printing_json() {
        let temp = Temp::new();
        let engine = temp.engine(&accepted(), 0, "kill -TERM \"$$\"");
        let input = temp.0.join("input");
        fs::write(&input, b"{}").unwrap();
        let output = temp.0.join("result");
        assert_eq!(
            review_with_engine(&engine, &input, &input, &output).unwrap(),
            4
        );
        let run: Value =
            serde_json::from_slice(&fs::read(output.join("run.json")).unwrap()).unwrap();
        assert_eq!(run["engine_exit"], Value::Null);
        assert_eq!(run["protocol_verified"], false);
    }
    #[test]
    fn colliding_file_is_preserved_on_output_failure() {
        let temp = Temp::new();
        let engine = temp.engine(&accepted(), 0, "printf keep >report.html");
        let input = temp.0.join("input");
        fs::write(&input, b"{}").unwrap();
        let output = temp.0.join("result");
        assert!(review_with_engine(&engine, &input, &input, &output).is_err());
        assert_eq!(fs::read(output.join("report.html")).unwrap(), b"keep");
        assert!(output.join("engine-result.json").exists());
    }
}
