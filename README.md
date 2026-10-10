# RAG citation reference contract

This Rust CLI checks a narrow source-and-retrieval contract for citations in a fixed synthetic corpus. It confirms that a cited source exists, appeared in the selected results for the query, has the expected version and text digest, and contains the exact nonempty quotation at a valid UTF-8 byte span. It reports expected retrieval-ID presence separately from citation reference integrity.

This package does not decide whether an answer is true or whether a quotation supports its claim. A deliberately false answer with an intact retrieved quotation is accepted structurally and reports `semantic_support: not_evaluated`.

## Where to look

| Question | File |
| --- | --- |
| What does the evaluator actually check? | [CLI source](cli/src/main.rs) |
| Which cases are run? | [Fixture manifest](cli/fixtures/cases.json) and [portable runner](verify-portable.sh) |
| Which code is copied or adapted? | [Provenance](PROVENANCE.md), [copied Tantivy](components/tantivy/) and [derivation diff](derivation.diff) |
| What passed, and what remains outside the claim? | [Verification](VERIFICATION.md), [fixture results](verification/fixture-results.json) and [research limits](RESEARCH.md) |

## Run the fixed checks

Requirements are Rust and Cargo compatible with Tantivy's declared Rust 1.86 minimum, plus Python 3. The task run used Rust/Cargo 1.99.0 and Python 3.12.3 on Ubuntu 24.04 x86_64.

```sh
./verify-portable.sh
```

The entrypoint builds the CLI with the checked-in lockfile and runs the 22 fixed fixtures through the compiled CLI. By default it keeps build and fixture outputs in a temporary directory. To retain the fixture outputs, pass a new output directory:

```sh
./verify-portable.sh /tmp/rag-citation-results
```

The recorded task run passed all 22 fixtures: 3 accepted, 12 rejected, and 7 invalid. Fifteen cases performed real Tantivy retrieval. See [the fixture receipt](verification/fixture-results.json) and [the run record](verification/portable-run.json) for the case outcomes and environment summary.

The CLI uses the complete local Tantivy copy at [`components/tantivy`](components/tantivy/Cargo.toml), pinned to commit `72d1ef9a6468aa68bbc69dcc80cdf60aaf64364d`. Unqualified queries default to `source_text`; Tantivy's native field-qualified grammar also permits explicitly indexed fields such as `source_id`. The documented `top_k` range is 1–5. No explicit `source_id` field-query fixture was executed.

Read [provenance](PROVENANCE.md), [verification scope](VERIFICATION.md), [research limits](RESEARCH.md), and [third-party notices](NOTICE) before reusing the bundle.
