# Goal

TwoFlowEngine starts a new graph execution path where `.2flow` is the primary logical format instead of RDF serialization.

## Why replace RDF as the primary format?

The `.2flow` format now uses 2Flow-Triples: compact subject-grouped relations that model semantic flows, operational graph relations, and directed domain edges directly. RDF can still be an interchange target in the future, but this project does not treat `.2flow` as serialized RDF and does not force every edge through RDF node/literal semantics.

## Why replace Spark/JVM with Zig + Rust?

Spark is powerful, but its JVM runtime, object model, and garbage collection impose costs that are undesirable for a low-level columnar graph engine. Zig is used for the parser and future actor runtime because it gives explicit memory control, C ABI exports, and predictable systems-level behavior. Rust is used for query execution because DataFusion and Arrow provide a mature columnar execution stack without a JVM.

## Why Arrow is the memory contract

Arrow defines language-neutral columnar memory. In the MVP, Zig builds Arrow-compatible StringArray buffers (`offsets` + `data`) and Rust reconstructs Arrow arrays. Arrow IPC and the Arrow C Data Interface are the planned contracts for zero-copy process and language boundaries.

## Why DataFusion first?

DataFusion gives an embeddable Rust SQL engine, optimizer, physical execution plans, and Arrow-native batches. It replaces the Spark SQL/DataFrame layer for the first query milestone while keeping the data model columnar.

## What this project is

- A JVM-free `.2flow` parser and query MVP.
- A Zig-to-Rust FFI integration point.
- An Arrow/DataFusion table provider for `.2flow` data.
- A foundation for distributed, actor-driven graph execution.

## What this project is not

- It is not a SANSA module.
- It is not a Scala/Spark extension.
- It is not an RDF parser.
- It is not a complete distributed graph engine yet.

## MVP limits

- `.2flow` supports `Subject -> Predicate:Value, Predicate:Value` subject-grouped lines.
- All values are Utf8; numeric/literal typing is intentionally not interpreted.
- Inline comments are not parsed.
- The parser exports grouped JSON, canonical triples JSON and Arrow-compatible batches; the Rust Arrow boundary currently uses a copying fallback.
- The actor runtime, partitioning, and Arrow IPC transport are roadmap items.
