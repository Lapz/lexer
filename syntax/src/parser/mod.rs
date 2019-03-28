mod lexer;

use crate::ast::*;
use crate::token::{Token, TokenType};
use rand::{self, Rng};
use std::collections::VecDeque;
use util::emmiter::Reporter;
use util::pos::{CharPosition, Position, Span, Spanned, EMPTYSPAN};
use util::symbol::{Symbol, Symbols};

pub type ParserResult<T> = Result<T, ()>;

pub struct Parser<'a> {
    /// The input string
    input: &'a str,
    chars: CharPosition<'a>,
    /// The reporter that collects all the erros
    reporter: Reporter,
    /// The symbols table that will be built up during parsing
    symbols: &'a mut Symbols<()>,
    /// The character ahead
    lookahead: Option<(Position, char)>,

    past_tokens: VecDeque<Spanned<Token<'a>>>,
    /// The very first character
    start: Position,
    /// The last charact
    end: Position,
    /// Flag that manages whetere we are in a cond or class instance
    parsing_cond: bool,
    /// Flag that manages whetere we are in a match_arm
    parsing_match_arm: bool,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str, reporter: Reporter, symbols: &'a mut Symbols<()>) -> Self {
        let mut chars = CharPosition::new(input);
        let end = chars.pos;
        let mut past_tokens = VecDeque::new();

        let mut parser = Self {
            input,
            end,
            start: end,
            reporter,
            lookahead: chars.next(),
            past_tokens: VecDeque::new(),
            chars,
            symbols,
            parsing_cond: false,
            parsing_match_arm: false,
        };

        past_tokens.push_back(parser.next().unwrap());
        parser.past_tokens = past_tokens;

        parser
    }

    pub fn parse(&mut self) -> ParserResult<Program> {
        let mut program = Program {
            classes: Vec::new(),
            functions: Vec::new(),
            aliases: Vec::new(),
            enums: Vec::new(),
        };

        let mut had_error = false;

        while self.peek(|token| token != '\0') {
            if self.recognise(TokenType::FUNCTION) {
                match self.parse_function("function") {
                    Ok(function) => program.functions.push(function),
                    Err(_) => {
                        had_error = true;

                        self.synchronize()?;
                    }
                }
            } else if self.recognise(TokenType::CLASS) {
                match self.parse_class_declaration() {
                    Ok(class) => program.classes.push(class),
                    Err(_) => {
                        had_error = true;
                        self.synchronize()?;
                    }
                }
            } else if self.recognise(TokenType::ENUM) {
                match self.parse_enum() {
                    Ok(sum) => program.enums.push(sum),
                    Err(_) => {
                        had_error = true;
                        self.synchronize()?;
                    }
                }
            } else if self.recognise(TokenType::TYPE) {
                match self.parse_type_alias() {
                    Ok(alias) => program.aliases.push(alias),
                    Err(_) => {
                        had_error = true;
                        self.synchronize()?;
                    }
                }
            } else {
                self.synchronize()?;
                had_error = true;
            }
        }

        if had_error {
            Err(())
        } else {
            Ok(program)
        }
    }

    pub fn synchronize(&mut self) -> ParserResult<()> {
        self.next()?;

        while self.peek(|token| token == '\0') {
            match self.next().map(|span| span.value.token) {
                Ok(TokenType::CLASS)
                | Ok(TokenType::FUNCTION)
                | Ok(TokenType::IDENTIFIER(_))
                | Ok(TokenType::FOR)
                | Ok(TokenType::IF)
                | Ok(TokenType::WHILE)
                | Ok(TokenType::LBRACE)
                | Ok(TokenType::RETURN) => break,
                Err(_) => unreachable!(),
                Ok(_) => self.next()?,
            };
        }

        Ok(())
    }

    /// Checks if any one of the given tokens is the next token
    fn matches(&mut self, tokens: Vec<TokenType>) -> bool {
        let mut found = false;

        for token in tokens {
            if self.recognise(token) {
                found = true;
            }
        }

        found
    }

    pub fn consume(&mut self, token_to_check: &TokenType<'a>, msg: &str) -> ParserResult<()> {
        match self.next() {
            Ok(Spanned {
                ref span,
                value: Token { ref token },
            }) => {
                if token == token_to_check {
                    return Ok(());
                }

                let msg = format!("{} but instead found `{}`", msg, token);

                self.span_error(msg, *span);

                Err(())
            }

            Err(_) => Err(()),
        }
    }

    fn consume_get_span(
        &mut self,
        token_to_check: &TokenType<'a>,
        msg: &str,
    ) -> ParserResult<Span> {
        match self.next() {
            Ok(Spanned {
                value: Token { ref token },
                ref span,
            }) => {
                if token == token_to_check {
                    return Ok(*span);
                }

                let msg = format!("{} but instead found `{}`", msg, token);

                self.span_error(msg, *span);

                Err(())
            }
            Err(_) => Err(()),
        }
    }

    pub fn consume_get_symbol(&mut self, msg: &str) -> ParserResult<Spanned<Symbol>> {
        match self.next() {
            Ok(Spanned {
                value:
                    Token {
                        token: TokenType::IDENTIFIER(ident),
                    },
                ref span,
            }) => Ok(Spanned {
                span: *span,
                value: self.symbols.symbol(ident),
            }),
            Ok(Spanned {
                value: Token { ref token },
                ref span,
            }) => {
                let msg = format!("{} but instead found `{}`", msg, token);

                self.span_error(msg, *span);

                Err(())
            }
            Err(_) => Err(()),
        }
    }

    fn consume_get_symbol_and_span(&mut self, msg: &str) -> ParserResult<(Span, Spanned<Symbol>)> {
        match self.next() {
            Ok(Spanned {
                value:
                    Token {
                        token: TokenType::IDENTIFIER(ident),
                    },
                ref span,
            }) => Ok((
                *span,
                Spanned {
                    span: *span,
                    value: self.symbols.symbol(ident),
                },
            )),
            Ok(Spanned {
                value: Token { ref token },
                ref span,
            }) => {
                let msg = format!("{} but instead found `{}`", msg, token);

                self.span_error(msg, *span);

                Err(())
            }
            Err(()) => Err(()),
        }
    }

    fn random_ident(&mut self) -> Symbol {
        let mut rng = rand::thread_rng();
        let letter: char = rng.gen_range(b'A', b'Z') as char;
        let number: u32 = rng.gen_range(0, 999_999);
        let s = format!("{}{:06}", letter, number);

        self.symbols.symbol(&s)
    }

    fn recognise(&mut self, expected: TokenType) -> bool {
        match self.past_tokens.back() {
            Some(token) => token.value.token == expected,
            None => false,
        }
        // Ok(self.next()?.value.token == token)
    }

    fn get_unary_op(&mut self) -> ParserResult<Spanned<UnaryOp>> {
        get_unary_op!(self,{
            BANG => Bang,
            MINUS => Minus
        })
    }

    fn get_binary_op(&mut self) -> ParserResult<Spanned<Op>> {
        get_op!(self, {
            AND => And,
            OR => Or,
            GREATERTHAN =>  GreaterThan,
            LESSTHAN => LessThan,
            GREATERTHANEQUAL => GreaterThanEqual,
            LESSTHANEQUAL => LessThanEqual,
            PLUS => Plus,
            MINUS => Minus,
            STAR => Star,
            SLASH => Slash,
            EQUALEQUAL => EqualEqual,
            MODULO => Modulo,
            EXPONENTIAL => Exponential,
            BANGEQUAL => BangEqual
        })
    }

    fn get_assign_op(&mut self) -> ParserResult<Spanned<AssignOperator>> {
        get_assign_op!(self,{
            ASSIGN => Equal,
            PLUSASSIGN => PlusEqual,
            MINUSASSIGN => MinusEqual,
            STARASSIGN => StarEqual,
            SLASHASSIGN => SlashEqual
        })
    }

    /// Parse an item name
    /// i.e.
    /// `Symbol <Vec<Symbol>>`
    /// fn Foo<T> | class Foo<T>
    fn parse_item_name(&mut self) -> ParserResult<Spanned<ItemName>> {
        let (open_span, name) = self.consume_get_symbol_and_span("Expected an identifier")?;

        let (type_params, end_span) = self.parse_generic_params()?;

        Ok(Spanned {
            span: open_span.to(end_span.unwrap_or(open_span)),
            value: ItemName { name, type_params },
        })
    }

    /// Parse generic params
    /// i.e.
    /// <T,T>
    /// <K,V>
    fn parse_generic_params(&mut self) -> ParserResult<(Vec<Spanned<Symbol>>, Option<Span>)> {
        if self.recognise(TokenType::LESSTHAN) {
            let open_span = self.consume_get_span(&TokenType::LESSTHAN, "Expected a '<' ")?;
            let mut generic_param = Vec::new();

            loop {
                generic_param.push(self.consume_get_symbol("Expected an identifier")?);
                if self.recognise(TokenType::COMMA) {
                    self.next()?;
                } else {
                    break;
                }
            }

            let close_span = self.consume_get_span(
                &TokenType::GREATERTHAN,
                "Expected a '>' to close generic params",
            )?;

            Ok((generic_param, Some(open_span.to(close_span))))
        } else {
            Ok((vec![], None))
        }
    }
}

