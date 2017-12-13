#[cfg(test)]
mod test {
    use lexer::Lexer;
    use parser::Parser;
    use ast::expr::*;
    use ast::statement::*;
    use pos::{Postition, WithPos};
    use symbol::{Symbol, Symbols};
    use types::Type;

    #[test]
    fn types() {
        let input = "var a:int = 10;";
        let tokens = Lexer::new(input).lex().unwrap();
        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let expected = vec![
            WithPos::new(
                Statement::Var(
                    Symbol(2),
                    WithPos::new(
                        Expression::Literal(Literal::Int(10)),
                        Postition {
                            line: 1,
                            column: 13,
                            absolute: 12,
                        },
                    ),
                    Some(Type::Int),
                ),
                Postition {
                    line: 1,
                    column: 1,
                    absolute: 0,
                },
            ),
        ];

        assert_eq!(expected, ast);
    }

    #[test]
    fn function_types() {
        let input = "fun add(a:int,b:int){ return a+b;}";
        let tokens = Lexer::new(input).lex().unwrap();
        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse();
        assert!(ast.is_ok())
    }

    #[test]
    fn function_return() {
        let input = "fun add(a:int,b:int) -> int { return a+b;}";
        let tokens = Lexer::new(input).lex().unwrap();
        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse();
        assert!(ast.is_ok())
    }

    #[test]
    fn do_while_statement() {
        let input = "do {print(10);} while (true)";
        let tokens = Lexer::new(input).lex().unwrap();
        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let condition = WithPos::new(
            Expression::Literal(Literal::True(true)),
            Postition {
                line: 1,
                column: 24,
                absolute: 23,
            },
        );

        let call_expr = WithPos::new(
            Expression::Var(Symbol(2), VariableUseHandle(0)),
            Postition {
                line: 1,
                column: 10,
                absolute: 9,
            },
        );

        let call = WithPos::new(
            Statement::ExpressionStmt(WithPos::new(
                Expression::Call {
                    callee: Box::new(call_expr),
                    arguments: vec![
                        WithPos::new(
                            Expression::Literal(Literal::Int(10)),
                            Postition {
                                line: 1,
                                column: 11,
                                absolute: 10,
                            },
                        ),
                    ],
                },
                Postition {
                    line: 1,
                    column: 13,
                    absolute: 12,
                },
            )),
            Postition {
                line: 1,
                column: 14,
                absolute: 13,
            },
        );

        let body = WithPos::new(
            Statement::Block(vec![call]),
            Postition {
                line: 1,
                column: 4,
                absolute: 3,
            },
        );

        let expected = WithPos::new(
            Statement::DoStmt {
                condition,
                body: Box::new(body),
            },
            Postition {
                line: 1,
                column: 1,
                absolute: 0,
            },
        );

        assert_eq!(ast, vec![expected]);
    }

