# Source provenance

## Tantivy source

The package contains the complete Tantivy 0.26.2 source workspace from commit `72d1ef9a6468aa68bbc69dcc80cdf60aaf64364d`, tree `0fedd3b9abba2dbf13e9e9094c66e3e62ef6e107`. The `components/tantivy` tree contains the 518 upstream blobs listed in [`source-copy-manifest.json`](source-copy-manifest.json). The original and copied trees were verified against that manifest by exact bytes, SHA-256 and Git blob identity. The source in `components/tantivy` was not modified for this package. The generated CLI lockfile is separate from those upstream blobs.

The CLI adapts the sequence in [`examples/basic_search.rs`](components/tantivy/examples/basic_search.rs): define stored/indexed fields, create an index writer, add documents, commit, open a reader and searcher, parse a query, collect `TopDocs`, then retrieve stored documents. Its direct Tantivy dependency is the local path `../components/tantivy`; the build record shows the copied path being compiled.

Task-specific code adds strict JSON fixture checks, a raw-corpus snapshot digest, source ID/version/text-digest checks, selected-result membership, and exact quotation checks over bounded UTF-8 byte spans. The fixed fixture matrix is in [`cli/fixtures/cases.json`](cli/fixtures/cases.json), and [`derivation.json`](derivation.json) records the adaptation and the public-query-field clarification.

## Licensing and authorship

The copied Tantivy source remains under its upstream MIT license in [`LICENSE`](LICENSE), with its upstream author list in [`AUTHORS`](AUTHORS). The CLI and authored documents use [`LICENSE.authored`](LICENSE.authored). These files identify different work and should remain together when the package is redistributed. See [`NOTICE`](NOTICE) for the attribution and dependency notice locations.

The CLI lockfile contains 148 packages: 138 registry packages and 10 local workspace packages. It retains 131 registry versions from the accepted baseline graph and adds seven digest-related registry packages. [`verification/dependency-resolution.json`](verification/dependency-resolution.json) records the lock hash, cache-reuse status, and retained license-file hashes. No fresh dependency acquisition was performed for the recorded public-stage run.