/* ***********************
* Enums
*  ***********************
*/

impl<'a> Parser<'a> {
    fn parse_enum(&mut self) -> ParserResult<Spanned<Enum>> {
        let open_span = self.consume_get_span(&TokenType::ENUM, "expected 'enum'")?;

        let ident = self.parse_item_name()?;

        let mut constructors = Vec::new();

        self.consume(&TokenType::LBRACE, "Expected `{`")?;

        if !self.recognise(TokenType::RBRACE) {
            loop {
                let (mut whole_span,constructor) =
                    self.consume_get_symbol_and_span("Expected a constructor identifier")?;

                let mut args = Spanned::new(Vec::new(), EMPTYSPAN);

                if self.recognise(TokenType::LPAREN) {
                    args.span = self.consume_get_span(&TokenType::LPAREN, "Expected `(`")?;
                    loop {
                        args.value.push(self.parse_type()?);

                        if self.recognise(TokenType::COMMA) {
                            self.next()?;
                        } else {
                            break;
                        }
                    }

                    let span = self.consume_get_span(&TokenType::RPAREN, "Expected `)`")?;
                    whole_span = whole_span.to(span);
                    args.span = args.span.to(span);
                }

                constructors.push(Spanned::new(
                    EnumConstructor {
                        con: constructor,
                        args,
                    },
                    whole_span,
                ));

                if self.recognise(TokenType::COMMA) {
                    self.next()?;
                    continue;
                } else {
                    break;
                }
            }
        }

        let close_span = self.consume_get_span(&TokenType::RBRACE, "Expected `}`")?;

        Ok(Spanned {
            value: Enum {
                name: ident,
                constructors,
            },
            span: open_span.to(close_span),
        })
    }
}

