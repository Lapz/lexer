mod test;
use token::{Token, TokenType};
use std::iter::Peekable;
use std::vec::IntoIter;
use pos::Postition;
use ast::expr::*;
use ast::statement::*;
use pos::WithPos;
use symbol::{Symbol, Symbols};
#[derive(Debug)]
pub struct Parser<'a> {
    tokens: Peekable<IntoIter<Token<'a>>>,
    loop_depth: i32,
    pub symbols: &'a mut Symbols<()>,
    variable_use_maker: VariableUseMaker,
}

use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Clone, Debug)]
pub enum ParserError {
    IllegalExpression(String),
    EOF,
    Expected(String),
    Break(String),
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ParserError::Expected(ref e)
            | ParserError::IllegalExpression(ref e)
            | ParserError::Break(ref e) => write!(f, "{}", e),
            ParserError::EOF => write!(f, "Unexpected end of file"),
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>, symbols: &'a mut Symbols<()>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
            symbols,
            loop_depth: 0,
            variable_use_maker: VariableUseMaker::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<WithPos<Statement>>, Vec<ParserError>> {
        let mut statements = vec![];

        let mut errors = vec![];

        while self.peek(|token| token != &TokenType::EOF) {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(e) => {
                    errors.push(e);
                    self.synchronize();
                }
            }
        }

