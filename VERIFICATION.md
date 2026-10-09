# Verification record and limits

## Recorded run

Run `./verify-portable.sh` to build the CLI with `--locked` and execute the unchanged standard-library fixture verifier against the compiled program. The recorded run used Ubuntu 24.04 x86_64, Rust/Cargo 1.99.0 and Python 3.12.3. It reused previously populated local registry caches and ran Cargo offline. No baseline example was repeated.

All 22 fixtures passed their expected outcomes:

| CLI outcome | Cases | Exit code |
| --- | ---: | ---: |
| Accepted | 3 | 0 |
| Rejected citation or retrieval expectation | 12 | 1 |
| Invalid fixture or request | 7 | 2 |

Fifteen cases executed actual copied-engine retrieval. The fixtures cover a valid quote, a valid multibyte quote, an invented source, a known but unretrieved source, stale version, digest mismatch, changed corpus/source, changed quote, invalid byte spans, UTF-8 boundary failure, missing required citations, duplicate IDs, missing expected retrieval IDs, malformed or empty inputs, and an invalid `top_k`. [`verification/fixture-results.json`](verification/fixture-results.json) records each case's output status, exit code, reason codes, retrieved IDs and fixture hashes.

The missing-expected-source case keeps the two results separate: its citation reference integrity passes, while its retrieval expectation fails. The false-answer counterexample remains structurally accepted with `semantic_support: not_evaluated`. Empty quotations and a changed source can trigger more than one reason; the receipt records each observed reason.

The corrected query description is precise: unqualified queries default to `source_text`, while explicitly qualified queries can use indexed fields such as `source_id`. The native field-query grammar was reviewed in the copied parser source, but no explicit field-query fixture was executed.

## What this does not verify

The CLI checks source identity, retrieval membership, version, text digest, quotation equality and span boundaries for these fixed inputs. It does not measure factuality, relevance, entailment, semantic citation support, per-claim citation completeness, Japanese retrieval quality, security, production suitability or customer value. It runs no NLI model, ALCE or Ragas benchmark. The in-memory index and synthetic fixtures do not establish behavior for arbitrary corpora or workloads. Full upstream Tantivy tests and fresh-machine dependency acquisition were not run in this curation.