    #[test]
    fn for_statement() {
        let input = "for (var i = 0; i < 2; i = i + 1)print(i);";
        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let init = WithPos {
            node: Statement::Var(
                Symbol(2),
                WithPos {
                    node: Expression::Literal(Literal::Int(0)),
                    pos: Postition {
                        line: 1,
                        column: 14,
                        absolute: 13,
                    },
                },
                None,
            ),
            pos: Postition {
                line: 1,
                column: 6,
                absolute: 5,
            },
        };

        let condition = WithPos::new(
            Expression::Binary {
                left_expr: Box::new(WithPos::new(
                    Expression::Var(Symbol(2), VariableUseHandle(0)),
                    Postition {
                        line: 1,
                        column: 17,
                        absolute: 16,
                    },
                )),
                operator: Operator::LessThan,
                right_expr: Box::new(WithPos::new(
                    Expression::Literal(Literal::Int(2)),
                    Postition {
                        line: 1,
                        column: 21,
                        absolute: 20,
                    },
                )),
            },
            Postition {
                line: 1,
                column: 19,
                absolute: 18,
            },
        );

        let call = WithPos::new(
            Statement::ExpressionStmt(WithPos::new(
                Expression::Call {
                    callee: Box::new(WithPos::new(
                        Expression::Var(Symbol(3), VariableUseHandle(4)),
                        Postition {
                            line: 1,
                            column: 39,
                            absolute: 38,
                        },
                    )),
                    arguments: vec![
                        WithPos::new(
                            Expression::Var(Symbol(2), VariableUseHandle(5)),
                            Postition {
                                line: 1,
                                column: 40,
                                absolute: 39,
                            },
                        ),
                    ],
                },
                Postition {
                    line: 1,
                    column: 41,
                    absolute: 40,
                },
            )),
            Postition {
                line: 1,
                column: 42,
                absolute: 41,
            },
        );

        let increment = WithPos::new(
            Statement::ExpressionStmt(WithPos::new(
                Expression::Assign {
                    handle: VariableUseHandle(3),
                    name: Symbol(2),
                    kind: AssignOperator::Equal,
                    value: Box::new(WithPos::new(
                        Expression::Binary {
                            left_expr: Box::new(WithPos::new(
                                Expression::Var(Symbol(2), VariableUseHandle(2)),
                                Postition {
                                    line: 1,
                                    column: 28,
                                    absolute: 27,
                                },
                            )),
                            operator: Operator::Plus,
                            right_expr: Box::new(WithPos::new(
                                Expression::Literal(Literal::Int(1)),
                                Postition {
                                    line: 1,
                                    column: 32,
                                    absolute: 31,
                                },
                            )),
                        },
                        Postition {
                            line: 1,
                            column: 30,
                            absolute: 29,
                        },
                    )),
                },
                Postition {
                    line: 1,
                    column: 26,
                    absolute: 25,
                },
            )),
            Postition {
                line: 1,
                column: 33,
                absolute: 32,
            },
        );

        let while_statement = WithPos::new(
            Statement::WhileStmt {
                condition,
                body: Box::new(WithPos::new(
                    Statement::Block(vec![call, increment]),
                    Postition {
                        line: 1,
                        column: 42,
                        absolute: 41,
                    },
                )),
            },
            Postition {
                line: 1,
                column: 42,
                absolute: 41,
            },
        );

        let expected = WithPos::new(
            Statement::Block(vec![init, while_statement]),
            Postition {
                line: 1,
                column: 1,
                absolute: 0,
            },
        );

        assert_eq!(ast, vec![expected]);
    }

    // #[test]
    fn class_statement() {
        let input = "class Foo {}";
        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let name = Symbol(2);

        let expected = WithPos::new(
            Statement::Class {
                name,
                methods: vec![],
            },
            Postition {
                line: 1,
                column: 1,
                absolute: 0,
            },
        );

        assert_eq!(ast, vec![expected]);
    }

    #[test]
    fn while_statement() {
        let input = "while (!true) {
            print(\"true\");
            break;continue;
            }";
        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let call = WithPos::new(
            Statement::ExpressionStmt(WithPos::new(
                Expression::Call {
                    callee: Box::new(WithPos::new(
                        Expression::Var(Symbol(2), VariableUseHandle(0)),
                        Postition {
                            line: 2,
                            column: 18,
                            absolute: 33,
                        },
                    )),
                    arguments: vec![
                        WithPos::new(
                            Expression::Literal(Literal::Str("true".to_owned())),
                            Postition {
                                line: 2,
                                column: 19,
                                absolute: 34,
                            },
                        ),
                    ],
                },
                Postition {
                    line: 2,
                    column: 25,
                    absolute: 40,
                },
            )),
            Postition {
                line: 2,
                column: 26,
                absolute: 41,
            },
        );

        let break_statement = WithPos::new(
            Statement::Break,
            Postition {
                line: 3,
                column: 13,
                absolute: 55,
            },
        );

        let continue_statement = WithPos::new(
            Statement::Continue,
            Postition {
                line: 3,
                column: 19,
                absolute: 61,
            },
        );

        let body = WithPos::new(
            Statement::Block(vec![call, break_statement, continue_statement]),
            Postition {
                line: 1,
                column: 15,
                absolute: 14,
            },
        );

        let expected = WithPos::new(
            Statement::WhileStmt {
                condition: WithPos::new(
                    Expression::Unary {
                        operator: UnaryOperator::Bang,
                        expr: Box::new(WithPos::new(
                            Expression::Literal(Literal::True(true)),
                            Postition {
                                line: 1,
                                column: 9,
                                absolute: 8,
                            },
                        )),
                    },
                    Postition {
                        line: 1,
                        column: 8,
                        absolute: 7,
                    },
                ),
                body: Box::new(body),
            },
            Postition {
                line: 1,
                column: 1,
                absolute: 0,
            },
        );

        assert_eq!(ast, vec![expected]);
    }