/* ***********************
*  Type Aliases
*  ***********************
*/
impl<'a> Parser<'a> {
    /// Parses a type alias of the form `type NAME = TYPE;`
    fn parse_type_alias(&mut self) -> ParserResult<Spanned<TypeAlias>> {
        let open_span = self.consume_get_span(&TokenType::TYPE, "Expected 'type' ")?;

        let alias = self.parse_item_name()?;

        self.consume(&TokenType::ASSIGN, "Expected '=' ")?;

        let ty = self.parse_type()?;

        let close_span = self.consume_get_span(&TokenType::SEMICOLON, "Expected ';' ")?;

        Ok(Spanned {
            span: open_span.to(close_span),
            value: TypeAlias { alias, ty },
        })
    }

    fn parse_type(&mut self) -> ParserResult<Spanned<Type>> {
        if self.recognise(TokenType::NIL) {
            Ok(Spanned {
                value: Type::Nil,
                span: self.consume_get_span(&TokenType::NIL, "Expected 'nil' ")?,
            })
        } else if self.recognise(TokenType::LBRACKET) {
            self.next()?;
            let ty = self.parse_type()?;
            Ok(Spanned {
                value: Type::Arr(Box::new(ty)),
                span: self.consume_get_span(&TokenType::RBRACKET, "Expected ']' ")?,
            })
        } else if self.recognise(TokenType::FUNCTION) {
            let open_span = self.consume_get_span(&TokenType::FUNCTION, "Expected 'fun' ")?;

            self.consume(&TokenType::LPAREN, "Expected a `(`")?;
            let mut param_ty = Vec::with_capacity(32);

            if !self.recognise(TokenType::RPAREN) {
                loop {
                    param_ty.push(self.parse_type()?);

                    if self.recognise(TokenType::COMMA) {
                        self.next()?;
                    } else {
                        break;
                    }
                }
            }

            let close_span = self.consume_get_span(&TokenType::RPAREN, "Expected  `)`")?;

            if self.recognise(TokenType::FRETURN) {
                self.next()?;
                let ty = self.parse_type()?;
                Ok(Spanned {
                    span: open_span.to(ty.get_span()),
                    value: Type::Func(param_ty, Some(Box::new(ty))),
                })
            } else {
                Ok(Spanned {
                    span: open_span.to(close_span),
                    value: Type::Func(param_ty, None),
                })
            }
        } else {
            let symbol = self.consume_get_symbol("Expected a type param")?;

            let mut types = vec![];

            if self.recognise(TokenType::LESSTHAN) {
                self.consume(&TokenType::LESSTHAN, "Expected '<' ")?;

                loop {
                    types.push(self.parse_type()?);
                    if self.recognise(TokenType::COMMA) {
                        self.next()?;
                    } else {
                        break;
                    }
                }

                Ok(Spanned {
                    span: symbol
                        .get_span()
                        .to(self.consume_get_span(&TokenType::GREATERTHAN, "Expected '>' ")?),
                    value: Type::Generic(symbol, types),
                })
            } else {
                Ok(Spanned {
                    span: symbol.get_span(),
                    value: Type::Simple(symbol),
                })
            }
        }
    }
}

/* ***********************
*  Functions
*  ***********************
*/
impl<'a> Parser<'a> {
    fn parse_function(&mut self, _kind: &str) -> ParserResult<Spanned<Function>> {
        let open_span = self.consume_get_span(&TokenType::FUNCTION, "Expected 'function' ")?;

        let name = self.parse_item_name()?;

        let param_span = self.consume_get_span(&TokenType::LPAREN, "Expected '(' ")?;

        let params = self.parse_params(param_span, "function")?;
        let mut returns = None;

        let rparen_span =
            self.consume_get_span(&TokenType::RPAREN, "Expected a ')' after function params")?;

        if self.recognise(TokenType::FRETURN) {
            self.next()?;

            returns = Some(self.parse_type()?);
        }

        let body = self.parse_block()?;

        Ok(Spanned {
            span: open_span.to(body.get_span()),
            value: Function {
                name,
                body,
                params: Spanned {
                    span: param_span.to(rparen_span),
                    value: params,
                },
                returns,
            },
        })
    }

    fn parse_params(
        &mut self,
        open_span: Span,
        kind: &str,
    ) -> ParserResult<Vec<Spanned<FunctionParam>>> {
        let mut params = Vec::with_capacity(32);

        if !self.recognise(TokenType::RPAREN) && !self.recognise(TokenType::BAR) {
            loop {
                if params.len() >= 32 {
                    self.span_error("Too many params", open_span);
                    break;
                };

                let msg = format!("Expected a {} type name", kind);

                let (open_span, name) = self.consume_get_symbol_and_span(&msg)?;

                self.consume(&TokenType::COLON, "Expected a colon")?;

                let ty = self.parse_type()?;

                params.push(Spanned {
                    span: open_span.to(ty.get_span()),
                    value: FunctionParam { name, ty },
                });

                if self.recognise(TokenType::COMMA) {
                    self.next()?;
                } else {
                    break;
                }
            }
        }

        Ok(params)
    }

