# Verification record and limits

## Recorded run

The original recorded run built the CLI with `--locked` and executed the
unchanged standard-library fixture verifier against the compiled program. It
used Ubuntu 24.04 x86_64, Rust/Cargo 1.99.0 and Python 3.12.3, reused populated
local registry caches and ran Cargo offline. No upstream baseline example was
repeated during that curation. The current portable entrypoint also checks the
new presentation wrapper as described below.

All 22 fixtures passed their expected outcomes:

| CLI outcome | Cases | Exit code |
| --- | ---: | ---: |
| Accepted | 3 | 0 |
| Rejected citation or retrieval expectation | 12 | 1 |
| Invalid fixture or request | 7 | 2 |

Fifteen cases executed actual copied-engine retrieval. The fixtures cover a valid quote, a valid multibyte quote, an invented source, a known but unretrieved source, stale version, digest mismatch, changed corpus/source, changed quote, invalid byte spans, UTF-8 boundary failure, missing required citations, duplicate IDs, missing expected retrieval IDs, malformed or empty inputs, and an invalid `top_k`. [`verification/fixture-results.json`](verification/fixture-results.json) records each case's output status, exit code, reason codes, retrieved IDs and fixture hashes.

The missing-expected-source case keeps the two results separate: its citation reference integrity passes, while its retrieval expectation fails. The false-answer counterexample remains structurally accepted with `semantic_support: not_evaluated`. Empty quotations and a changed source can trigger more than one reason; the receipt records each observed reason.

The corrected query description is precise: unqualified queries default to `source_text`, while explicitly qualified queries can use indexed fields such as `source_id`. The native field-query grammar was reviewed in the copied parser source, but no explicit field-query fixture was executed.

## Continuous integration

The [portable workflow](.github/workflows/portable.yml) reuses `verify-portable.sh`:
it builds both actual local-path binaries with the lockfile, runs the wrapper's
boundary tests, then checks the 22 fixtures directly and through the wrapper.
An unsafe-text control checks escaped answer display; three recorded examples
must reproduce all six bundle files byte for byte. The same checker verifies
33 protected engine/fixture files and all 518 copied component files. The original
checkout, toolchain and build/test sequence follows the inspected copied
[Tantivy test workflow](components/tantivy/.github/workflows/test.yml) and the
rustup installation in its [coverage workflow](components/tantivy/.github/workflows/coverage.yml).

CI selects Ubuntu 24.04, Rust/Cargo 1.99.0 and Python 3.12.3. Third-party Actions are pinned by commit, the repository token has only `contents: read`, and checkout does not retain its credentials. CI starts without a project dependency cache and may acquire locked registry dependencies; it is not an offline or bit-for-bit reproducible build. The hosted runner image can change within its Ubuntu label.

The configured seven-day `rag-fixture-results` artifact uploads
`fixture-results.json` and `report-results.json` from the checked-in synthetic
checks, not input bundles, the repository or build directory. Keep private
corpora and credentials out of fixtures: offline report bundles contain the
full corpus and request bytes as well as query and answer text.

Observed on **2026-10-11 (JST)**:
[GitHub Actions run 38074903217](https://github.com/skyon0522-ai/rag-citation-evaluator/actions/runs/38074903217)
succeeded for commit `0885fb561277affbdc89986d77dc585b6f115961` on the earlier
fixture-only workflow. The new wrapper/workflow steps have not run on GitHub.
The historical remote result is not evidence for answer correctness, privacy
or production use. See [security limits and reporting](SECURITY.md).

## Linux report verification: 2026-10-15

The local `verify-portable.sh` run used Ubuntu 24.04 x86_64, Rust/Cargo 1.99.0
and Python 3.12.3. It built the locked binaries in a new target directory,
reusing a populated registry cache with Cargo offline. Later checks reused that
target directory. Dependency acquisition on a new machine was not exercised.
The [local receipt](verification/report-review-20261015.json) separates these
results from the unrun new GitHub workflow.

- 10 Unix boundary tests passed, including malformed or signaled engine output,
  existing paths, dangling symlinks, output collision, escaping and private modes.
- The original 22 direct fixtures and their 22 review bundles passed. All 22
  raw engine JSON outputs match the retained earlier results byte for byte.
- Unsafe answer text and three recorded examples passed: 26 report executions
  over 25 distinct inputs, including the rejected-quote fixture repeated as a
  recorded example. Every recorded bundle reproduced all six files exactly.
- The original engine, lockfile, fixtures and verifier match their 33 recorded
  hashes. The 518 copied Tantivy files match their existing source manifest.
  Appending one byte to the engine or copied license was rejected before output
  creation; both temporary changes were restored.
- A stale checksum entry for the protected manifest itself was removed. The 33
  actual file entries are unchanged; the manifest now states its scope and base
  commit instead of claiming equality to an unavailable private acceptance file.
- Both executables copied together ran outside the checkout, including paths
  with spaces and Japanese text. A missing sibling produced a contextual error
  and no bundle. Existing output was rejected without replacing its contents.
- Nine actual report variants passed isolated browser checks at widths 1440
  and 390, including quotation errors, invalid input, UTF-8 and unsafe text.
  Displayed answers, quotes, sources and decisions agreed with the JSON. Keyboard
  disclosure, horizontal table scrolling and local evidence links worked;
  no external requests or active injected elements were observed. Print and
  reduced-motion media checks passed. Desktop and mobile images were inspected.

The new layer presents the unchanged evaluator output; it does not change
retrieval, score semantic support or call a provider. The fictional purchase
policy and deliberately false answer are demonstrations, not customer evidence.
The report and input snapshots are ordinary editable files, not a signed audit
trail. Browser checks do not establish comprehensive accessibility or security.

## What this does not verify

The CLI checks source identity, retrieval membership, version, text digest, quotation equality and span boundaries for these fixed inputs. It does not measure factuality, relevance, entailment, semantic citation support, per-claim citation completeness, Japanese retrieval quality, security, production suitability or customer value. It runs no NLI model, ALCE or Ragas benchmark. The in-memory index and synthetic fixtures do not establish behavior for arbitrary corpora or workloads. Full upstream Tantivy tests and fresh-machine dependency acquisition were not run in this curation.
