# Parser evolution

This note records the current parsing strategy and the rationale for not yet introducing a full parser framework such as tree-sitter.

## Current state

The repository currently uses a lightweight input-analysis strategy based on:

- file extension detection
- language-sensitive keyword matching
- simple structural heuristics
- reproducible benchmark and regression tests

This approach is limited, but it has two advantages at the current stage:

1. low implementation complexity
2. high controllability during rapid benchmark iteration

## Why tree-sitter is not introduced yet

Tree-sitter would likely improve:

- language-aware structure extraction
- robustness of control-flow detection
- multi-language consistency
- future multi-file project analysis

However, introducing it now would also increase:

- dependency surface
- parser integration complexity
- test matrix size
- maintenance burden while the benchmark evidence layer is still maturing

## Current decision

At the current project stage, the correct decision is:

- continue strengthening benchmark evidence first
- continue improving reproducibility and report quality
- defer parser-framework adoption until the expected value is clearer

## Trigger conditions for future tree-sitter adoption

Tree-sitter should be reconsidered when at least one of these becomes a priority:

- better multi-file project understanding
- more rigorous language-aware control-flow extraction
- stronger Rust / TypeScript / JavaScript structural parsing
- a benchmark gap that cannot be reduced with the current heuristic layer

## Correct current claim

The project currently has:

- practical multi-language file handling
- lightweight structural signal extraction
- improved Rust / TypeScript classification support

The project does not yet have:

- AST-grade parsing
- tree-sitter-backed structure extraction
- language-complete semantic analysis