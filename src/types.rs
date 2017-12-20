// use std::collections::{HashMap, HashSet};
use pos::Postition;
use symbol::Symbol;
#[derive(Debug, Clone)]
pub enum TypeError {
    Expected(Type, Postition),
    Undefinded,
    UndefindedVar,
    NotSame,
    Function,
    InvalidIndex,
    NotArray,
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum Type {
    Int,
    Str,
    Bool,
    Nil,
    Float,
    Dict(Box<Type>, Box<Type>), // Key, Value
    Array(Box<Type>),
    Name(Symbol,Box<Type>),
}
