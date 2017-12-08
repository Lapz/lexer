use ast::expr::{Expression, Variable};
use pos::WithPos;
#[derive(Debug, PartialOrd, Clone, PartialEq)]
pub enum Statement<'a> {
    ExpressionStmt(Expression<'a>),
    PPrint(Expression<'a>),
    Var(Variable<'a>, Expression<'a>),
    Block(Vec<WithPos<Statement<'a>>>),
    Class {
        name: Variable<'a>,
        methods: Vec<Statement<'a>>,
    },
    IfStmt {
        condition: Expression<'a>,
        then_branch: Box<WithPos<Statement<'a>>>,
        else_branch: Option<Box<WithPos<Statement<'a>>>>,
    },
    WhileStmt {
        condition: Expression<'a>,
        body: Box<WithPos<Statement<'a>>>,
    },

    Function {
        name: Variable<'a>,
        body: Expression<'a>,
    },

    DoStmt {
        condition: Expression<'a>,
        body: Box<WithPos<Statement<'a>>>,
    },
    Break,
    Continue,
    Return(Option<Expression<'a>>),
}

#[derive(Debug, PartialOrd, Clone, PartialEq)]
pub struct Class<'a> {
    pub name: Variable<'a>,
    pub methods: Vec<Statement<'a>>,
}


#[derive(Debug, PartialOrd, Clone, PartialEq)]
pub struct For<'a> {
    pub initializer: Statement<'a>,
    pub condition: Expression<'a>,
    pub increment: Expression<'a>,
    pub body: Statement<'a>,
}