    /* ******************
     *
     * STATEMENT PARSERS
     *
     * ***************** */
    pub fn parse_statement(&mut self) -> ParserResult<Spanned<Statement>> {
        if self.recognise(TokenType::LBRACE) {
            self.parse_block()
        } else if self.recognise(TokenType::LET) {
            self.parse_var_declaration()
        } else if self.recognise(TokenType::BREAK) {
            self.parse_break_statement()
        } else if self.recognise(TokenType::CONTINUE) {
            self.parse_continue_statement()
        } else if self.recognise(TokenType::RETURN) {
            self.parse_return_statement()
        } else if self.recognise(TokenType::IF) {
            self.parse_if_statement()
        } else if self.recognise(TokenType::DO) {
            self.parse_do_statement()
        } else if self.recognise(TokenType::WHILE) {
            self.parse_while_statement()
        } else if self.recognise(TokenType::FOR) {
            self.parse_for_statement()
        } else if self.recognise(TokenType::PRINT) {
            self.parse_print_statement()
        } else {
            self.parse_expression_statement()
        }
    }

    fn parse_var_declaration(&mut self) -> ParserResult<Spanned<Statement>> {
        let open_span = self.consume_get_span(&TokenType::LET, "Expected 'var' ")?;

        let ident = self.consume_get_symbol("Expected an IDENTIFIER after a 'var' ")?;

        let ty = if self.recognise(TokenType::COLON) {
            self.next()?;

            Some(self.parse_type()?)
        } else {
            None
        };

        let expr = if self.recognise(TokenType::SEMICOLON) {
            None
        } else {
            self.consume(&TokenType::ASSIGN, "Expected '='")?;
            Some(self.parse_expression()?)
        };

        let close_span = self.consume_get_span(&TokenType::SEMICOLON, "Expected ';'")?;

        Ok(Spanned {
            span: open_span.to(close_span),
            value: Statement::VarDeclaration { ident, ty, expr },
        })
    }

    fn parse_block(&mut self) -> ParserResult<Spanned<Statement>> {
        let open_span = self.consume_get_span(&TokenType::LBRACE, "Expected a '{' ")?;

        let mut statements = vec![];

        while !self.recognise(TokenType::RBRACE) {
            statements.push(self.parse_statement()?);
        }

        let close_span =
            self.consume_get_span(&TokenType::RBRACE, "Expected a `}` after block.")?;
        Ok(Spanned {
            span: open_span.to(close_span),
            value: Statement::Block(statements),
        })
    }

    fn parse_break_statement(&mut self) -> ParserResult<Spanned<Statement>> {
        self.consume_get_span(&TokenType::BREAK, "Expected a 'break' ")?;
        Ok(Spanned {
            value: Statement::Break,
            span: self.consume_get_span(&TokenType::SEMICOLON, "Expected ';' ")?,
        })
    }

    fn parse_continue_statement(&mut self) -> ParserResult<Spanned<Statement>> {
        self.consume_get_span(&TokenType::CONTINUE, "Expected 'continue' ")?;
        Ok(Spanned {
            value: Statement::Continue,
            span: self.consume_get_span(&TokenType::SEMICOLON, "Expected ';' ")?,
        })
    }

    pub fn parse_expression_statement(&mut self) -> ParserResult<Spanned<Statement>> {
        let expr = self.parse_expression()?;

        Ok(Spanned {
            span: if self.parsing_match_expression {
                expr.span
            } else {
                self.consume_get_span(&TokenType::SEMICOLON, "Expected ';' ")?
            },
            value: Statement::Expr(expr),
        })
    }

    fn parse_print_statement(&mut self) -> ParserResult<Spanned<Statement>> {
        let open_span = self.consume_get_span(&TokenType::PRINT, "Expected 'print' ")?;

        let expr = self.parse_expression()?;

        Ok(Spanned {
            span: open_span.to(self.consume_get_span(&TokenType::SEMICOLON, "Expected ';' ")?),
            value: Statement::Print(expr),
        })
    }

    fn parse_return_statement(&mut self) -> ParserResult<Spanned<Statement>> {
        let open_span = self.consume_get_span(&TokenType::RETURN, "Expected 'return' ")?;

        let expr = if self.recognise(TokenType::SEMICOLON) {
            let span = self.consume_get_span(&TokenType::SEMICOLON, "Expected ';' ")?;

            return Ok(Spanned {
                span: open_span.to(span),
                value: Statement::Return(Spanned {
                    span,
                    value: Expression::Literal(Literal::Nil),
                }),
            });
        } else {
            self.parse_expression()?
        };

        let close_span = self.consume_get_span(&TokenType::SEMICOLON, "Expected ';' ")?;

        Ok(Spanned {
            span: open_span.to(close_span),
            value: Statement::Return(expr),
        })
    }