        if errors.is_empty() {
            Ok(statements)
        } else {
            Err(errors)
        }
    }

    pub fn synchronize(&mut self) {
        self.advance();

        while self.peek(|token| token == &TokenType::EOF) {
            match self.advance().map(|t| t.token) {
                Some(TokenType::CLASS)
                | Some(TokenType::FUNCTION)
                | Some(TokenType::IDENTIFIER(_))
                | Some(TokenType::FOR)
                | Some(TokenType::IF)
                | Some(TokenType::WHILE)
                | Some(TokenType::RETURN) => break,
                None => unreachable!(),
                _ => self.advance(),
            };
        }
    }

    fn error(&self, message: &str, pos: Postition) -> String {
        format!("{} on {}", message, pos)
    }

    fn peek<F>(&mut self, mut check: F) -> bool
    where
        F: FnMut(&TokenType<'a>) -> bool,
    {
        self.tokens
            .peek()
            .map_or(false, |token| check(&token.token))
    }

    fn recognise(&mut self, token: TokenType<'a>) -> bool {
        if self.peek(|peeked| peeked == &token) {
            return true;
        }

        false
    }

    fn matched(&mut self, tokens: Vec<TokenType<'a>>) -> bool {
        let mut found = false;

        for token in tokens {
            if self.peek(|peeked| peeked == &token) {
                found = true;
            }
        }

        if found {
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> Option<Token<'a>> {
        self.tokens.next()
    }

    fn consume(&mut self, token_to_check: TokenType<'a>, msg: &str) -> Result<(), ParserError> {
        match self.advance() {
            Some(Token { ref token, ref pos }) => {
                if token == &token_to_check {
                    return Ok(());
                }

                Err(ParserError::Expected(self.error(msg, *pos)))
            }
            None => Err(ParserError::EOF),
        }
    }

    fn consume_get_pos(
        &mut self,
        token_to_check: TokenType<'a>,
        msg: &str,
    ) -> Result<Postition, ParserError> {
        match self.advance() {
            Some(Token { ref token, ref pos }) => {
                if token == &token_to_check {
                    return Ok(*pos);
                }

                Err(ParserError::Expected(self.error(msg, *pos)))
            }
            None => Err(ParserError::EOF),
        }
    }

    fn consume_name(&mut self, msg: &str) -> Result<Symbol, ParserError> {
        match self.advance() {
            Some(Token {
                token: TokenType::IDENTIFIER(ref ident),
                ..
            }) => Ok(self.symbols.symbol(ident)),
            Some(Token { ref pos, .. }) => Err(ParserError::Expected(self.error(msg, *pos))),
            None => Err(ParserError::EOF),
        }
    }

    fn consume_name_symbol(&mut self, msg: &str) -> Result<(Symbol, Postition), ParserError> {
        match self.advance() {
            Some(Token {
                token: TokenType::IDENTIFIER(ref ident),
                ref pos,
            }) => Ok((self.symbols.symbol(ident), *pos)),
            Some(Token { ref pos, .. }) => Err(ParserError::Expected(self.error(msg, *pos))),
            None => Err(ParserError::EOF),
        }
    }

    fn get_pos(&mut self) -> Result<Postition, ParserError> {
        match self.advance() {
            Some(Token { ref pos, .. }) => Ok(*pos),
            None => Err(ParserError::EOF),
        }
    }

    fn get_type(&mut self) -> Result<Option<Symbol>, ParserError> {
        if self.recognise(TokenType::COLON) {
            self.advance();

            let possilbe_type = self.advance().unwrap();

            match possilbe_type.token {
                TokenType::IDENTIFIER(ref ty) => return Ok(Some(self.symbols.symbol(ty))),
                TokenType::NIL => return Ok(Some(self.symbols.symbol("nil"))),
                _ => {
                    return Err(ParserError::Expected(self.error(
                        "Expected a proper type",
                        possilbe_type.pos,
                    )))
                }
            }
        }

        Ok(None)
    }
}

// Statements
impl<'a> Parser<'a> {
    fn declaration(&mut self) -> Result<WithPos<Statement>, ParserError> {
        if self.recognise(TokenType::VAR) {
            self.var_declaration()
        } else if self.recognise(TokenType::FUNCTION) {
            self.function("function")
        } else if self.recognise(TokenType::CLASS) {
            self.class_declaration()
        } else {
            self.statement()
        }
    }

    fn statement(&mut self) -> Result<WithPos<Statement>, ParserError> {
        if self.recognise(TokenType::LBRACE) {
            self.block()
        } else if self.recognise(TokenType::BREAK) {
            self.break_statement()
        } else if self.recognise(TokenType::CONTINUE) {
            self.continue_statement()
        } else if self.recognise(TokenType::RETURN) {
            self.return_statement()
        } else if self.recognise(TokenType::IF) {
            self.if_statement()
        } else if self.recognise(TokenType::DO) {
            self.do_statement()
        } else if self.recognise(TokenType::WHILE) {
            self.while_statement()
        } else if self.recognise(TokenType::FOR) {
            self.for_statement()
        } else {
            self.expression_statement()
        }
    }

    fn expression_statement(&mut self) -> Result<WithPos<Statement>, ParserError> {
        let expr = self.expression()?;

        let pos = self.consume_get_pos(TokenType::SEMICOLON, "Expected \';\' after value.")?;

        Ok(WithPos::new(Statement::ExpressionStmt(expr), pos))
    }

    fn function(&mut self, kind: &str) -> Result<WithPos<Statement>, ParserError> {
        let func_pos = self.get_pos()?;

        let name = self.consume_name(&format!("Expected a {} name", kind))?;
        Ok(WithPos::new(
            Statement::Function {
                name,
                body: self.fun_body(kind)?,
            },
            func_pos,
        ))
    }

    // Keyword statements

    fn break_statement(&mut self) -> Result<WithPos<Statement>, ParserError> {
        let break_pos = self.get_pos()?;

        if !(self.loop_depth >= 0) {
            let error = "Must be inside a loop to use break".to_owned();
            return Err(ParserError::Break(error));
        }

        self.consume(TokenType::SEMICOLON, "Expected ';' after 'break'")?;

        Ok(WithPos::new(Statement::Break, break_pos))
    }

    fn continue_statement(&mut self) -> Result<WithPos<Statement>, ParserError> {
        let cont_pos = self.get_pos()?;

        if !(self.loop_depth >= 0) {
            let error = "Must be inside a loop to use 'continue'".to_owned();

            return Err(ParserError::Break(error));
        }

        self.consume(TokenType::SEMICOLON, "Expected ';' after 'continue'")?;

        Ok(WithPos::new(Statement::Continue, cont_pos))
    }

    // Control Flow Statements

    fn for_statement(&mut self) -> Result<WithPos<Statement>, ParserError> {
        let for_pos = self.get_pos()?;
        self.consume(TokenType::LPAREN, "Expected '(' after 'for'")?;

        let mut initi = None;

        if self.recognise(TokenType::SEMICOLON) {
            self.advance();
        } else if self.recognise(TokenType::VAR) {
            initi = Some(self.var_declaration()?);
        } else {
            initi = Some(self.expression_statement()?);
        }

        let mut condition = None;

        if !self.recognise(TokenType::SEMICOLON) {
            condition = Some(self.expression()?);
        }

        self.consume(TokenType::SEMICOLON, "Expected ';' after loop condition .")?;

        let mut increment = None;

        if !self.recognise(TokenType::RPAREN) {
            increment = Some(self.expression()?);
        }

        let increment_pos =
            self.consume_get_pos(TokenType::RPAREN, "Expected ')' after for clauses.")?;

        self.loop_depth += 1;

        let mut body = self.statement()?;
        let body_pos = body.pos.clone();

        if increment != None {
            body = WithPos::new(
                Statement::Block(vec![
                    body,
                    WithPos::new(Statement::ExpressionStmt(increment.unwrap()), increment_pos),
                ]),
                body_pos,
            );
        } else if condition == None {
            condition = Some(WithPos::new(
                Expression::Literal(Literal::True(true)),
                body_pos,
            ));
        }

        body = WithPos::new(
            Statement::WhileStmt {
                condition: condition.unwrap(),
                body: Box::new(body),
            },
            body_pos,
        );

        if initi != None {
            let mut statements = vec![];

            statements.push(initi.unwrap());
            statements.push(body);

            body = WithPos::new(Statement::Block(statements), for_pos)
        }

        self.loop_depth -= 1;

        Ok(body)
    }

    fn do_statement(&mut self) -> Result<WithPos<Statement>, ParserError> {
        let do_pos = self.get_pos()?;

        let body = self.statement()?;

        self.consume(TokenType::WHILE, "Expected while after 'do' condition")?;

        self.consume(TokenType::LPAREN, "Expected '(' after while'")?;

        let condition = self.expression()?;

        self.consume(TokenType::RPAREN, "Expected ')' after 'while'")?;

        let do_statement = Statement::DoStmt {
            body: Box::new(body),
            condition,
        };

        Ok(WithPos::new(do_statement, do_pos))
    }

    fn while_statement(&mut self) -> Result<WithPos<Statement>, ParserError> {
        let while_pos = self.get_pos()?; // Eats the while;

        self.consume(TokenType::LPAREN, "Expected '(' after while'")?;

        let condition = self.expression()?;

        self.consume(TokenType::RPAREN, "Expected ')' after 'while'")?;

        self.loop_depth += 1;

        let body = self.statement()?;

        let while_st = Statement::WhileStmt {
            condition,
            body: Box::new(body),
        };

        self.loop_depth -= 1;

        Ok(WithPos::new(while_st, while_pos))
    }

    fn if_statement(&mut self) -> Result<WithPos<Statement>, ParserError> {
        let if_pos = self.get_pos()?; // Eats the if ;

        self.consume(TokenType::LPAREN, "Expected a \'(\' after \'if\'")?;

        let condition = self.expression()?;

        self.consume(TokenType::RPAREN, "Expected ')' after 'if'")?;

        let then_branch = Box::new(self.statement()?);
        let mut else_branch = None;

        if self.recognise(TokenType::ELSE) {
            else_branch = Some(Box::new(self.statement()?));

            return Ok(WithPos::new(
                Statement::IfStmt {
                    condition,
                    then_branch,
                    else_branch,
                },
                if_pos,
            ));
        }

        Ok(WithPos::new(
            Statement::IfStmt {
                condition,
                then_branch,
                else_branch,
            },
            if_pos,
        ))
    }

    fn return_statement(&mut self) -> Result<WithPos<Statement>, ParserError> {
        let pos = self.get_pos()?;

        let mut value = None;

        if !self.recognise(TokenType::COLON) {
            value = Some(self.expression()?);
        }

        self.consume(TokenType::SEMICOLON, "Expected a ")?;

        Ok(WithPos::new(Statement::Return(value), pos))
    }

    fn block(&mut self) -> Result<WithPos<Statement>, ParserError> {
        let pos = self.get_pos()?;

        let mut statement = vec![];

        while !self.recognise(TokenType::RBRACE) {
            statement.push(self.declaration()?);
        }

        self.consume(TokenType::RBRACE, "Expected a \'}\' after block.")?;

        Ok(WithPos::new(Statement::Block(statement), pos))
    }

    fn class_declaration(&mut self) -> Result<WithPos<Statement>, ParserError> {
        let class_pos = self.get_pos()?;
        let name = self.consume_name("Expected a class name")?;

        self.consume(TokenType::LBRACE, "Expect \'{ \' before class body")?;

        let mut methods = vec![];

        while !self.recognise(TokenType::RBRACE) {
            methods.push(self.function("method")?);
        }

        self.consume(TokenType::RBRACE, "Expect \'}\'' after class body")?;

        Ok(WithPos::new(Statement::Class { methods, name }, class_pos))
    }

    fn var_declaration(&mut self) -> Result<WithPos<Statement>, ParserError> {
        let var_pos = self.get_pos()?;
        let name = self.consume_name("Expected an IDENTIFIER after a \'var\' ")?;

        if self.recognise(TokenType::SEMICOLON) {
            let pos = self.consume_get_pos(TokenType::SEMICOLON, "Expected a ';'")?;

            let value = WithPos::new(Expression::Literal(Literal::Nil), pos);

            return Ok(WithPos::new(Statement::Var(name, value, None), var_pos));
        }

        let var_type = self.get_type()?;

        if self.matched(vec![
            TokenType::ASSIGN,
            TokenType::MINUSASSIGN,
            TokenType::PLUSASSIGN,
            TokenType::SLASHASSIGN,
            TokenType::STARASSIGN,
        ]) {
            self.advance();
            let expr = self.expression()?;
            self.consume(
                TokenType::SEMICOLON,
                "Expect \';\' after variable decleration.",
            )?;
            return Ok(WithPos::new(Statement::Var(name, expr, var_type), var_pos));
        }

        Err(ParserError::Expected(self.error(
            "Expected an assignment",
            var_pos,
        )))
    }
}

// Expression Parsing
impl<'a> Parser<'a> {
    fn expression(&mut self) -> Result<WithPos<Expression>, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<WithPos<Expression>, ParserError> {
        let expr = self.ternary()?;

        if self.matched(vec![
            TokenType::ASSIGN,
            TokenType::PLUSASSIGN,
            TokenType::MINUSASSIGN,
            TokenType::STARASSIGN,
            TokenType::SLASHASSIGN,
        ]) {
            let next = self.advance().unwrap();
            let kind = get_assign_operator(next.token);

            let value = self.assignment()?;

            match expr.node {
                Expression::Var(name, _) => {
                    return Ok(WithPos::new(
                        Expression::Assign {
                            handle: self.variable_use_maker.next(),
                            name,
                            value: Box::new(value),
                            kind,
                        },
                        next.pos,
                    ))
                },

                Expression::Get(ref get) => return Ok(Expression::Set{
                        object: get.object.clone(),
                        name: get.name.clone(),
                        value,
                        handle: self.variable_use_maker.next(),
                }),
                _ => {
                    return Err(ParserError::IllegalExpression(
                        "Error at '=': Invalid assignment target.".to_owned(),
                    ))
                }
            }
        }

        Ok(expr)
    }

    fn ternary(&mut self) -> Result<WithPos<Expression>, ParserError> {
        let mut condition = self.or()?;

        while self.matched(vec![TokenType::QUESTION]) {
            let ternary_pos = self.consume_get_pos(TokenType::QUESTION, "Expected a '?'")?;

            let then_branch = Box::new(self.expression()?);

            self.consume(
                TokenType::COLON,
                "Expected ':' after lhs ternary condition.",
            )?;

            let else_branch = Box::new(self.ternary()?);

            condition = WithPos::new(
                Expression::Ternary {
                    condition: Box::new(condition),
                    then_branch,
                    else_branch,
                },
                ternary_pos,
            )
        }

        Ok(condition)
    }

    fn or(&mut self) -> Result<WithPos<Expression>, ParserError> {
        let mut expr = self.and()?;

        while self.recognise(TokenType::OR) {
            let next = self.advance().unwrap();

            let operator = get_logic_operator(next.token);

            let right = Box::new(self.and()?);

            expr = WithPos::new(
                Expression::Logical {
                    left: Box::new(expr),
                    operator,
                    right,
                },
                next.pos,
            )
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<WithPos<Expression>, ParserError> {
        let mut expr = self.equality()?;

        while self.recognise(TokenType::AND) {
            let next = self.advance().unwrap();

            let operator = get_logic_operator(next.token);

            let right = Box::new(self.equality()?);

            expr = WithPos::new(
                Expression::Logical {
                    left: Box::new(expr),
                    operator,
                    right,
                },
                next.pos,
            )
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<WithPos<Expression>, ParserError> {
        let mut expr = self.comparison()?;

        while self.matched(vec![TokenType::BANGEQUAL, TokenType::EQUALEQUAL]) {
            let next = self.advance().unwrap();

            let operator = get_operator(next.token);

            let right_expr = Box::new(self.comparison()?);

            expr = WithPos::new(
                Expression::Binary {
                    left_expr: Box::new(expr),
                    operator,
                    right_expr,
                },
                next.pos,
            );

            println!("token {:#?}", self.tokens);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<WithPos<Expression>, ParserError> {
        let mut expr = self.addition()?;

        while self.matched(vec![
            TokenType::LESSTHAN,
            TokenType::LESSTHANEQUAL,
            TokenType::GREATERTHAN,
            TokenType::GREATERTHANEQUAL,
        ]) {
            let next = self.advance().unwrap();

            let operator = get_operator(next.token);

            let right_expr = Box::new(self.addition()?);

            expr = WithPos::new(
                Expression::Binary {
                    left_expr: Box::new(expr),
                    operator,
                    right_expr,
                },
                next.pos,
            )
        }

        Ok(expr)
    }

    fn addition(&mut self) -> Result<WithPos<Expression>, ParserError> {
        let mut expr = self.multiplication()?;

        while self.matched(vec![TokenType::MINUS, TokenType::PLUS]) {
            let next = self.advance().unwrap();

            let operator = get_operator(next.token);

            let right_expr = Box::new(self.multiplication()?);

            expr = WithPos::new(
                Expression::Binary {
                    left_expr: Box::new(expr),
                    operator,
                    right_expr,
                },
                next.pos,
            )
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<WithPos<Expression>, ParserError> {
        let mut expr = self.unary()?;

        while self.matched(vec![TokenType::SLASH, TokenType::STAR]) {
            let next = self.advance().unwrap();

            let operator = get_operator(next.token);

            let right_expr = Box::new(self.unary()?);

            expr = WithPos::new(
                Expression::Binary {
                    left_expr: Box::new(expr),
                    operator,
                    right_expr,
                },
                next.pos,
            )
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<WithPos<Expression>, ParserError> {
        if self.matched(vec![TokenType::BANG, TokenType::MINUS]) {
            let next = self.advance().unwrap();

            let operator = get_unary_operator(next.token);

            let right = Box::new(self.unary()?);

            return Ok(WithPos::new(
                Expression::Unary {
                    expr: right,
                    operator,
                },
                next.pos,
            ));
        }

        self.call()
    }

    fn call(&mut self) -> Result<WithPos<Expression>, ParserError> {
        let mut expr = self.primary()?;

        loop {
            if self.recognise(TokenType::LBRACKET) {
                self.advance();
                let index = Box::new(self.expression()?);
                let index_pos = self.consume_get_pos(
                    TokenType::RBRACKET,
                    "Expected ']' to close an index expression",
                )?;
                return Ok(WithPos::new(
                    Expression::IndexExpr {
                        target: Box::new(expr),
                        index,
                    },
                    index_pos,
                ));
            } else if self.recognise(TokenType::LPAREN) {
                let call_pos = self.advance().unwrap().pos;
                expr.pos = call_pos;
                expr = self.finish_call(expr)?;
            } else if self.recognise(TokenType::DOT) {
                let (name, pos) = self.consume_name_symbol("Expected a \'class\' name")?;
                expr = WithPos::new(
                    Expression::Get {
                        object: Box::new(expr),
                        name,
                        handle: self.variable_use_maker.next(),
                    },
                    pos,
                )
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<WithPos<Expression>, ParserError> {
        match self.advance() {
            Some(Token { ref token, ref pos }) => match *token {
                TokenType::FALSE(_) => Ok(WithPos::new(
                    Expression::Literal(Literal::False(false)),
                    *pos,
                )),
                TokenType::TRUE(_) => {
                    Ok(WithPos::new(Expression::Literal(Literal::True(true)), *pos))
                }
                TokenType::NIL => Ok(WithPos::new(Expression::Literal(Literal::Nil), *pos)),
                TokenType::INT(ref i) => {
                    Ok(WithPos::new(Expression::Literal(Literal::Int(*i)), *pos))
                }
                TokenType::FLOAT(ref f) => {
                    Ok(WithPos::new(Expression::Literal(Literal::Float(*f)), *pos))
                }
                TokenType::STRING(ref s) => Ok(WithPos::new(
                    Expression::Literal(Literal::Str(s.clone())),
                    *pos,
                )),
                TokenType::IDENTIFIER(ref ident) => Ok(WithPos::new(
                    Expression::Var(self.symbols.symbol(ident), self.variable_use_maker.next()),
                    *pos,
                )),
                TokenType::THIS => Ok(WithPos::new(
                    Expression::This(self.variable_use_maker.next()),
                    *pos,
                )),
                TokenType::FUNCTION => self.fun_body("function"),
                TokenType::LBRACKET => {
                    let mut items = vec![];

                    if self.recognise(TokenType::RBRACKET) {
                        self.advance();
                        return Ok(WithPos::new(Expression::Array { items }, *pos));
                    }

                    while {
                        items.push(self.expression()?);

                        self.recognise(TokenType::COMMA)
                            && self.advance().map(|t| t.token) == Some(TokenType::COMMA)
                    } {}

                    println!("{:#?}", self);

                    self.consume(
                        TokenType::RBRACKET,
                        "Expected a ']' to close the brackets .",
                    )?;

                    Ok(WithPos::new(Expression::Array { items }, *pos))
                }

                TokenType::LBRACE => {
                    let mut items: Vec<
                        (WithPos<Expression>, WithPos<Expression>),
                    > = vec![];

                    if self.recognise(TokenType::RBRACE) {
                        self.advance();
                        return Ok(WithPos::new(Expression::Dict { items }, *pos));
                    }

                    while {
                        let left = self.expression()?;
                        self.consume(TokenType::COLON, "Expected a ':' after dict key ")?;
                        let right = self.expression()?;

                        items.push((left, right));
                        self.recognise(TokenType::COMMA)
                            && self.advance().map(|t| t.token) == Some(TokenType::COMMA)
                    } {}

                    let pos = self.consume_get_pos(
                        TokenType::RBRACE,
                        "Expected a '}' to close a dictionary.",
                    )?;

                    Ok(WithPos::new(Expression::Dict { items }, pos))
                }

                TokenType::LPAREN => {
                    let expr = Box::new(self.expression()?);
                    let pos =
                        self.consume_get_pos(TokenType::RPAREN, "Expect \')\' after expression")?;

                    return Ok(WithPos::new(Expression::Grouping { expr }, pos));
                }

                _ => {
                    println!("{:#?}", token);
                    Err(ParserError::IllegalExpression(self.error(
                        "Cannot parse the expression",
                        *pos,
                    )))
                }
            },
            None => Err(ParserError::EOF),
        }
    }
}

// Helper parsing functions
impl<'a> Parser<'a> {
    fn fun_body(&mut self, kind: &str) -> Result<WithPos<Expression>, ParserError> {
        let func_pos = self.consume_get_pos(TokenType::LPAREN, "Expected '(' ")?;

        let mut parameters = vec![];
        let mut returns = None;

        if !self.recognise(TokenType::RPAREN) {
            while {
                if parameters.len() >= 32 {
                    println!("Cannot have more than 32 arguments")
                };

                let identifier = self.consume_name(&format!("Expected a {} name", kind))?;

                self.consume(TokenType::COLON, "Expected a colon")?;

                let id_type = self.advance().unwrap();

                let mut _ty = Symbol(0);

                match id_type.token {
                    TokenType::IDENTIFIER(ref ty) => _ty = self.symbols.symbol(ty),
                    TokenType::NIL => _ty = self.symbols.symbol("nil"),
                    _ => {
                        return Err(ParserError::Expected(self.error(
                            "Expected a proper type",
                            id_type.pos,
                        )))
                    }
                }

                parameters.push((identifier, _ty));

                self.recognise(TokenType::COMMA)
                    && self.advance().map(|t| t.token) == Some(TokenType::COMMA)
            } {}
        }

        self.consume(TokenType::RPAREN, "Expected ')' after parameters.")?;

        if self.recognise(TokenType::FRETURN) {
            self.advance();

            let possilbe_type = self.advance().unwrap();

            match possilbe_type.token {
                TokenType::IDENTIFIER(ref ty) => returns = Some(self.symbols.symbol(ty)),
                TokenType::NIL => returns = Some(self.symbols.symbol("nil")),
                _ => {
                    return Err(ParserError::Expected(self.error(
                        "Expected a proper type",
                        possilbe_type.pos,
                    )))
                }
            }
        }

        let body = Box::new(self.block()?);

        Ok(WithPos::new(
            Expression::Func {
                parameters,
                body,
                returns,
            },
            func_pos,
        ))
    }

    fn finish_call(
        &mut self,
        callee: WithPos<Expression>,
    ) -> Result<WithPos<Expression>, ParserError> {
        let mut arguments = vec![];

        if !self.recognise(TokenType::RPAREN) {
            while {
                if arguments.len() >= 32 {
                    println!("Cannot have more than 32 arguments.");
                }

                arguments.push(self.expression()?);
                self.recognise(TokenType::COMMA)
                    && self.advance().map(|t| t.token) == Some(TokenType::COMMA)
            } {}
        }

        let pos = self.consume_get_pos(TokenType::RPAREN, "Expected ')' after arguments.")?;

        Ok(WithPos::new(
            Expression::Call {
                callee: Box::new(callee),
                arguments,
            },
            pos,
        ))
    }
}