    #[test]
    fn if_stmt() {
        let input = "if (true) {}";

        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let expected = WithPos::new(
            Statement::IfStmt {
                condition: WithPos::new(
                    Expression::Literal(Literal::True(true)),
                    Postition {
                        line: 1,
                        column: 5,
                        absolute: 4,
                    },
                ),
                then_branch: Box::new(WithPos::new(
                    Statement::Block(vec![]),
                    Postition {
                        line: 1,
                        column: 11,
                        absolute: 10,
                    },
                )),
                else_branch: None,
            },
            Postition {
                line: 1,
                column: 1,
                absolute: 0,
            },
        );

        assert_eq!(ast, vec![expected]);
    }

    #[test]
    fn block() {
        let input = "{}";

        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let expected = WithPos::new(
            Statement::Block(vec![]),
            Postition {
                line: 1,
                column: 1,
                absolute: 0,
            },
        );

        assert_eq!(ast, vec![expected]);
    }

    #[test]
    fn array() {
        let input = "[10,12,13];";

        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let array = WithPos::new(
            Expression::Array {
                items: vec![
                    WithPos::new(
                        Expression::Literal(Literal::Int(10)),
                        Postition {
                            line: 1,
                            column: 2,
                            absolute: 1,
                        },
                    ),
                    WithPos::new(
                        Expression::Literal(Literal::Int(12)),
                        Postition {
                            line: 1,
                            column: 5,
                            absolute: 4,
                        },
                    ),
                    WithPos::new(
                        Expression::Literal(Literal::Int(13)),
                        Postition {
                            line: 1,
                            column: 8,
                            absolute: 7,
                        },
                    ),
                ],
            },
            Postition {
                line: 1,
                column: 1,
                absolute: 0,
            },
        );

        let expected = WithPos::new(
            Statement::ExpressionStmt(array),
            Postition {
                line: 1,
                column: 11,
                absolute: 10,
            },
        );

        assert_eq!(ast, vec![expected]);
    }

    #[test]
    fn indexing() {
        let input = "a[2+1];";

        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let index = WithPos::new(
            Expression::IndexExpr {
                target: Box::new(WithPos::new(
                    Expression::Var(Symbol(2), VariableUseHandle(0)),
                    Postition {
                        line: 1,
                        column: 1,
                        absolute: 0,
                    },
                )),
                index: Box::new(WithPos::new(
                    Expression::Binary {
                        left_expr: Box::new(WithPos::new(
                            Expression::Literal(Literal::Int(2)),
                            Postition {
                                line: 1,
                                column: 3,
                                absolute: 2,
                            },
                        )),
                        operator: Operator::Plus,
                        right_expr: Box::new(WithPos::new(
                            Expression::Literal(Literal::Int(1)),
                            Postition {
                                line: 1,
                                column: 5,
                                absolute: 4,
                            },
                        )),
                    },
                    Postition {
                        line: 1,
                        column: 4,
                        absolute: 3,
                    },
                )),
            },
            Postition {
                line: 1,
                column: 6,
                absolute: 5,
            },
        );

        let expected = WithPos::new(
            Statement::ExpressionStmt(index),
            Postition {
                line: 1,
                column: 7,
                absolute: 6,
            },
        );

        assert_eq!(ast, vec![expected]);
    }

    #[test]
    fn print() {
        let input = "print(9+9);";

        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let call = WithPos::new(
            Expression::Call {
                callee: Box::new(WithPos::new(
                    Expression::Var(Symbol(2), VariableUseHandle(0)),
                    Postition {
                        line: 1,
                        column: 6,
                        absolute: 5,
                    },
                )),
                arguments: vec![
                    WithPos::new(
                        Expression::Binary {
                            left_expr: Box::new(WithPos::new(
                                Expression::Literal(Literal::Int(9)),
                                Postition {
                                    line: 1,
                                    column: 7,
                                    absolute: 6,
                                },
                            )),
                            operator: Operator::Plus,
                            right_expr: Box::new(WithPos::new(
                                Expression::Literal(Literal::Int(9)),
                                Postition {
                                    line: 1,
                                    column: 9,
                                    absolute: 8,
                                },
                            )),
                        },
                        Postition {
                            line: 1,
                            column: 8,
                            absolute: 7,
                        },
                    ),
                ],
            },
            Postition {
                line: 1,
                column: 10,
                absolute: 9,
            },
        );

        let expected = WithPos::new(
            Statement::ExpressionStmt(call),
            Postition {
                line: 1,
                column: 11,
                absolute: 10,
            },
        );
        assert_eq!(ast, vec![expected]);
    }

    #[test]
    fn literal() {
        let input = "123;";

        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let expected = vec![
            WithPos::new(
                Statement::ExpressionStmt(WithPos::new(
                    Expression::Literal(Literal::Int(123)),
                    Postition {
                        line: 1,
                        column: 1,
                        absolute: 0,
                    },
                )),
                Postition {
                    line: 1,
                    column: 4,
                    absolute: 3,
                },
            ),
        ];

        assert_eq!(expected, ast);
    }