    fn parse_while_statement(&mut self) -> ParserResult<Spanned<Statement>> {
        let open_span = self.consume_get_span(&TokenType::WHILE, "Expected 'while' ")?;

        self.parsing_cond = true;
        let cond = self.parse_expression()?;
        self.parsing_cond = false;

        let body = self.parse_statement()?;

        Ok(Spanned {
            span: open_span.to(body.get_span()),
            value: Statement::While {
                cond,
                body: Box::new(body),
            },
        })
    }

    fn parse_do_statement(&mut self) -> ParserResult<Spanned<Statement>> {
        let open_span = self.consume_get_span(&TokenType::DO, "Expected 'do' ")?;

        let body = self.parse_statement()?;

        self.consume(&TokenType::WHILE, "Expected while after 'do' condition")?;

        let cond = self.parse_expression()?;

        Ok(Spanned {
            span: open_span.to(body.get_span()),
            value: Statement::While {
                cond,
                body: Box::new(body),
            },
        })
    }

    fn parse_if_statement(&mut self) -> ParserResult<Spanned<Statement>> {
        let open_span = self.consume_get_span(&TokenType::IF, "Expected 'if' ")?;

        self.parsing_cond = true;
        let cond = self.parse_expression()?;
        self.parsing_cond = false;

        let then = Box::new(self.parse_statement()?);

        let (close_span, otherwise) = if self.recognise(TokenType::ELSE) {
            self.next()?;

            let otherwise = self.parse_statement()?;

            (Some(otherwise.get_span()), Some(Box::new(otherwise)))
        } else {
            (None, None)
        };

        Ok(Spanned {
            span: open_span.to(close_span.unwrap_or(open_span)),
            value: Statement::If {
                cond,
                then,
                otherwise,
            },
        })
    }

    fn parse_for_statement(&mut self) -> ParserResult<Spanned<Statement>> {
        let open_span = self.consume_get_span(&TokenType::FOR, "Expected 'for' ")?;

        self.consume(&TokenType::LPAREN, "Expected '(' after 'for'")?;

        let mut init = None;

        if self.recognise(TokenType::SEMICOLON) {
            self.next()?;
        } else if self.recognise(TokenType::LET) {
            init = Some(Box::new(self.parse_var_declaration()?));
        } else {
            init = Some(Box::new(self.parse_expression_statement()?));
        }

        let cond = if !self.recognise(TokenType::SEMICOLON) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(&TokenType::SEMICOLON, "Expected ';' after loop condition .")?;

        let incr = if !self.recognise(TokenType::RPAREN) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(&TokenType::RPAREN, "Expected ')' after for clauses.")?;

        let body = self.parse_statement()?;

        Ok(Spanned {
            span: open_span.to(body.get_span()),
            value: Statement::For {
                init,
                cond,
                incr,
                body: Box::new(body),
            },
        })
    }

    /* ******************
     *
     * EXPRESSION PARSERS
     *
     * ***************** */

    fn parse_expression(&mut self) -> ParserResult<Spanned<Expression>> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserResult<Spanned<Expression>> {
        let expr = self.parse_ternary()?;

        if self.matches(vec![
            TokenType::ASSIGN,
            TokenType::PLUSASSIGN,
            TokenType::MINUSASSIGN,
            TokenType::STARASSIGN,
            TokenType::SLASHASSIGN,
        ]) && !self.parsing_match_arm
        {
            let kind = self.get_assign_op()?;

            let value = self.assignment()?;

            match expr {
                Spanned {
                    span,
                    value: Expression::Var(var),
                } => {
                    return Ok(Spanned {
                        span: span.to(value.get_span()),
                        value: Expression::Assign {
                            name: var,
                            value: Box::new(value),
                            kind,
                        },
                    });
                }

                Spanned {
                    span,
                    value:
                        Expression::Get {
                            object, property, ..
                        },
                } => {
                    return Ok(Spanned {
                        span: span.to(value.get_span()),
                        value: Expression::Set {
                            object,
                            name: property,
                            value: Box::new(value),
                        },
                    });
                }

                Spanned { ref span, .. } => {
                    self.span_error("Not a valid assingment target", *span);
                    return Err(());
                }
            }
        }

        Ok(expr)
    }

    fn parse_ternary(&mut self) -> ParserResult<Spanned<Expression>> {
        let mut expr = self.parse_or()?;

        if self.recognise(TokenType::QUESTION) {
            self.next()?;

            let if_true = self.parse_expression()?;

            self.consume(&TokenType::COLON, "Expected ':'")?;

            let if_false = self.parse_expression()?;

            expr = Spanned {
                span: expr.get_span().to(if_false.get_span()),
                value: Expression::Ternary {
                    condition: Box::new(expr),
                    then_branch: Box::new(if_true),
                    else_branch: Box::new(if_false),
                },
            }
        };

        Ok(expr)
    }

    fn parse_or(&mut self) -> ParserResult<Spanned<Expression>> {
        let mut lhs = self.parse_and()?;

        use self::TokenType::*;

        binary!(self, OR, lhs, parse_and);

        Ok(lhs)
    }

    fn parse_and(&mut self) -> ParserResult<Spanned<Expression>> {
        let mut lhs = self.parse_equality()?;

        use self::TokenType::*;

        if self.recognise(TokenType::AS) {
            self.next()?;

            let ty = self.parse_type()?;

            Ok(Spanned {
                span: lhs.span.to(ty.span),
                value: Expression::Cast {
                    from: Box::new(lhs),
                    to: ty,
                },
            })
        } else {
            binary!(self, AND, lhs, parse_equality);
            Ok(lhs)
        }
    }

