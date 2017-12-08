use token::{Token, TokenType};
use std::iter::Peekable;
use std::vec::IntoIter;
use pos::Postition;
use ast::expr::*;
use ast::statement::*;
use pos::WithPos;
// use pprint::PrettyPrint;
#[derive(Debug)]
pub struct Parser<'a> {
    tokens: Peekable<IntoIter<Token<'a>>>,
    loop_depth: i32,
    variable_use_maker: VariableUseMaker,
}

use std::fmt::{Display, Formatter};
use std::fmt;


#[derive(Clone, Debug)]
pub enum ParserError<'a> {
    NotExpected(String),
    IllegalToken(String),
    IllegalExpression(String),
    EOF,
    EOFExpected(TokenType<'a>),
    EOFMany(Vec<TokenType<'a>>),
    MissingCloseBracket,
    Expected(String),
    Break(String),
}

impl<'a> Display for ParserError<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ParserError::NotExpected(ref e) |
            ParserError::IllegalToken(ref e) |
            ParserError::Expected(ref e) |
            ParserError::IllegalExpression(ref e) |
            ParserError::Break(ref e) => write!(f, "{}", e),
            ParserError::MissingCloseBracket => write!(f, " ')' was expected but not found "),
            ParserError::EOF => write!(f, "Unexpected end of file"),
            ParserError::EOFExpected(ref e) => {
                write!(f, "Expected {} but instead found the EOF", e)
            }
            ParserError::EOFMany(ref many) => {
                write!(f, "Expected {:?} but instead found EOF", many)
            }

        }
    }
}


impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),

            loop_depth: 0,
            variable_use_maker: VariableUseMaker::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<WithPos<Statement<'a>>>, Vec<ParserError<'a>>> {
        let mut statements = vec![];

        let mut errors = vec![];

        while self.tokens.peek() != None {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(e) => {
                    match self.synchronize() {
                        Ok(()) => (),
                        Err(e) => errors.push(e),
                    }
                    errors.push(e)
                }
            }
        }

        if errors.is_empty() {
            Ok(statements)
        } else {
            Err(errors)
        }


    }

    pub fn parse_single(&mut self) -> Result<WithPos<Expression<'a>>, ParserError<'a>> {
        let pos = self.tokens.peek().map(|t| t.pos).unwrap();

        Ok(WithPos::new(self.expression()?, pos))
    }

    pub fn synchronize(&mut self) -> Result<(), ParserError<'a>> {
        self.advance();

        while self.peek(|token| token == &TokenType::EOF) {
            match self.advance().map(|t| t.token) {
                Some(TokenType::CLASS) |
                Some(TokenType::FUNCTION) |
                Some(TokenType::IDENTIFIER(_)) |
                Some(TokenType::FOR) |
                Some(TokenType::IF) |
                Some(TokenType::WHILE) |
                Some(TokenType::RETURN) => break,
                None => unreachable!(),
                _ => self.tokens.next(),
            };
        }

        Ok(())

    }


    fn error(&self, message: &str, pos: Postition) -> String {
        format!("{} on {}", message, pos)
    }


    fn peek<F>(&mut self, mut check: F) -> bool
    where
        F: FnMut(&TokenType<'a>) -> bool,
    {
        self.tokens.peek().map_or(
            false,
            |token| check(&token.token),
        )
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

        if found { true } else { false }

    }

    fn advance(&mut self) -> Option<Token<'a>> {
        self.tokens.next()
    }


    fn token_type(&mut self) -> TokenType<'a> {
        self.advance().map(|t| t.token).unwrap()
    }

    fn consume(&mut self, token_to_check: TokenType<'a>, msg: &str) -> Result<(), ParserError<'a>> {
        match self.tokens.next() {
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
    ) -> Result<Postition, ParserError<'a>> {
        match self.tokens.next() {
            Some(Token { ref token, ref pos }) => {
                if token == &token_to_check {
                    return Ok(*pos);
                }

                Err(ParserError::Expected(self.error(msg, *pos)))
            }
            None => Err(ParserError::EOF),
        }
    }

    fn consume_name_pos(
        &mut self,
        msg: &str,
    ) -> Result<(Variable<'a>, Postition), ParserError<'a>> {
        match self.tokens.next() {
            Some(Token {
                     token: TokenType::IDENTIFIER(ref ident),
                     pos,
                 }) => Ok((Variable(ident), pos)),
            Some(Token { ref pos, .. }) => Err(ParserError::Expected(self.error(msg, *pos))),
            None => Err(ParserError::EOF),
        }
    }

    fn get_pos(&mut self) -> Postition {
        match self.tokens.next() {
            Some(Token { ref pos, .. }) => return *pos,
            _ => unreachable!(),
        }
    }
}

// Statements
impl<'a> Parser<'a> {
    fn statement(&mut self) -> Result<WithPos<Statement<'a>>, ParserError<'a>> {

        if self.recognise(TokenType::LBRACE) {
            self.block()
        } else {
            self.expression_statement()
        }
    }




    fn expression_statement(&mut self) -> Result<WithPos<Statement<'a>>, ParserError<'a>> {
        let expr = self.expression()?;

        let pos = self.consume_get_pos(
            TokenType::SEMICOLON,
            "Expected \';\' after value.",
        )?;

        Ok(WithPos::new(Statement::ExpressionStmt(expr), pos))
    }

    fn break_statement(&mut self) -> Result<WithPos<Statement<'a>>, ParserError<'a>> {
        let break_pos = self.get_pos();

        if !(self.loop_depth >= 0) {
            let error = "Must be inside a loop to use break".to_owned();
            return Err(ParserError::Break(error));
        }

        self.consume(
            TokenType::SEMICOLON,
            "Expected ';' after 'break'",
        )?;

        Ok(WithPos::new(Statement::Break, break_pos))
    }

    fn continue_statement(&mut self) -> Result<WithPos<Statement<'a>>, ParserError<'a>> {
        let cont_pos = self.get_pos();

        if !(self.loop_depth >= 0) {
            let error = "Must be inside a loop to use 'continue'".to_owned();

            return Err(ParserError::Break(error));
        }

        self.consume(
            TokenType::SEMICOLON,
            "Expected ';' after 'continue'",
        )?;

        Ok(WithPos::new(Statement::Continue, cont_pos))
    }

    // Control Flow Statements

    fn do_statement(&mut self) -> Result<WithPos<Statement<'a>>, ParserError<'a>> {
        let do_pos = self.get_pos();

        let body = self.statement()?;

        self.consume(
            TokenType::WHILE,
            "Expected while after 'do' condition",
        )?;

        self.consume(TokenType::LPAREN, "Expected '(' after while'")?;

        let condition = self.expression()?;

        self.consume(
            TokenType::RPAREN,
            "Expected ')' after 'while'",
        )?;


        let do_statement = Statement::DoStmt {
            body: Box::new(body),
            condition,
        };

        Ok(WithPos::new(do_statement, do_pos))
    }

    fn while_statement(&mut self) -> Result<WithPos<Statement<'a>>, ParserError<'a>> {
        let pos = self.get_pos(); // Eats the while;

        self.consume(TokenType::LPAREN, "Expected '(' after while'")?;

        let condition = self.expression()?;

        self.consume(
            TokenType::RPAREN,
            "Expected ')' after 'while'",
        )?;

        self.loop_depth += 1;

        let body = self.statement()?;

        let while_st = Statement::WhileStmt {
            condition,
            body: Box::new(body),
        };

        self.loop_depth -= 1;

        Ok(WithPos::new(while_st, pos))
    }

    fn if_statement(&mut self) -> Result<WithPos<Statement<'a>>, ParserError<'a>> {
        let pos = self.get_pos(); // Eats the if ;

        self.consume(
            TokenType::LPAREN,
            "Expected a \'(\' after \'if\'",
        )?;

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
                pos,
            ));
        }

        Ok(WithPos::new(
            Statement::IfStmt {
                condition,
                then_branch,
                else_branch,
            },
            pos,
        ))

    }


    fn return_statement(&mut self) -> Result<WithPos<Statement<'a>>, ParserError<'a>> {
        let pos = self.get_pos();

        let mut value = None;

        if self.peek(|token| token != &TokenType::COLON) {
            value = Some(self.expression()?);
        }

        self.consume(TokenType::SEMICOLON, "Expected a ")?;

        Ok(WithPos::new(Statement::Return(value), pos))

    }

    fn block(&mut self) -> Result<WithPos<Statement<'a>>, ParserError<'a>> {
        let pos = self.get_pos();

        let mut statement = vec![];

        while self.peek(|token| token != &TokenType::RBRACE) {
            statement.push(self.declaration()?);
        }

        self.consume(
            TokenType::RBRACE,
            "Expected a \'}\' after block.",
        )?;

        Ok(WithPos::new(Statement::Block(statement), pos))
    }

    fn declaration(&mut self) -> Result<WithPos<Statement<'a>>, ParserError<'a>> {
        if self.recognise(TokenType::VAR) {
            return self.var_declaration();
        }

        unimplemented!()
    }

    fn var_declaration(&mut self) -> Result<WithPos<Statement<'a>>, ParserError<'a>> {
        let (name, pos) = self.consume_name_pos(
            "Expected an IDENTIFIER after a \'var\' ",
        )?;

        if self.recognise(TokenType::SEMICOLON) {
            self.advance();

            let value = Expression::Literal(Literal::Nil);

            return Ok(WithPos::new(Statement::Var(name, value), pos));
        }

        if self.matched(vec![
            TokenType::ASSIGN,
            TokenType::MINUSASSIGN,
            TokenType::PLUSASSIGN,
            TokenType::SLASHASSIGN,
            TokenType::STARASSIGN,
        ])
        {
            self.advance();
            let expr = self.expression()?;
            self.consume(
                TokenType::SEMICOLON,
                "Expect \';\' after variable decleration.",
            )?;
            return Ok(WithPos::new(Statement::Var(name, expr), pos));
        }

        Err(ParserError::Expected(
            self.error("Expected an assignment", pos),
        ))
    }
}


