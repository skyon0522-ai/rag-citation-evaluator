# Research context

[Gao et al., “Enabling Large Language Models to Generate Text with Citations” (EMNLP 2023)](https://aclanthology.org/2023.emnlp-main.398/) treats answer correctness and citation support as separate evaluation questions. This package uses only that conceptual distinction to keep retrieval and reference-integrity checks separate from semantic support.

The package does not implement or reproduce ALCE's NLI method, datasets, scores, or benchmark. It performs no semantic or factuality evaluation and makes no claim about citation completeness or Japanese answer quality. Its structural acceptance can coexist with a false answer, as the fixed counterexample demonstrates. See [verification limits](VERIFICATION.md).