    fn parse_equality(&mut self) -> ParserResult<Spanned<Expression>> {
        let mut lhs = self.parse_comparison()?;

        binary!(
            self,
            vec![TokenType::BANGEQUAL, TokenType::EQUALEQUAL],
            lhs,
            parse_comparison
        );

        Ok(lhs)
    }

    fn parse_comparison(&mut self) -> ParserResult<Spanned<Expression>> {
        let mut lhs = self.parse_addition()?;

        binary!(
            self,
            vec![
                TokenType::LESSTHAN,
                TokenType::LESSTHANEQUAL,
                TokenType::GREATERTHAN,
                TokenType::GREATERTHANEQUAL,
            ],
            lhs,
            parse_addition
        );

        Ok(lhs)
    }

    fn parse_addition(&mut self) -> ParserResult<Spanned<Expression>> {
        let mut lhs = self.parse_multiplication()?;

        binary!(
            self,
            vec![
                TokenType::PLUS,
                TokenType::MINUS,
                TokenType::EXPONENTIAL,
                TokenType::MODULO
            ],
            lhs,
            parse_multiplication
        );

        Ok(lhs)
    }

    fn parse_multiplication(&mut self) -> ParserResult<Spanned<Expression>> {
        let mut lhs = self.parse_unary()?;

        binary!(
            self,
            vec![TokenType::SLASH, TokenType::STAR],
            lhs,
            parse_unary
        );

        Ok(lhs)
    }

    fn parse_unary(&mut self) -> ParserResult<Spanned<Expression>> {
        if self.recognise(TokenType::BANG) || self.recognise(TokenType::MINUS) {
            let op = self.get_unary_op()?;

            let right = self.parse_unary()?;

            return Ok(Spanned {
                span: op.get_span().to(right.get_span()),
                value: Expression::Unary {
                    op,
                    expr: Box::new(right),
                },
            });
        }

        self.call()
    }

    fn primary(&mut self) -> ParserResult<Spanned<Expression>> {
        match self.next() {
            Ok(Spanned {
                ref span,
                ref value,
            }) => match value.token {
                TokenType::TRUE(_) => Ok(Spanned {
                    span: *span,
                    value: Expression::Literal(Literal::True(true)),
                }),
                TokenType::NIL => Ok(Spanned {
                    span: *span,
                    value: Expression::Literal(Literal::Nil),
                }),
                TokenType::FALSE(_) => Ok(Spanned {
                    span: *span,
                    value: Expression::Literal(Literal::False(false)),
                }),
                TokenType::STRING(ref s) => Ok(Spanned {
                    span: *span,
                    value: Expression::Literal(Literal::Str(s.clone())),
                }),
                TokenType::INT(n) => Ok(Spanned {
                    span: *span,
                    value: Expression::Literal(Literal::Int(n)),
                }),
                TokenType::FLOAT(n) => Ok(Spanned {
                    span: *span,
                    value: Expression::Literal(Literal::Float(n)),
                }),

                TokenType::MATCH => self.parse_match(*span),

                TokenType::LPAREN => {
                    let expr = Box::new(self.parse_expression()?);

                    let close_span = self.consume_get_span(&TokenType::RPAREN, "Expected ')'")?;

                    Ok(Spanned {
                        span: span.to(close_span),
                        value: Expression::Grouping { expr },
                    })
                }

                TokenType::IDENTIFIER(ident) => {
                    let ident = Spanned {
                        value: self.symbols.symbol(ident),
                        span: *span,
                    };
                    self.parse_ident(ident)
                }

                TokenType::BAR => {
                    let closure = self.parse_closure(*span)?;

                    Ok(Spanned {
                        span: closure.get_span(),
                        value: Expression::Closure(Box::new(closure)),
                    })
                }

                TokenType::LBRACKET => {
                    let mut items = Vec::with_capacity(32);

                    if !self.recognise(TokenType::RBRACKET) {
                        loop {
                            if items.len() >= 32 {
                                break;
                            };

                            items.push(self.parse_expression()?);

                            if self.recognise(TokenType::COMMA) {
                                self.next()?;
                            } else {
                                break;
                            }
                        }
                    }

                    let close_span =
                        self.consume_get_span(&TokenType::RBRACKET, "Expected a closing `]`")?;

                    Ok(Spanned {
                        value: Expression::Array { items },
                        span: span.to(close_span),
                    })
                }

                ref other => {
                    let msg = format!("No rules expected '{}' ", other);

                    self.span_error(msg, *span);

                    Err(())
                }
            },
            Err(_) => Err(()), // TODO: ADD an error?
        }
    }