    #[test]
    fn unclosed_group() {
        let input = "(123";
        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse();
        assert!(ast.is_err());
    }

    #[test]
    fn unary_with_no_operand() {
        let input = "-<5";

        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse();

        assert!(ast.is_err());
    }

    #[test]
    fn brackets() {
        let input = "-123*(45.67);";

        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let expected = vec![
            WithPos::new(
                Statement::ExpressionStmt(WithPos::new(
                    Expression::Binary {
                        left_expr: Box::new(WithPos::new(
                            Expression::Unary {
                                operator: UnaryOperator::Minus,
                                expr: Box::new(WithPos::new(
                                    Expression::Literal(Literal::Int(123)),
                                    Postition {
                                        line: 1,
                                        column: 2,
                                        absolute: 1,
                                    },
                                )),
                            },
                            Postition {
                                line: 1,
                                column: 1,
                                absolute: 0,
                            },
                        )),
                        operator: Operator::Star,
                        right_expr: Box::new(WithPos::new(
                            Expression::Grouping {
                                expr: Box::new(WithPos::new(
                                    Expression::Literal(Literal::Float(45.67)),
                                    Postition {
                                        line: 1,
                                        column: 7,
                                        absolute: 6,
                                    },
                                )),
                            },
                            Postition {
                                line: 1,
                                column: 12,
                                absolute: 11,
                            },
                        )),
                    },
                    Postition {
                        line: 1,
                        column: 5,
                        absolute: 4,
                    },
                )),
                Postition {
                    line: 1,
                    column: 13,
                    absolute: 12,
                },
            ),
        ];

        assert_eq!(expected, ast);
    }

    #[test]
    fn precedence_add_mul() {
        let input = "123+456*789;";

        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let expected = vec![
            WithPos {
                node: Statement::ExpressionStmt(WithPos {
                    node: Expression::Binary {
                        left_expr: Box::new(WithPos {
                            node: Expression::Literal(Literal::Int(123)),
                            pos: Postition {
                                line: 1,
                                column: 1,
                                absolute: 0,
                            },
                        }),
                        operator: Operator::Plus,
                        right_expr: Box::new(WithPos {
                            node: Expression::Binary {
                                left_expr: Box::new(WithPos {
                                    node: Expression::Literal(Literal::Int(456)),
                                    pos: Postition {
                                        line: 1,
                                        column: 5,
                                        absolute: 4,
                                    },
                                }),
                                operator: Operator::Star,
                                right_expr: Box::new(WithPos {
                                    node: Expression::Literal(Literal::Int(789)),
                                    pos: Postition {
                                        line: 1,
                                        column: 9,
                                        absolute: 8,
                                    },
                                }),
                            },
                            pos: Postition {
                                line: 1,
                                column: 8,
                                absolute: 7,
                            },
                        }),
                    },
                    pos: Postition {
                        line: 1,
                        column: 4,
                        absolute: 3,
                    },
                }),
                pos: Postition {
                    line: 1,
                    column: 12,
                    absolute: 11,
                },
            },
        ];

        assert_eq!(expected, ast);
    }

    #[test]
    fn precedence_group() {
        let input = "123+(45.76*789-3);";

        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let expected = vec![
            WithPos::new(
                Statement::ExpressionStmt(Expression::Binary {
                    left_expr: Box::new(WithPos::new(Expression::Literal(Literal::Int(123)), Postition{

                    })),
                    operator: Operator::Plus,
                    right_expr: Box::new(WithPos::new(Expression::Grouping {
                        expr: Box::new(WithPos::new(Expression::Binary {
                            left_expr: Box::new(Expression::Binary {
                                left_expr: Box::new(WithPos::new(Expression::Literal(Literal::Float(45.76)), Postition{
                                    
                                })),
                                operator: Operator::Star,
                                right_expr: Box::new(WithPos::new(Expression::Literal(Literal::Int(789)), Postition{

                                })),
                            }),
                            operator: Operator::Minus,
                            right_expr: Box::new(WithPos::new(Expression::Literal(Literal::Int(3)),Postition{

                            })),
                        }, Postition{

                        })),
                    }, Postition{

                    })),
                }),
                Postition {
                    line: 1,
                    column: 18,
                    absolute: 17,
                },
            ),
        ];

        assert_eq!(expected, ast);
    }

