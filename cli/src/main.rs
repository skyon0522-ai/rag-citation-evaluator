// Retrieval sequence adapted from pinned Tantivy examples/basic_search.rs.
// Copyright (c) 2018 by the project authors listed in retained AUTHORS; see LICENSE.
use serde::Deserialize;
use serde_json::{json, Value as Json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};
use std::{env, error::Error, fs, process};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source { id: String, version: String, text: String }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Corpus { sources: Vec<Source> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Citation {
    source_id: String, source_version: String, source_sha256: String,
    start: usize, end: usize, quote: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    query: String, top_k: usize, corpus_sha256: String,
    expected_source_ids: Vec<String>, answer: String, citations: Vec<Citation>,
}

fn sha256(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }
fn nonempty(text: &str) -> bool { !text.trim().is_empty() }
fn digest_shape(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn invalid(code: &str, detail: impl ToString) -> (Json, i32) {
    (json!({"status":"invalid_fixture", "code":code, "detail":detail.to_string(),
        "retrieval_evaluated":false, "semantic_support":"not_evaluated"}), 2)
}

fn evaluate(corpus: Corpus, request: Request, corpus_digest: String) -> Result<(Json, i32), Box<dyn Error>> {
    if corpus.sources.is_empty() { return Ok(invalid("empty_corpus", "At least one source is required.")); }
    let mut ids = BTreeSet::new();
    for source in &corpus.sources {
        if !nonempty(&source.id) || !nonempty(&source.version) || !nonempty(&source.text) {
            return Ok(invalid("empty_source_field", "Every source needs nonempty id, version and text."));
        }
        if !ids.insert(source.id.clone()) { return Ok(invalid("duplicate_source_id", &source.id)); }
    }
    if !nonempty(&request.query) || !nonempty(&request.answer) || !(1..=5).contains(&request.top_k) {
        return Ok(invalid("invalid_request_fields", "Nonempty query/answer and top_k 1..5 are required."));
    }
    if !digest_shape(&request.corpus_sha256) {
        return Ok(invalid("invalid_corpus_digest", "Expected lowercase SHA-256 hexadecimal."));
    }
    if request.expected_source_ids.is_empty()
        || request.expected_source_ids.iter().any(|s| !nonempty(s))
        || request.expected_source_ids.iter().collect::<BTreeSet<_>>().len() != request.expected_source_ids.len() {
        return Ok(invalid("expected_sources_required", "Distinct nonempty expected retrieval IDs are required."));
    }
    if request.citations.is_empty() { return Ok(invalid("citations_required", "At least one citation is required.")); }
    if request.citations.iter().any(|c| !nonempty(&c.source_id) || !nonempty(&c.source_version) || !digest_shape(&c.source_sha256)) {
        return Ok(invalid("invalid_citation_fields", "Citation ID/version and lowercase source digest are required."));
    }

    // minimalism: one in-memory index per request; no server, persistent index or agent framework.
    let mut schema_builder = Schema::builder();
    let id = schema_builder.add_text_field("source_id", STRING | STORED);
    let text = schema_builder.add_text_field("source_text", TEXT | STORED);
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema);
    let mut writer: IndexWriter = index.writer(50_000_000)?;
    for source in &corpus.sources {
        writer.add_document(doc!(id => source.id.as_str(), text => source.text.as_str()))?;
    }
    writer.commit()?;
    let reader = index.reader_builder().reload_policy(ReloadPolicy::OnCommitWithDelay).try_into()?;
    let searcher = reader.searcher();
    let parser = QueryParser::for_index(&index, vec![text]);
    let query = match parser.parse_query(&request.query) {
        Ok(query) => query,
        Err(error) => return Ok(invalid("invalid_query", error)),
    };
    let top_docs = searcher.search(&query, &TopDocs::with_limit(request.top_k).order_by_score())?;
    let sources: HashMap<_, _> = corpus.sources.iter().map(|s| (s.id.as_str(), s)).collect();
    let mut selected = BTreeSet::new();
    let mut retrieved = Vec::new();
    for (score, address) in top_docs {
        let stored: TantivyDocument = searcher.doc(address)?;
        let source_id = stored.get_first(id).and_then(|v| v.as_str().map(str::to_owned)).ok_or("Stored ID missing.")?;
        let stored_text = stored.get_first(text).and_then(|v| v.as_str().map(str::to_owned)).ok_or("Stored text missing.")?;
        let source = sources.get(source_id.as_str()).ok_or("Stored ID absent from corpus.")?;
        if stored_text != source.text { return Err("Stored UTF-8 text differs from the input source.".into()); }
        selected.insert(source_id.clone());
        retrieved.push(json!({"source_id":source_id, "source_version":source.version,
            "source_sha256":sha256(source.text.as_bytes()), "stored_text_sha256":sha256(stored_text.as_bytes()), "score":score}));
    }
    let missing: Vec<_> = request.expected_source_ids.iter().filter(|id| !selected.contains(*id)).collect();
    let mut errors = Vec::new();
    if request.corpus_sha256 != corpus_digest {
        errors.push(json!({"code":"corpus_snapshot_mismatch", "expected":request.corpus_sha256, "actual":corpus_digest}));
    }
    for (position, citation) in request.citations.iter().enumerate() {
        let mut reject = |code: &str| errors.push(json!({"citation_index":position, "source_id":citation.source_id, "code":code}));
        let Some(source) = sources.get(citation.source_id.as_str()) else { reject("unknown_source"); continue; };
        if !selected.contains(&citation.source_id) { reject("source_not_retrieved"); }
        if citation.source_version != source.version { reject("source_version_mismatch"); }
        if citation.source_sha256 != sha256(source.text.as_bytes()) { reject("source_digest_mismatch"); }
        if !nonempty(&citation.quote) { reject("empty_quote"); }
        if citation.start >= citation.end || citation.end > source.text.len() {
            reject("invalid_span_bounds");
        } else if !source.text.is_char_boundary(citation.start) || !source.text.is_char_boundary(citation.end) {
            reject("invalid_utf8_boundary");
        } else if source.text[citation.start..citation.end] != citation.quote {
            reject("quote_mismatch");
        }
    }
    let integrity_passed = errors.is_empty();
    let retrieval_passed = missing.is_empty();
    let accepted = integrity_passed && retrieval_passed;
    Ok((json!({"status":if accepted {"accepted"} else {"rejected"}, "corpus_sha256":corpus_digest,
        "query":request.query, "top_k":request.top_k, "retrieval_evaluated":true, "retrieved":retrieved,
        "retrieval_expectation":{"passed":retrieval_passed, "expected_source_ids":request.expected_source_ids, "missing_source_ids":missing},
        "reference_integrity":{"passed":integrity_passed, "errors":errors},
        "answer":request.answer, "semantic_support":"not_evaluated",
        "engine":{"package":"tantivy", "dependency":"../components/tantivy", "source_pin":"72d1ef9a6468aa68bbc69dcc80cdf60aaf64364d"}}), if accepted {0} else {1}))
}

fn run() -> Result<(Json, i32), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 { return Ok(invalid("usage", "rag-reference-contract CORPUS.json REQUEST.json")); }
    let corpus_bytes = fs::read(&args[1])?;
    let corpus = match serde_json::from_slice(&corpus_bytes) {
        Ok(corpus) => corpus, Err(error) => return Ok(invalid("invalid_corpus_json", error)),
    };
    let request = match serde_json::from_slice(&fs::read(&args[2])?) {
        Ok(request) => request, Err(error) => return Ok(invalid("invalid_request_json", error)),
    };
    evaluate(corpus, request, sha256(&corpus_bytes))
}
fn main() {
    let (report, exit) = run().unwrap_or_else(|error| (json!({"status":"execution_error", "detail":error.to_string(), "semantic_support":"not_evaluated"}), 3));
    println!("{}", serde_json::to_string_pretty(&report).expect("JSON report serialization"));
    process::exit(exit);
}