    fn parse_match(&mut self, start_span: Span) -> ParserResult<Spanned<Expression>> {
        self.parsing_cond = true;

        let cond = self.parse_expression()?;

        self.parsing_cond = false;

        let open_span = self.consume_get_span(&TokenType::LBRACE, "Expected `{` ")?;

        let mut arms = Vec::new();
        // counter for how many times we've seen the `_` pattern
        let mut seen_catch_all = 0;

        if !self.recognise(TokenType::RBRACE) {
            loop {
                self.parsing_match_expression= true;

                if self.recognise(TokenType::UNDERSCORE) {
                    seen_catch_all += 1;

                    let pattern = self.consume_get_span(&TokenType::UNDERSCORE, "Expected `_` ")?;

                    self.consume(&TokenType::MATCHARROW, "Expected `=>` ")?;

                    let body = self.parse_statement()?;

                    self.parsing_match_expression= false;

                    if seen_catch_all > 1 {
                        self.span_warn("`_` pattern is allready present", pattern.to(body.span));
                    }

                    let span = pattern.to(body.span);

                    arms.push(Spanned {
                        value: MatchArm {
                            pattern: None,
                            body,
                            is_all: true,
                        },
                        span,
                    });

                    if self.recognise(TokenType::COMMA) {
                        self.next()?;
                        continue;
                    } else {
                        break;
                    }
                }

                let pattern = self.parse_expression()?;

                self.consume(&TokenType::MATCHARROW, "Expected `=>` ")?;

                let body = self.parse_statement()?;

                self.parsing_match_expression= false;

                let span = pattern.span.to(body.span);

                arms.push(Spanned {
                    value: MatchArm {
                        pattern: Some(pattern),
                        body,
                        is_all: false,
                    },
                    span,
                });

                if self.recognise(TokenType::COMMA) {
                    self.next()?;
                } else {
                    break;
                }
            }
        }

        let close_span = self.consume_get_span(&TokenType::RBRACE, "Expected `}` ")?;

        Ok(Spanned {
            value: Expression::Match {
                cond: Box::new(cond),
                arms: Spanned::new(arms, open_span.to(close_span)),
            },
            span: start_span.to(close_span),
        })
    }

