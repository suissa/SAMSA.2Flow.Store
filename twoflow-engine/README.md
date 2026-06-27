# TwoFlowEngine

TwoFlowEngine is a JVM-free conceptual fork of the SANSA/Spark graph stack. The MVP parses 2Flow-Triples `.2flow` documents with Zig, crosses a C ABI into Rust, builds Apache Arrow `RecordBatch` values, exports grouped/canonical JSON, and queries canonical triples with DataFusion.

## Required Zig

Target compiler:

```bash
zig 0.17.0-dev.305+bdfbf432d
```

Install that Zig build and ensure `zig` is on `PATH`.

## Build and test the Zig parser

```bash
cd zig/twoflow_parser
zig build
zig build test
```

The Zig library is named `twoflow_zigparser` and is installed under `zig/twoflow_parser/zig-out/lib`.

## Build and test Rust

From this directory:

```bash
cargo fmt --all
cargo test --workspace
```

The Rust build script invokes `zig build -Doptimize=ReleaseFast` so the FFI library is rebuilt before linking `twoflow-query`.

## `.2flow` example

```txt
Torre_Eiffel -> localizacao:Paris, altura:330, tipo:Monumento
Paris -> pais:Franca, continente:Europa
Usuario_123 -> intent:ComprarPizza, canal:WhatsApp
```

Each valid line becomes one Arrow row with non-null Utf8 columns:

```txt
subject, predicate, value
```

## SQL example

```sql
SELECT value
FROM graph
WHERE subject = 'Torre_Eiffel' AND predicate = 'altura'
```

Expected result:

```txt
330
```

## Non-goals in this MVP

- No JVM, Scala, or Spark internals.
- No RDF parser and no conversion from `.2flow` to RDF. See `FORMAT.md` for the formal 2Flow-Triples syntax.
- No JSON intermediate format.
- No distributed actor runtime yet; the actor layer is part of the architecture roadmap.
