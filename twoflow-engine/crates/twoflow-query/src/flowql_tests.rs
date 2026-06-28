use crate::executor::Triple;
use crate::flowql::{execute_query_on_triples, parse_query, query_2flowql_json};
use crate::lexer::{TokenKind, lex_2flowql};
use crate::ql_ast::{ProjectionItem, Term};
use crate::ql_parser::QueryParseErrorKind;

const INPUT: &[u8] = b"Torre_Eiffel -> localizacao:Paris, altura:330, tipo:Monumento\nParis -> pais:Franca, continente:Europa\nAllasCode.Institute -> tipo:ResearchInstitute, foco:AgenticSystems\n";

fn triples() -> Vec<Triple> {
    vec![
        Triple {
            subject: "Torre_Eiffel".into(),
            predicate: "localizacao".into(),
            value: "Paris".into(),
            line: 1,
            pair_index: 0,
        },
        Triple {
            subject: "Torre_Eiffel".into(),
            predicate: "altura".into(),
            value: "330".into(),
            line: 1,
            pair_index: 1,
        },
        Triple {
            subject: "Torre_Eiffel".into(),
            predicate: "tipo".into(),
            value: "Monumento".into(),
            line: 1,
            pair_index: 2,
        },
        Triple {
            subject: "Paris".into(),
            predicate: "pais".into(),
            value: "Franca".into(),
            line: 2,
            pair_index: 0,
        },
        Triple {
            subject: "Paris".into(),
            predicate: "continente".into(),
            value: "Europa".into(),
            line: 2,
            pair_index: 1,
        },
        Triple {
            subject: "AllasCode.Institute".into(),
            predicate: "tipo".into(),
            value: "ResearchInstitute".into(),
            line: 3,
            pair_index: 0,
        },
        Triple {
            subject: "AllasCode.Institute".into(),
            predicate: "foco".into(),
            value: "AgenticSystems".into(),
            line: 3,
            pair_index: 1,
        },
    ]
}

#[test]
fn lexer_recognizes_core_symbols_and_literals() {
    let tokens =
        lex_2flowql("@2Flow.Store ? $monumento -> \"localizacao:principal -> x\" => value, *;")
            .unwrap();
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::At)));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Question)));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Arrow)));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::FatArrow)));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Dollar)));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Star)));
    assert!(tokens.iter().any(
        |t| matches!(t.kind, TokenKind::StringLiteral(ref s) if s.contains(':') && s.contains("->"))
    ));
}

#[test]
fn parser_simple_wildcard_predicate_value_path_variable_and_multi_pattern() {
    let simple = parse_query("@2Flow.Store ? Torre_Eiffel -> altura => value;").unwrap();
    assert_eq!(simple.store.name, "2Flow.Store");
    assert_eq!(simple.patterns.len(), 1);
    assert_eq!(simple.projection, vec![ProjectionItem::Value]);

    let wildcard = parse_query("@2Flow.Store ? Torre_Eiffel -> * => triples;").unwrap();
    assert_eq!(wildcard.patterns[0].edges[0].predicate, Term::Wildcard);

    let pred_value = parse_query("@2Flow.Store ? * -> pais:Franca => subject;").unwrap();
    assert_eq!(
        pred_value.patterns[0].edges[0].value,
        Some(Term::Identifier("Franca".into()))
    );

    let path = parse_query("@2Flow.Store ? Torre_Eiffel -> localizacao -> pais => value;").unwrap();
    assert_eq!(path.patterns[0].edges.len(), 2);

    let var =
        parse_query("@2Flow.Store ? $monumento -> localizacao:$cidade => $monumento, $cidade;")
            .unwrap();
    assert_eq!(var.projection.len(), 2);

    let multi = parse_query("@2Flow.Store ? $monumento -> localizacao:$cidade $cidade -> pais:Franca => $monumento, $cidade;").unwrap();
    assert_eq!(multi.patterns.len(), 2);
}

