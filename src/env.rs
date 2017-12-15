use types::Type;
use symbol::{Symbol, SymbolFactory, Symbols};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Entry {
    VarEntry(Type),
    FunEntry { parms: Vec<Type>, returns: Type },
}

#[derive(Debug, Clone)]
pub struct Env {
    pub types: Symbols<Type>,
    pub vars: Symbols<Entry>,
}

impl Env {
    pub fn new(strings: &Rc<SymbolFactory>) -> Self {

        let mut types = Symbols::new(Rc::clone(strings));
        let string_symbol =  types.symbol("str");
        let int_symbol = types.symbol("int");
        let float_symbol = types.symbol("float");
        let nil_symbol = types.symbol("nil");
        let bool_symbol  = types.symbol("bool");
        
        types.enter(int_symbol,Type::Int);
        types.enter(float_symbol,Type::Float);
        types.enter(bool_symbol,Type::Bool);
        types.enter(nil_symbol,Type::Nil);
        types.enter(string_symbol,Type::Str);

        Env {
            types,
            vars: Symbols::new(Rc::clone(strings)),
        }
    }
    pub fn look_type(&mut self, symbol: Symbol) -> Option<&Type> {
        self.types.look(symbol)
    }

    pub fn add_var(&mut self, symbol: Symbol, data: Entry) {
        self.vars.enter(symbol, data)
    }
}
