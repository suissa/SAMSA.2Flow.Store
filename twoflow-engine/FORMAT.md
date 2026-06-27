# 2Flow-Triples Format

2Flow-Triples is a compact subject-grouped relation notation.
It replaces RDF/Turtle/N-Triples as the primary textual input format in TwoFlowEngine, but it is not RDF and does not attempt to preserve RDF semantics by default.
Each source line declares one subject and one or more predicate:value pairs.
The parser expands those pairs into canonical triples for query execution, indexing, columnar storage and distributed processing.

2Flow-Triples é uma notação compacta de relações agrupadas por sujeito.
Ela substitui RDF/Turtle/N-Triples como formato textual primário dentro do TwoFlowEngine, mas não é RDF e não tenta preservar a semântica RDF por padrão.
Cada linha declara um sujeito e um ou mais pares predicado:valor.
O parser expande esses pares em triples canônicos para consulta, indexação, armazenamento colunar e processamento distribuído.

## Syntax

```2flow
Subject -> Predicate:Value, Predicate:Value, Predicate:Value
```

The first `->` separates the subject from the pair list. The first `:` inside each pair separates the predicate from the value. Commas split pairs only outside quoted strings.

## Why not RDF?

2Flow-Triples is a native operational graph notation. It does not require RDF IRIs, blank nodes, Turtle prefixes, RDF literals or statement terminators. Future RDF export can be an adapter, not the core semantic model.

## Why no final dot?

A newline terminates a statement. This keeps parsing simple and avoids Turtle-like punctuation that is not needed for the subject-grouped model.

## Why group predicates by subject?

Repeated subject text is noisy and inefficient for human-authored graph data. Grouping allows one subject declaration to expand into multiple canonical triples while preserving compact source form.

## Strings and comments

Quoted strings may contain commas, colons, `->`, `#`, spaces and escaped quotes. Inline comments are recognized only as whitespace followed by `#` outside a string.

## JSON grouped form

The grouped JSON form preserves one document entry per valid source line and stores each `predicate:value` as a pair with `value`, `raw_value`, `type_hint` and `pair_index`.

## Canonical triples JSON

The canonical JSON form expands each pair into `{ subject, predicate, value, line, pair_index }` for query planning, indexing and storage.

## Arrow representation

The MVP Arrow-compatible representation expands every pair to one row:

```txt
subject: Utf8
predicate: Utf8
value: Utf8
```

Subject values are repeated in the MVP. Dictionary encoding for subjects/predicates is a planned optimization.

## MVP limitations

- All values are strings internally.
- Type detection is a non-destructive `type_hint` only.
- No RDF conversion is performed.
- No null bitmap in the Zig Arrow-compatible StringArray layout.
- The Rust Arrow boundary currently uses a copying fallback for safe ownership.

## Roadmap

- Arrow C Data Interface zero-copy handoff.
- Dictionary encoding for repeated subjects/predicates.
- 2FlowQL graph planning on top of canonical triples.
- Distributed actor ingestion and Arrow IPC transport.