#[test]
fn parser_reports_required_syntax_errors() {
    assert_eq!(
        parse_query("2Flow.Store ? Torre_Eiffel -> altura => value;")
            .unwrap_err()
            .kind,
        QueryParseErrorKind::ExpectedStore
    );
    assert_eq!(
        parse_query("@2Flow.Store Torre_Eiffel -> altura => value;")
            .unwrap_err()
            .kind,
        QueryParseErrorKind::ExpectedQueryMarker
    );
    assert!(parse_query("@2Flow.Store ? Torre_Eiffel -> altura value;").is_err());
    assert_eq!(
        parse_query("@2Flow.Store ? Torre_Eiffel -> altura => value")
            .unwrap_err()
            .kind,
        QueryParseErrorKind::ExpectedSemicolon
    );
}

#[tokio::test]
async fn executor_simple_query_returns_330() {
    let result = execute_query_on_triples(
        &triples(),
        "@2Flow.Store ? Torre_Eiffel -> altura => value;",
    )
    .await
    .unwrap();
    assert_eq!(result.rows[0].get("value").unwrap(), "330");
}

#[tokio::test]
async fn executor_subject_triples_returns_three_rows() {
    let result =
        execute_query_on_triples(&triples(), "@2Flow.Store ? Torre_Eiffel -> * => triples;")
            .await
            .unwrap();
    assert_eq!(result.rows.len(), 3);
}

#[tokio::test]
async fn executor_predicate_value_returns_subject() {
    let result =
        execute_query_on_triples(&triples(), "@2Flow.Store ? * -> pais:Franca => subject;")
            .await
            .unwrap();
    assert_eq!(result.rows[0].get("subject").unwrap(), "Paris");
}

#[tokio::test]
async fn executor_path_returns_franca() {
    let result = execute_query_on_triples(
        &triples(),
        "@2Flow.Store ? Torre_Eiffel -> localizacao -> pais => value;",
    )
    .await
    .unwrap();
    assert_eq!(result.rows[0].get("value").unwrap(), "Franca");
}

#[tokio::test]
async fn executor_variables_and_multiple_patterns() {
    let one = execute_query_on_triples(&triples(), "@2Flow.Store ? $x -> tipo:Monumento => $x;")
        .await
        .unwrap();
    assert_eq!(one.rows[0].get("x").unwrap(), "Torre_Eiffel");

    let multi = execute_query_on_triples(&triples(), "@2Flow.Store ? $monumento -> localizacao:$cidade $cidade -> pais:Franca => $monumento, $cidade;").await.unwrap();
    assert_eq!(multi.rows[0].get("monumento").unwrap(), "Torre_Eiffel");
    assert_eq!(multi.rows[0].get("cidade").unwrap(), "Paris");
}

#[tokio::test]
async fn validator_rejects_unbound_variable_and_wildcard_has_no_binding() {
    let err = execute_query_on_triples(&triples(), "@2Flow.Store ? Torre_Eiffel -> altura => $x;")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("Unbound variable"));

    let result =
        execute_query_on_triples(&triples(), "@2Flow.Store ? * -> tipo:Monumento => subject;")
            .await
            .unwrap();
    assert_eq!(result.rows[0].get("subject").unwrap(), "Torre_Eiffel");
    assert!(!result.rows[0].contains_key("*"));
}

#[tokio::test]
async fn high_level_json_result_and_empty_result() {
    let json = query_2flowql_json(INPUT, "@2Flow.Store ? Torre_Eiffel -> altura => value;")
        .await
        .unwrap();
    assert_eq!(json, r#"[{"value":"330"}]"#);

    let empty = query_2flowql_json(
        INPUT,
        "@2Flow.Store ? Torre_Eiffel -> desconhecido => value;",
    )
    .await
    .unwrap();
    assert_eq!(empty, "[]");
}