// Expression Parsing
impl<'a> Parser<'a> {
    fn expression(&mut self) -> Result<Expression<'a>, ParserError<'a>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expression<'a>, ParserError<'a>> {
        let expr = self.ternary()?;

        if self.matched(vec![
            TokenType::ASSIGN,
            TokenType::PLUSASSIGN,
            TokenType::MINUSASSIGN,
            TokenType::SLASHASSIGN,
            TokenType::STARASSIGN,
        ])
        {
            let kind = get_assign_operator(self.token_type());

            let value = self.assignment()?;

            match expr {
                Expression::Var(name, _) => {
                    return Ok(Expression::Assign {
                        handle: self.variable_use_maker.next(),
                        name,
                        value: Box::new(value),
                        kind,
                    })
                }
                _ => {
                    return Err(ParserError::IllegalExpression(
                        "Error at '=': Invalid assignment target.".to_owned(),
                    ))
                }
            }
        }

        Ok(expr)


    }


    fn ternary(&mut self) -> Result<Expression<'a>, ParserError<'a>> {
        let mut condition = self.or()?;

        while self.matched(vec![TokenType::QUESTION]) {
            self.consume(TokenType::QUESTION, "Expected a '?'")?;

            let then_branch = Box::new(self.expression()?);

            self.consume(
                TokenType::COLON,
                "Expected ':' after lhs ternary condition.",
            )?;

            let else_branch = Box::new(self.ternary()?);

            condition = Expression::Ternary {
                condition: Box::new(condition),
                then_branch,
                else_branch,
            }
        }

        Ok(condition)
    }

