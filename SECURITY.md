# Security limits and reporting

This is an experimental, fixed-fixture CLI snapshot. It has not received a security audit, and no supported-release or security-response schedule is promised. The citation checks establish source and retrieval reference integrity for the tested inputs; they do not establish factual answer correctness, privacy, confidentiality, prompt-injection resistance or suitability for production use.

## Handling inputs and results

The CLI reads local corpus and request JSON into memory and builds an in-memory index. It has no authentication, access-control policy, input-size quota or sandbox. Only use data you are authorized to process, and isolate untrusted inputs with operating-system resource limits. Building dependencies may access the registry even though the CLI does not call a provider or start a network service.

Reports include query and answer text, source identifiers and diagnostic details. The fixture verifier also records stdout and stderr. Treat results according to the sensitivity of the input; do not publish private corpora, credentials or customer information in fixtures, CI artifacts, logs or issue reports. CI uploads only the receipt from the checked-in synthetic suite.

## Reporting a vulnerability

Use GitHub's **Report a vulnerability** option on the [repository security page](https://github.com/skyon0522-ai/rag-citation-evaluator/security) if private reporting is available. If it is unavailable, open an [issue](https://github.com/skyon0522-ai/rag-citation-evaluator/issues/new) only to request a private reporting channel; omit exploit details and sensitive data until that channel is agreed.

For a private report, include the affected commit, environment, a minimal synthetic reproducer and the observed impact. This policy does not confirm that private reporting is enabled or that a report has been sent.
