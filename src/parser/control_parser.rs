// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder
use crate::{
    ast::*,
    expect_token,
    lexer::Token::*,
    parser::{parse_any_in_region, parse_body_in_region},
    Diagnostic,
};

use super::ParseSession;
use super::{parse_expression, parse_reference, parse_statement};

pub fn parse_control_statement(lexer: &mut ParseSession) -> AstStatement {
    match lexer.token {
        KeywordIf => parse_if_statement(lexer),
        KeywordFor => parse_for_statement(lexer),
        KeywordWhile => parse_while_statement(lexer),
        KeywordRepeat => parse_repeat_statement(lexer),
        KeywordCase => parse_case_statement(lexer),
        KeywordReturn => parse_return_statement(lexer),
        KeywordContinue => parse_continue_statement(lexer),
        KeywordExit => parse_exit_statement(lexer),
        _ => parse_statement(lexer),
    }
}

fn parse_return_statement(lexer: &mut ParseSession) -> AstStatement {
    let location = lexer.location();
    lexer.advance();
    AstStatement::ReturnStatement {
        location,
        id: lexer.next_id(),
    }
}

fn parse_exit_statement(lexer: &mut ParseSession) -> AstStatement {
    let location = lexer.location();
    lexer.advance();
    AstStatement::ExitStatement {
        location,
        id: lexer.next_id(),
    }
}

fn parse_continue_statement(lexer: &mut ParseSession) -> AstStatement {
    let location = lexer.location();
    lexer.advance();
    AstStatement::ContinueStatement {
        location,
        id: lexer.next_id(),
    }
}

fn parse_if_statement(lexer: &mut ParseSession) -> AstStatement {
    let start = lexer.range().start;
    lexer.advance(); //If
    let mut conditional_blocks = vec![];

    while lexer.last_token == KeywordElseIf || lexer.last_token == KeywordIf {
        let condition = parse_expression(lexer);
        expect_token!(
            lexer,
            KeywordThen,
            AstStatement::EmptyStatement {
                location: lexer.location(),
                id: lexer.next_id(),
            }
        );
        lexer.advance();

        let condition_block = ConditionalBlock {
            condition: Box::new(condition),
            body: parse_body_in_region(lexer, vec![KeywordEndIf, KeywordElseIf, KeywordElse]),
        };

        conditional_blocks.push(condition_block);
    }

    let mut else_block = Vec::new();

    if lexer.last_token == KeywordElse {
        else_block.append(&mut parse_body_in_region(lexer, vec![KeywordEndIf]));
    }

    let end = lexer.last_range.end;

    AstStatement::IfStatement {
        blocks: conditional_blocks,
        else_block,
        location: SourceRange::new(start..end),
        id: lexer.next_id(),
    }
}

fn parse_for_statement(lexer: &mut ParseSession) -> AstStatement {
    let start = lexer.range().start;
    lexer.advance(); // FOR

    let counter_expression = parse_reference(lexer);
    expect_token!(
        lexer,
        KeywordAssignment,
        AstStatement::EmptyStatement {
            location: lexer.location(),
            id: lexer.next_id()
        }
    );
    lexer.advance();

    let start_expression = parse_expression(lexer);
    expect_token!(
        lexer,
        KeywordTo,
        AstStatement::EmptyStatement {
            location: lexer.location(),
            id: lexer.next_id(),
        }
    );
    lexer.advance();
    let end_expression = parse_expression(lexer);

    let step = if lexer.token == KeywordBy {
        lexer.advance(); // BY
        Some(Box::new(parse_expression(lexer)))
    } else {
        None
    };

    lexer.consume_or_report(KeywordDo); // DO

    AstStatement::ForLoopStatement {
        counter: Box::new(counter_expression),
        start: Box::new(start_expression),
        end: Box::new(end_expression),
        by_step: step,
        body: parse_body_in_region(lexer, vec![KeywordEndFor]),
        location: SourceRange::new(start..lexer.last_range.end),
        id: lexer.next_id(),
    }
}

fn parse_while_statement(lexer: &mut ParseSession) -> AstStatement {
    let start = lexer.range().start;
    lexer.advance(); //WHILE

    let condition = parse_expression(lexer);
    lexer.consume_or_report(KeywordDo);

    AstStatement::WhileLoopStatement {
        condition: Box::new(condition),
        body: parse_body_in_region(lexer, vec![KeywordEndWhile]),
        location: SourceRange::new(start..lexer.last_range.end),
        id: lexer.next_id(),
    }
}

fn parse_repeat_statement(lexer: &mut ParseSession) -> AstStatement {
    let start = lexer.range().start;
    lexer.advance(); //REPEAT

    let body = parse_body_in_region(lexer, vec![KeywordUntil, KeywordEndRepeat]); //UNTIL
    let condition = if lexer.last_token == KeywordUntil {
        parse_any_in_region(lexer, vec![KeywordEndRepeat], |lexer| {
            parse_expression(lexer)
        })
    } else {
        AstStatement::EmptyStatement {
            location: lexer.location(),
            id: lexer.next_id(),
        }
    };

    AstStatement::RepeatLoopStatement {
        condition: Box::new(condition),
        body,
        location: SourceRange::new(start..lexer.range().end),
        id: lexer.next_id(),
    }
}

fn parse_case_statement(lexer: &mut ParseSession) -> AstStatement {
    let start = lexer.range().start;
    lexer.advance(); // CASE

    let selector = Box::new(parse_expression(lexer));

    expect_token!(
        lexer,
        KeywordOf,
        AstStatement::EmptyStatement {
            location: lexer.location(),
            id: lexer.next_id()
        }
    );

    lexer.advance();

    let mut case_blocks = Vec::new();
    if lexer.token != KeywordEndCase && lexer.token != KeywordElse {
        let body = parse_body_in_region(lexer, vec![KeywordEndCase, KeywordElse]);

        let mut current_condition = None;
        let mut current_body = vec![];
        for statement in body {
            if let AstStatement::CaseCondition { condition, .. } = statement {
                if let Some(condition) = current_condition {
                    let block = ConditionalBlock {
                        condition,
                        body: current_body,
                    };
                    case_blocks.push(block);
                    current_body = vec![];
                }
                current_condition = Some(condition);
            } else {
                //If no current condition is available, log a diagnostic and add an empty condition
                if current_condition.is_none() {
                    lexer.accept_diagnostic(Diagnostic::syntax_error(
                        "Missing Case-Condition",
                        lexer.location(),
                    ));
                    current_condition = Some(Box::new(AstStatement::EmptyStatement {
                        location: lexer.location(),
                        id: lexer.next_id(),
                    }));
                }
                current_body.push(statement);
            }
        }
        if let Some(condition) = current_condition {
            let block = ConditionalBlock {
                condition,
                body: current_body,
            };
            case_blocks.push(block);
        }
    }

    let else_block = if lexer.last_token == KeywordElse {
        parse_body_in_region(lexer, vec![KeywordEndCase])
    } else {
        vec![]
    };

    let end = lexer.last_range.end;
    AstStatement::CaseStatement {
        selector,
        case_blocks,
        else_block,
        location: SourceRange::new(start..end),
        id: lexer.next_id(),
    }
}