    fn or(&mut self) -> Result<Expression<'a>, ParserError<'a>> {
        let mut expr = self.and()?;

        if self.recognise(TokenType::OR) {
            let operator = get_logic_operator(self.token_type());

            let right = Box::new(self.and()?);

            expr = Expression::Logical {
                left: Box::new(expr),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expression<'a>, ParserError<'a>> {
        let mut expr = self.comma()?;

        if self.recognise(TokenType::AND) {
            let operator = get_logic_operator(self.token_type());

            let right = Box::new(self.comma()?);

            expr = Expression::Logical {
                left: Box::new(expr),
                operator,
                right,
            }
        }

        Ok(expr)
    }

    fn comma(&mut self) -> Result<Expression<'a>, ParserError<'a>> {

        let mut expr = self.equality()?;

        while self.matched(vec![TokenType::COMMA]) {
            let operator = get_operator(self.token_type());

            let right_expr = Box::new(self.equality()?);

            expr = Expression::Binary {
                left_expr: Box::new(expr),
                operator,
                right_expr,
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expression<'a>, ParserError<'a>> {
        let mut expr = self.comparison()?;

        while self.matched(vec![TokenType::BANGEQUAL, TokenType::EQUALEQUAL]) {

            let operator = get_operator(self.token_type());

            let right_expr = Box::new(self.comparison()?);

            expr = Expression::Binary {
                left_expr: Box::new(expr),
                operator,
                right_expr,
            };

            println!("token {:#?}", self.tokens);

        }

        Ok(expr)

    }

    fn comparison(&mut self) -> Result<Expression<'a>, ParserError<'a>> {

        let mut expr = self.addition()?;

        while self.matched(vec![
            TokenType::LESSTHAN,
            TokenType::LESSTHANEQUAL,
            TokenType::GREATERTHAN,
            TokenType::GREATERTHANEQUAL,
        ])
        {
            let operator = get_operator(self.token_type());

            let right_expr = Box::new(self.addition()?);

            expr = Expression::Binary {
                left_expr: Box::new(expr),
                operator,
                right_expr,
            }
        }

        Ok(expr)
    }

    fn addition(&mut self) -> Result<Expression<'a>, ParserError<'a>> {

        let mut expr = self.multiplication()?;

        while self.matched(vec![TokenType::MINUS, TokenType::PLUS]) {
            let operator = get_operator(self.token_type());

            let right_expr = Box::new(self.multiplication()?);

            expr = Expression::Binary {
                left_expr: Box::new(expr),
                operator,
                right_expr,
            }

        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<Expression<'a>, ParserError<'a>> {
        let mut expr = self.unary()?;

        while self.matched(vec![TokenType::SLASH, TokenType::STAR]) {
            let operator = get_operator(self.token_type());

            let right_expr = Box::new(self.unary()?);

            expr = Expression::Binary {
                left_expr: Box::new(expr),
                operator,
                right_expr,
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression<'a>, ParserError<'a>> {
        if self.matched(vec![TokenType::BANG, TokenType::MINUS, TokenType::PLUS]) {
            let operator = get_unary_operator(self.token_type());

            let right = Box::new(self.unary()?);

            return Ok(Expression::Unary {
                expr: right,
                operator,
            });
        }

        self.primary()

    }


    fn primary(&mut self) -> Result<Expression<'a>, ParserError<'a>> {
        match self.advance() {
            Some(Token { ref token, ref pos }) => {
                match *token {
                    TokenType::FALSE(_) => Ok(Expression::Literal(Literal::False(false))),
                    TokenType::TRUE(_) => Ok(Expression::Literal(Literal::True(true))),
                    TokenType::NIL => Ok(Expression::Literal(Literal::Nil)),
                    TokenType::INT(ref i) => Ok(Expression::Literal(Literal::Int(*i))),
                    TokenType::FLOAT(ref f) => Ok(Expression::Literal(Literal::Float(*f))),
                    TokenType::STRING(ref s) => Ok(Expression::Literal(Literal::Str(s.clone()))),
                    TokenType::IDENTIFIER(ref i) => Ok(Expression::Var(
                        Variable(i),
                        self.variable_use_maker.next(),
                    )),
                    TokenType::LPAREN => {
                        let expr = Box::new(self.expression()?);
                        self.consume(
                            TokenType::RPAREN,
                            "Expect \')\' after expression",
                        )?;

                        return Ok(Expression::Grouping { expr });

                    }

                    _ => unimplemented!(),
                }
            }
            None => Err(ParserError::EOF),
            Some(other) => unimplemented!(),

        }
    }
}
