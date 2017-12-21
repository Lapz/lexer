use ast::expr::Expression;
use pos::WithPos;
use symbol::Symbol;

#[derive(Debug, PartialOrd, Clone, PartialEq)]
pub enum Statement {
    ExpressionStmt(WithPos<Expression>),
    Var(Symbol, WithPos<Expression>, Option<Symbol>),
    Block(Vec<WithPos<Statement>>),
    Class {
        name: Symbol,
        methods: Vec<WithPos<Statement>>,
        properties: Vec<(Symbol, Symbol)>,
    },
    IfStmt {
        condition: WithPos<Expression>,
        then_branch: Box<WithPos<Statement>>,
        else_branch: Option<Box<WithPos<Statement>>>,
    },
    WhileStmt {
        condition: WithPos<Expression>,
        body: Box<WithPos<Statement>>,
    },

    Function {
        name: Symbol,
        body: WithPos<Expression>,
    },

    DoStmt {
        condition: WithPos<Expression>,
        body: Box<WithPos<Statement>>,
    },

    Break,
    Continue,
    TypeAlias {
        alias: Symbol,
        ty: Symbol,
    },
    Return(Option<WithPos<Expression>>),
}
