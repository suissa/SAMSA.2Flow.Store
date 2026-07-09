# Architecture

## Layers

1. **Zig parser**: scans `.2flow` bytes line by line, validates delimiters with SIMD-assisted search, builds a lightweight AST, expands subject-grouped pairs to canonical triples, and writes accepted triples into Arrow-compatible column buffers.
2. **Zig columnar layout**: stores each Utf8 column as `offsets: [i32]` and concatenated `data: [u8]`. Nulls are not supported in the MVP.
3. **C ABI FFI**: exposes stable `extern struct` views over Zig-owned buffers and explicit `twoflow_free_batch` ownership release.
4. **Rust Arrow bridge**: reconstructs Rust Arrow arrays and `RecordBatch` values from the Zig FFI view.
5. **DataFusion provider**: exposes the batch as a `TableProvider` for SQL scans.
6. **Query API**: registers `.2flow` buffers as DataFusion tables and runs SQL.

## Parser Zig

The parser does not allocate temporary strings per token. It slices the input line, trims whitespace using slice views, validates required `->` and `:` delimiters, and appends only final token bytes to columnar buffers.

`findCharSIMD` uses `@Vector(16, u8)` comparisons and a safe lane loop instead of relying on a boolean-vector bit mask representation. This keeps the implementation portable across Zig development snapshots.

## Actors Zig

The distributed actor runtime is not implemented in the MVP. The intended layer is a Zig-native actor system inspired by BEAM/Erlang supervision and message passing, but without a VM or garbage collector. Actors will own parser shards, Arrow buffer producers, and IPC writers.

## FFI

The public ABI is C-compatible:

- `twoflow_parse_buffer(ptr, len) -> ?*CArrowRecordBatch2Flow`
- `twoflow_free_batch(batch)`

The returned pointer is stable until `twoflow_free_batch`. Zig never returns pointers to stack memory. The internal owner stores both the batch and the public C view.

## Arrow bridge

MVP currently uses CopyingFallback at the Arrow Rust boundary.
The Zig parser itself is zero-copy while slicing input and columnar while building Arrow-compatible buffers.
Full zero-copy Rust ownership will be implemented using Arrow C Data Interface / ArrowArray / ArrowSchema in the next milestone.

The copying fallback is intentional and honest: Arrow Rust arrays own their buffers safely, while Zig owns and later frees the FFI buffers. This avoids unsound lifetime tricks.

## DataFusion provider

`TwoFlowTableProvider` wraps one `RecordBatch`, supports projection by projecting the batch before building a `MemoryExec`, and applies a simple limit through `RecordBatch::slice`. Filter pushdown is marked unsupported, so DataFusion applies filters above the scan.

## Roadmap: distributed execution

- Add Zig actor runtime primitives: mailbox, scheduler, supervision tree, bounded channels, and failure semantics.
- Assign parser actors to file shards or network partitions.
- Emit Arrow IPC streams from parser actors to Rust query workers.
- Add backpressure between actors and query stages.

## Roadmap: Arrow IPC between processes

- Replace in-process FFI batch transfer with Arrow IPC streaming.
- Use Arrow C Data Interface for zero-copy handoff where ownership can be made explicit.
- Add schema negotiation and dictionary encoding for repeated subjects/predicates.

## Roadmap: SPARQL-like planner over 2flow

- Define a `.2flow` graph algebra instead of mapping directly to RDF.
- Add pattern matching and path expressions over `subject`, `predicate`, and `value` columns.
- Lower graph patterns into DataFusion logical plans.

## Roadmap: partitioning and dynamic sharding

- Partition by subject, predicate, domain-specific keys, or runtime heuristics.
- Track hot predicates and skew.
- Add adaptive re-sharding for long-running actor clusters.

## 2FlowQL layer

The Rust query crate now includes a lexer, parser, typed AST, semantic validator, in-memory executor and JSON result serializer for 2FlowQL. The public DSL avoids SQL/SPARQL/RDF syntax; DataFusion lowering remains an internal roadmap item.
