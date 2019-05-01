use util::pos::Spanned;
use util::symbol::Symbol;

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<Spanned<Function>>,
    pub classes: Vec<Spanned<Class>>,
    pub aliases: Vec<Spanned<TypeAlias>>,
    pub enums: Vec<Spanned<Enum>>,
}

#[derive(Debug, Clone)]
pub struct ItemName {
    pub name: Spanned<Symbol>,
    pub type_params: Vec<Spanned<Symbol>>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Spanned<ItemName>,
    pub params: Spanned<Vec<Spanned<FunctionParam>>>,
    pub body: Spanned<Statement>,
    pub returns: Option<Spanned<Type>>,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub name: Spanned<ItemName>,
    pub superclass: Option<Spanned<Symbol>>,
    pub methods: Vec<Spanned<Function>>,
    pub fields: Vec<Spanned<Field>>,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: Spanned<ItemName>,
    pub constructors: Vec<Spanned<EnumConstructor>>,
}
#[derive(Debug, Clone)]
pub struct EnumConstructor {
    pub con: Spanned<Symbol>,
    pub args: Spanned<Vec<Spanned<Type>>>,
}

#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub alias: Spanned<ItemName>,
    pub ty: Spanned<Type>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Block(Vec<Spanned<Statement>>),
    Break,
    Continue,
    Expr(Spanned<Expression>),
    For {
        init: Option<Box<Spanned<Statement>>>,
        cond: Option<Spanned<Expression>>,
        incr: Option<Spanned<Expression>>,
        body: Box<Spanned<Statement>>,
    },

    If {
        cond: Spanned<Expression>,
        then: Box<Spanned<Statement>>,
        otherwise: Option<Box<Spanned<Statement>>>,
    },

    Print(Spanned<Expression>),

    While {
        cond: Spanned<Expression>,
        body: Box<Spanned<Statement>>,
    },

    VarDeclaration {
        ident: Spanned<Symbol>,
        ty: Option<Spanned<Type>>,
        expr: Option<Spanned<Expression>>,
    },

    Return(Spanned<Expression>),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: Spanned<Symbol>,
    pub ty: Spanned<Type>,
}

#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: Spanned<Symbol>,
    pub ty: Spanned<Type>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Var(Spanned<Symbol>),
    Or(Box<Spanned<Pattern>>, Box<Spanned<Pattern>>),
    /// A construtor like Red(125,255,255)
    /// A pattern var is a constructor with no args
    /// A litteral is 
    Con(Spanned<Symbol>, Spanned<Vec<Spanned<Pattern>>>),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Array {
        items: Vec<Spanned<Expression>>,
    },
    Assign {
        name: Spanned<Symbol>,
        kind: Spanned<AssignOperator>,
        value: Box<Spanned<Expression>>,
    },
    Binary {
        lhs: Box<Spanned<Expression>>,
        op: Spanned<Op>,
        rhs: Box<Spanned<Expression>>,
    },

    Call(Spanned<Call>),

    Cast {
        from: Box<Spanned<Expression>>,
        to: Spanned<Type>,
    },

    ClassLiteral(Spanned<ClassLiteral>),

    Closure(Box<Spanned<Function>>),

    Get {
        object: Box<Spanned<Expression>>,
        property: Spanned<Symbol>,
    },

    Grouping {
        expr: Box<Spanned<Expression>>,
    },

    Match {
        cond: Box<Spanned<Expression>>,
        arms: Spanned<Vec<Spanned<MatchArm>>>,
    },

    SubScript {
        target: Box<Spanned<Expression>>,
        index: Box<Spanned<Expression>>,
    },

    Literal(Literal),

    Set {
        object: Box<Spanned<Expression>>,
        name: Spanned<Symbol>,
        value: Box<Spanned<Expression>>,
    },

    Ternary {
        condition: Box<Spanned<Expression>>,
        then_branch: Box<Spanned<Expression>>,
        else_branch: Box<Spanned<Expression>>,
    },
    Unary {
        op: Spanned<UnaryOp>,
        expr: Box<Spanned<Expression>>,
    },

    Var(Spanned<Symbol>),

    Variant {
        enum_name: Spanned<Symbol>,
        variant: Spanned<Symbol>,
        inner: Option<Box<Spanned<Expression>>>,
    },
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Option<Spanned<Pattern>>,
    pub body: Spanned<Statement>,
    pub is_all: bool,
}

#[derive(Debug, Clone)]
pub struct ClassLiteralField {
    pub symbol: Spanned<Symbol>,
    pub expr: Spanned<Expression>,
}

/// A class literal with  provided types or with no provided types
/// i.e Foo::<i32> {
///     int:0
/// }
///  /// i.e Foo {
///     int:0
/// }
/// if a classliteral is not instantiated the span for types is the [`EMPTYSPAN`]
#[derive(Debug, Clone)]
pub struct ClassLiteral {
    pub symbol: Spanned<Symbol>,
    pub types: Spanned<Vec<Spanned<Type>>>,
    pub props: Vec<Spanned<ClassLiteralField>>,
}

/// A function/method literal with optional provided types
/// i.e id::<i32>(foo); foo.add::<i32,i32>(10,2); foo(1,2);
/// if a function call is not instantiated the span for types is the [`EMPTYSPAN`]
#[derive(Debug, Clone)]
pub struct Call {
    pub args: Vec<Spanned<Expression>>,
    pub callee: Box<Spanned<Expression>>,
    pub types: Spanned<Vec<Spanned<Type>>>,
}
#[derive(Debug, Clone)]
pub enum Type {
    /// Type that is an identifier i.e bool,int,float
    Simple(Spanned<Symbol>),
    /// Type that is an array i.e [int]
    Arr(Box<Spanned<Type>>),
    /// Type that is a function i.e fn(int,int) -> int;
    Func(Vec<Spanned<Type>>, Option<Box<Spanned<Type>>>),
    /// Type that is nill
    /// TODO remove and rely on Simple
    Nil,
    /// Type of a generic type i.e List<i32> Foo<List<List<i32>>>
    Generic(Spanned<Symbol>, Vec<Spanned<Type>>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    // The raw values available
    Float(f64),
    Int(i64),
    Str(String),
    True(bool),
    False(bool),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    // The possible operators for the binary and unary expression
    BangEqual,
    EqualEqual,
    LessThan,
    LessThanEqual,
    GreaterThanEqual,
    GreaterThan,
    Plus,
    Minus,
    Star,
    Slash,
    Modulo,
    Exponential,
    And,
    Or,
}

#[derive(Debug, Clone, Hash)]
pub enum AssignOperator {
    // The possible operators for the binary and unary expression
    Equal,
    MinusEqual,
    PlusEqual,
    StarEqual,
    SlashEqual,
}

#[derive(Debug, Clone, Hash)]
pub enum UnaryOp {
    Bang,
    Minus,
}