    fn parse_closure(&mut self, open_span: Span) -> ParserResult<Spanned<Function>> {
        let params = self.parse_params(open_span, "closure")?;

        let params_span = self.consume_get_span(&TokenType::BAR, "Expected `|` ")?;

        let returns = if self.recognise(TokenType::FRETURN) {
            self.next()?; // skip the ->
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(Spanned {
            span: open_span.to(body.get_span()),
            value: Function {
                name: Spanned {
                    span: open_span.to(body.get_span()),
                    value: ItemName {
                        name: Spanned {
                            span: params_span,
                            value: self.random_ident(),
                        },
                        type_params: vec![],
                    },
                },
                params: Spanned {
                    span: open_span.to(params_span),
                    value: params,
                },
                body,
                returns,
            },
        })
    }

    fn parse_generic_call(
        &mut self,
        expr: Spanned<Expression>,
    ) -> ParserResult<Spanned<Expression>> {
        let less_than_span = self.consume_get_span(&TokenType::LESSTHAN, "Expected `<` ")?;

        let mut types = vec![];

        if !self.recognise(TokenType::GREATERTHAN) {
            loop {
                types.push(self.parse_type()?);

                if self.recognise(TokenType::COMMA) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let greater_than_span = self.consume_get_span(&TokenType::GREATERTHAN, "Expected `>` ")?;

        if self.recognise(TokenType::LBRACE) {
            let ident = match expr.value {
                Expression::Var(ref s) => s.clone(),
                _ => unreachable!(),
            };

            let struct_lit = self.parse_ident(ident)?;

            match struct_lit {
                Spanned {
                    value:
                        Expression::ClassLiteral(Spanned {
                            value: ClassLiteral { symbol, props, .. },
                            ..
                        }),
                    span: end_span,
                } => Ok(Spanned {
                    value: Expression::ClassLiteral(Spanned {
                        span: expr.span.to(greater_than_span),
                        value: ClassLiteral {
                            types: Spanned {
                                span: less_than_span.to(greater_than_span),
                                value: types,
                            },
                            symbol,
                            props,
                        },
                    }),
                    span: expr.get_span().to(end_span),
                }),

                _ => unreachable!(),
            }
        } else {
            let whole_span = expr.span;
            let call = self.finish_call(expr)?;

            match call {
                        Spanned {
                            value:
                                Expression::Call(Spanned {
                                    value:
                                        Call {
                                            args,
                                            callee,
                                            .. // replace the old types with the new
                                        },
                                    ..
                                }),
                            ..
                        } => {
                            Ok(Spanned {
                                value: Expression::Call(Spanned {
                                    span: whole_span.to(call.span),
                                    value: Call {
                                        types: Spanned {
                                            span: less_than_span.to(greater_than_span),
                                            value: types,
                                        },
                                        callee,
                                        args,
                                    },
                                }),
                                span: { whole_span.to(call.span) },
                            })
                        }
                        _ => unreachable!(),
                    }
        }
    }

    fn call(&mut self) -> ParserResult<Spanned<Expression>> {
        let mut expr = self.primary()?;

        loop {
            if self.recognise(TokenType::NAMESPACE) {
                self.next()?; // Eat the ::

                if self.recognise(TokenType::LESSTHAN) {
                    return self.parse_generic_call(expr);
                } else {
                    let (mut span, enum_name) = match expr {
                        Spanned {
                            span,
                            value: Expression::Var(ref ident),
                        } => (span, ident.clone()),

                        Spanned { span, .. } => {
                            self.reporter.error("Expected an identifier", span);
                            return Err(());
                        }
                    };
                    let variant = self.consume_get_symbol("Expected an identifier")?;
                    let mut inner = None;

                    if self.recognise(TokenType::LPAREN) {
                        self.next()?; //eat the ::
                        inner = Some(Box::new(self.parse_expression()?));
                        span = span.to(self.consume_get_span(&TokenType::RPAREN, "Expected `(`")?);
                    }

                    return Ok(Spanned {
                        span,
                        value: Expression::Variant {
                            enum_name,
                            variant,
                            inner,
                        },
                    });
                }
            } else if self.recognise(TokenType::LPAREN) {
                expr = self.finish_call(expr)?;
            } else if self.recognise(TokenType::LBRACKET) {
                let open_span = self.consume_get_span(&TokenType::LBRACKET, "Expected '[' ")?;

                let index = Box::new(self.parse_expression()?);

                let close_span = self.consume_get_span(&TokenType::RBRACKET, "Expected ']' ")?;
                return Ok(Spanned {
                    span: open_span.to(close_span),
                    value: Expression::SubScript {
                        target: Box::new(expr),
                        index,
                    },
                });
            } else if self.recognise(TokenType::DOT) {
                self.next()?;

                let (close_span, property) =
                    self.consume_get_symbol_and_span("Expected an identifier")?;

                expr = Spanned {
                    span: expr.get_span().to(close_span),
                    value: Expression::Get {
                        object: Box::new(expr),
                        property,
                    },
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_ident(&mut self, symbol: Spanned<Symbol>) -> ParserResult<Spanned<Expression>> {
        if self.recognise(TokenType::LBRACE) && !self.parsing_cond {
            self.next()?;

            let mut props = vec![];

            if !self.recognise(TokenType::RBRACE) {
                loop {
                    let (open_span, symbol) =
                        self.consume_get_symbol_and_span("Expected a field name")?;

                    self.consume(&TokenType::COLON, "Expected a colon")?;

                    let expr = self.parse_expression()?;

                    props.push(Spanned {
                        span: open_span.to(symbol.get_span()),
                        value: ClassLiteralField { symbol, expr },
                    });

                    if self.recognise(TokenType::COMMA) {
                        self.next()?;
                    } else {
                        break;
                    }
                }
            }

            let close_span = self.consume_get_span(&TokenType::RBRACE, "Expected '}' ")?;
            // { symbol, props }
            Ok(Spanned {
                span: symbol.get_span().to(close_span),
                value: Expression::ClassLiteral(Spanned {
                    span: symbol.span.to(close_span),
                    value: ClassLiteral {
                        symbol,
                        props,
                        types: Spanned::new(Vec::new(), EMPTYSPAN),
                    },
                }),
            })
        } else {
            Ok(Spanned {
                span: symbol.get_span(),
                value: Expression::Var(symbol),
            })
        }
    }
    fn finish_call(&mut self, callee: Spanned<Expression>) -> ParserResult<Spanned<Expression>> {
        self.consume(&TokenType::LPAREN, "Expected '(' ")?;

        let mut args = vec![];

        if !self.recognise(TokenType::RPAREN) {
            loop {
                args.push(self.parse_expression()?);

                if self.recognise(TokenType::COMMA) {
                    self.next()?;
                } else {
                    break;
                }
            }
        }

        let close_span = self.consume_get_span(&TokenType::RPAREN, "Expected '(' ")?;

        Ok(Spanned {
            span: callee.get_span().to(close_span),
            value: Expression::Call(Spanned {
                span: callee.get_span().to(close_span),
                value: Call {
                    callee: Box::new(callee),
                    args,
                    types: Spanned::new(vec![], EMPTYSPAN),
                },
            }),
        })
    }
}

/* ***********************
*  Classes
*  ***********************
*/
impl<'a> Parser<'a> {
    fn parse_class_declaration(&mut self) -> ParserResult<Spanned<Class>> {
        let open_span = self.consume_get_span(&TokenType::CLASS, "Expected 'class' ")?;

        let name = self.parse_item_name()?;

        let superclass = if self.recognise(TokenType::EXTENDS) {
            self.next()?;
            Some(self.consume_get_symbol("Expected a superclass name")?)
        } else {
            None
        };

        let mut properties = vec![];
        let mut methods = vec![];

        self.consume(&TokenType::LBRACE, "Expected a '{' after class name")?;

        while !self.recognise(TokenType::RBRACE) {
            if !self.recognise(TokenType::FUNCTION) {
                loop {
                    let (open_span, name) =
                        self.consume_get_symbol_and_span("Expected a property name")?;

                    self.consume(&TokenType::COLON, "Expected ':'")?;

                    let ty = self.parse_type()?;

                    properties.push(Spanned {
                        span: open_span.to(ty.get_span()),
                        value: Field { name, ty },
                    });

                    if self.recognise(TokenType::COMMA) {
                        self.next()?;
                    } else {
                        break;
                    }
                }

                self.consume(
                    &TokenType::SEMICOLON,
                    "Expected a semicolon after declaring properties",
                )?;

                if self.recognise(TokenType::RBRACE) {
                    break;
                }
            }

            methods.push(self.parse_function("method")?);
        }

        let close_span = self.consume_get_span(&TokenType::RBRACE, "Expected '}' ")?;

        Ok(Spanned {
            span: open_span.to(close_span),
            value: Class {
                name,
                superclass,
                methods,
                fields: properties,
            },
        })
    }
}