    #[test]
    fn precedence_mul_add() {
        let input = "123*456+789;";

        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let expected = vec![
            WithPos::new(
                Statement::ExpressionStmt(WithPos::new(
                    Expression::Binary {
                        left_expr: Box::new(WithPos::new(
                            Expression::Binary {
                                left_expr: Box::new(WithPos::new(
                                    Expression::Literal(Literal::Int(123)),
                                    Postition {
                                        line: 1,
                                        column: 1,
                                        absolute: 0,
                                    },
                                )),
                                operator: Operator::Star,
                                right_expr: Box::new(WithPos::new(
                                    Expression::Literal(Literal::Int(456)),
                                    Postition {
                                        line: 1,
                                        column: 5,
                                        absolute: 4,
                                    },
                                )),
                            },
                            Postition {
                                line: 1,
                                column: 4,
                                absolute: 3,
                            },
                        )),
                        operator: Operator::Plus,
                        right_expr: Box::new(WithPos::new(
                            Expression::Literal(Literal::Int(789)),
                            Postition {
                                line: 1,
                                column: 9,
                                absolute: 8,
                            },
                        )),
                    },
                    Postition {
                        line: 1,
                        column: 8,
                        absolute: 7,
                    },
                )),
                Postition {
                    line: 1,
                    column: 12,
                    absolute: 11,
                },
            ),
        ];

        assert_eq!(expected, ast);
    }

    #[test]
    fn precedence_mul_mul() {
        let input = "123*456*789;";

        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let expected = vec![
            WithPos::new(
                Statement::ExpressionStmt(WithPos::new(
                    Expression::Binary {
                        left_expr: Box::new(WithPos::new(
                            Expression::Binary {
                                left_expr: Box::new(WithPos::new(
                                    Expression::Literal(Literal::Int(123)),
                                    Postition {
                                        line: 1,
                                        column: 1,
                                        absolute: 0,
                                    },
                                )),
                                operator: Operator::Star,
                                right_expr: Box::new(WithPos::new(
                                    Expression::Literal(Literal::Int(456)),
                                    Postition {
                                        line: 1,
                                        column: 5,
                                        absolute: 4,
                                    },
                                )),
                            },
                            Postition {
                                line: 1,
                                column: 4,
                                absolute: 3,
                            },
                        )),
                        operator: Operator::Star,
                        right_expr: Box::new(WithPos::new(
                            Expression::Literal(Literal::Int(789)),
                            Postition {
                                line: 1,
                                column: 9,
                                absolute: 8,
                            },
                        )),
                    },
                    Postition {
                        line: 1,
                        column: 8,
                        absolute: 7,
                    },
                )),
                Postition {
                    line: 1,
                    column: 12,
                    absolute: 11,
                },
            ),
        ];

        assert_eq!(expected, ast);
    }

    #[test]
    fn precedence_mul_add_unary() {
        let input = "-123*456+789;";
        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new();
        let ast = Parser::new(tokens, &mut symbols).parse().unwrap();

        let expected = vec![
            WithPos {
                node: Statement::ExpressionStmt(WithPos {
                    node: Expression::Binary {
                        left_expr: Box::new(WithPos {
                            node: Expression::Binary {
                                left_expr: Box::new(WithPos {
                                    node: Expression::Unary {
                                        operator: UnaryOperator::Minus,
                                        expr: Box::new(WithPos {
                                            node: Expression::Literal(Literal::Int(123)),
                                            pos: Postition {
                                                line: 1,
                                                column: 2,
                                                absolute: 1,
                                            },
                                        }),
                                    },
                                    pos: Postition {
                                        line: 1,
                                        column: 1,
                                        absolute: 0,
                                    },
                                }),
                                operator: Operator::Star,
                                right_expr: Box::new(WithPos {
                                    node: Expression::Literal(Literal::Int(456)),
                                    pos: Postition {
                                        line: 1,
                                        column: 6,
                                        absolute: 5,
                                    },
                                }),
                            },
                            pos: Postition {
                                line: 1,
                                column: 5,
                                absolute: 4,
                            },
                        }),
                        operator: Operator::Plus,
                        right_expr: Box::new(WithPos {
                            node: Expression::Literal(Literal::Int(789)),
                            pos: Postition {
                                line: 1,
                                column: 10,
                                absolute: 9,
                            },
                        }),
                    },
                    pos: Postition {
                        line: 1,
                        column: 9,
                        absolute: 8,
                    },
                }),
                pos: Postition {
                    line: 1,
                    column: 13,
                    absolute: 12,
                },
            },
        ];

        assert_eq!(expected, ast);
    }

}
