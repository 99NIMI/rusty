use crate::{
    ast::AstStatement::LiteralInteger,
    ast::*,
    parser::{parse, tests::lex},
    Diagnostic,
};

#[test]
fn illegal_literal_time_missing_segments_test() {
    let lexer = lex("
        PROGRAM exp 
            T#;
        END_PROGRAM
        ");
    let (_, diagnostics) = parse(lexer);
    assert_eq!(
        diagnostics,
        vec![Diagnostic::unexpected_token_found(
            "Literal",
            ";",
            SourceRange::new(36..37)
        )]
    );
}

#[test]
fn time_literal_problems_can_be_recovered_from_during_parsing() {
    let lexer = lex("
        PROGRAM exp 
            T#1d4d2h3m;
            x;
        END_PROGRAM
        ");
    let (cu, diagnostics) = parse(lexer);

    let actual_statements = cu.implementations[0].statements.len();
    assert_eq!(actual_statements, 2);
    assert_eq!(
        diagnostics,
        vec![Diagnostic::syntax_error(
            "Invalid TIME Literal: segments must be unique",
            (34..44).into()
        )]
    );
}

#[test]
fn illegal_literal_time_double_segments_test() {
    let lexer = lex("
        PROGRAM exp 
            T#1d4d2h3m;
        END_PROGRAM
        ");

    let (_, diagnostics) = parse(lexer);
    assert_eq!(
        diagnostics[0],
        Diagnostic::syntax_error(
            "Invalid TIME Literal: segments must be unique",
            SourceRange::new(34..44)
        )
    );
}

#[test]
fn illegal_literal_time_out_of_order_segments_test() {
    let lexer = lex("
        PROGRAM exp 
            T#1s2h3d;
        END_PROGRAM
        ");

    let (_, diagnostics) = parse(lexer);
    assert_eq!(
        diagnostics[0],
        Diagnostic::syntax_error(
            "Invalid TIME Literal: segments out of order, use d-h-m-s-ms",
            SourceRange::new(34..42),
        )
    );
}

#[test]
fn literal_hex_number_with_double_underscores() {
    let lexer = lex("PROGRAM exp 16#DEAD__beef; END_PROGRAM");
    let result = parse(lexer).1;

    assert_eq!(
        result.first().unwrap(),
        &Diagnostic::unexpected_token_found("KeywordSemicolon", "'__beef'", (19..25).into())
    );
}

#[test]
fn literal_dec_number_with_double_underscores() {
    let lexer = lex("PROGRAM exp 43__000; END_PROGRAM");
    let result = parse(lexer).1;

    assert_eq!(
        result.first().unwrap(),
        &Diagnostic::unexpected_token_found("KeywordSemicolon", "'__000'", (14..19).into())
    );
}

#[test]
fn literal_bin_number_with_double_underscores() {
    let lexer = lex("PROGRAM exp 2#01__001_101_01; END_PROGRAM");
    let result = parse(lexer).1;

    assert_eq!(
        result.first().unwrap(),
        &Diagnostic::unexpected_token_found("KeywordSemicolon", "'__001_101_01'", (16..28).into())
    );
}

#[test]
fn literal_oct_number_with_double_underscores() {
    let lexer = lex("PROGRAM exp 8#7__7; END_PROGRAM");
    let result = parse(lexer).1;

    assert_eq!(
        result.first().unwrap(),
        &Diagnostic::unexpected_token_found("KeywordSemicolon", "'__7'", (15..18).into())
    );
}

#[test]
fn string_with_round_parens_can_be_parsed() {
    let (result, diagnostics) = parse(lex(r#"
            TYPE MyString1 : STRING(253); END_TYPE
            TYPE MyString2 : STRING[254) := 'abc'; END_TYPE
            TYPE MyString3 : STRING(255]; END_TYPE
            "#));

    assert_eq!(
        diagnostics,
        vec! [
            Diagnostic::ImprovementSuggestion {
                message: "Unusual type of parentheses around string size expression, consider using square parentheses '[]'"
                    .into(),
                range: SourceRange::new(37..41),
            },
            Diagnostic::ImprovementSuggestion {
                message: "Mismatched types of parentheses around string size expression".into(),
                range: SourceRange::new(88..92),
            },
            Diagnostic::ImprovementSuggestion {
                message: "Mismatched types of parentheses around string size expression".into(),
                range: SourceRange::new(148..152),
            }
        ]
    );

    let ast_string = format!("{:#?}", &result.types);

    let expected_ast = format!(
        "{:#?}",
        vec![
            UserTypeDeclaration {
                data_type: DataType::StringType {
                    name: Some("MyString1".to_string()),
                    size: Some(LiteralInteger {
                        value: 253,
                        location: (10..11).into(),
                        id: 0,
                    }),
                    is_wide: false,
                },
                initializer: None,
                location: (18..42).into(),
            },
            UserTypeDeclaration {
                data_type: DataType::StringType {
                    name: Some("MyString2".to_string()),
                    size: Some(LiteralInteger {
                        value: 254,
                        location: (10..11).into(),
                        id: 0,
                    }),
                    is_wide: false,
                },
                initializer: Some(AstStatement::LiteralString {
                    is_wide: false,
                    location: (69..102).into(),
                    value: "abc".into(),
                    id: 0,
                }),
                location: SourceRange::undefined(),
            },
            UserTypeDeclaration {
                data_type: DataType::StringType {
                    name: Some("MyString3".to_string()),
                    size: Some(LiteralInteger {
                        value: 255,
                        location: (10..11).into(),
                        id: 0,
                    }),
                    is_wide: false,
                },
                initializer: None,
                location: SourceRange::undefined(),
            }
        ]
    );

    assert_eq!(ast_string, expected_ast);
}

#[test]
fn literal_cast_with_space() {
    let lexer = lex("PROGRAM exp INT# 123; END_PROGRAM");
    let (_, diagnostics) = parse(lexer);

    assert_eq!(
        vec![Diagnostic::syntax_error(
            "Incomplete statement",
            (12..16).into()
        )],
        diagnostics
    );
}
