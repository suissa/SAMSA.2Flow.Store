# 2FlowQL

2FlowQL is a graph-path query DSL for TwoFlowEngine.
It queries canonical 2Flow-Triples without exposing SQL, SPARQL, RDF or Turtle to the user.
The language follows the pattern:

```2flowql
@Store ? Pattern => Projection;
```

Patterns describe relation paths using the same directional `->` intuition as 2Flow-Triples.
The executor may lower 2FlowQL into DataFusion logical plans internally, but SQL is not part of the public language.

2FlowQL é uma DSL de consulta orientada a caminhos de grafo para o TwoFlowEngine.
Ela consulta 2Flow-Triples canônicos sem expor SQL, SPARQL, RDF ou Turtle ao usuário.
A linguagem segue o padrão:

```2flowql
@Store ? Pattern => Projection;
```

Os padrões descrevem caminhos relacionais usando a mesma intuição direcional `->` do 2Flow-Triples.
O executor pode compilar 2FlowQL internamente para planos lógicos do DataFusion, mas SQL não faz parte da linguagem pública.

## Why not SQL?

2FlowQL exposes graph-path relations directly. SQL may be used internally for DataFusion lowering later, but users write graph patterns rather than `SELECT/FROM/WHERE`.

## Why not SPARQL?

2FlowQL does not expose RDF terms, Turtle syntax, RDF literals, prefixes or SPARQL graph patterns. It is designed for canonical 2Flow-Triples.

## Symbols

| Symbol | Meaning |
| --- | --- |
| `@` | store selector |
| `?` | start pattern block |
| `->` | relation/path step |
| `:` | predicate value constraint |
| `*` | wildcard |
| `$` | variable |
| `=>` | projection |
| `,` | projection separator |
| `;` | query terminator |
| `'...'` / `"..."` | string literals |

## Examples

```2flowql
@2Flow.Store ? Torre_Eiffel -> altura => value;
@2Flow.Store ? Torre_Eiffel -> * => triples;
@2Flow.Store ? * -> pais:Franca => subject;
@2Flow.Store ? Torre_Eiffel -> localizacao -> pais => value;
@2Flow.Store ? $monumento -> localizacao:$cidade => $monumento, $cidade;
```

Multiple patterns join on shared variables:

```2flowql
@2Flow.Store ?
  $monumento -> localizacao:$cidade
  $cidade -> pais:Franca
=> $monumento, $cidade;
```

## Path semantics

A path chains the previous edge value into the next edge subject:

```2flowql
Torre_Eiffel -> localizacao -> pais
```

means:

```txt
Torre_Eiffel --localizacao--> Paris
Paris --pais--> Franca
```

## Variables and wildcards

Variables bind values and repeated variables must match the existing binding. Wildcards match any value and never create a binding.

## Projections

The MVP supports:

```txt
subject, predicate, value, triple, triples, $variable
```

`triples` returns canonical triple objects and cannot be combined with other projections in the MVP.

## MVP limitations

- In-memory Rust executor is the source of truth for now.
- DataFusion lowering is planned but not required for public APIs.
- Aggregation, ordering, grouping and mutation are not implemented.
- `triples` cannot be mixed with other projections.

## Roadmap

- Lower 2FlowQL AST to DataFusion logical plans.
- Add optional line/pair metadata projections.
- Add path optimizations and predicate indexes.
- Add distributed execution over Arrow IPC partitions.
