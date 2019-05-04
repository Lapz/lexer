use crate::ast as t;
use crate::infer::types;
use ir::instructions::*;
use std::collections::HashMap;
use syntax::ast::{Literal, Op,self};
use util::pos::Spanned;
use util::symbol::Symbol;
use util::symbol::Symbols;

#[derive(Debug)]
struct Builder<'a> {
    symbols: &'a Symbols<()>,
    locals: HashMap<Symbol, Register>,
    parameters: HashMap<Symbol, Register>,
    current_loop: Option<LoopDescription>,
    blocks: Vec<(BlockID, Block)>,
    current_block: Option<(BlockID, Vec<Instruction>)>,
}

impl<'a> Builder<'a> {
    pub fn new(symbols: &'a Symbols<()>) -> Self {
        Builder {
            symbols,
            locals: HashMap::new(),
            parameters: HashMap::new(),
            current_loop: None,
            current_block: None,
            blocks: Vec::new(),
        }
    }

    pub fn blocks(self) -> Vec<(BlockID, Block)> {
        self.blocks
    }

    pub fn new_block(&mut self) -> BlockID {
        BlockID::new()
    }

    pub fn start_block(&mut self, id: BlockID) {
        if self.current_block.is_some() {
            panic!("Block is unfinished");
        }

        self.current_block = Some((id, Vec::new()));
    }

    pub fn end_block(&mut self, end: BlockEnd) {
        let (id, inst) = self.current_block.take().unwrap();

        self.blocks.push(
            (id,
            Block {
                instructions: inst,
                end,
            }),
        );
    }

    pub fn parameters(&mut self) -> Vec<Register> {
        self.parameters
            .iter()
            .map(|(_, register)| *register)
            .collect()
    }

    pub fn locals(&mut self) -> Vec<Register> {
        self.locals.iter().map(|(_, register)| *register).collect()
    }

    pub fn emit_instruction(&mut self, inst: Inst) {
        self.current_block
            .as_mut()
            .expect("Basic Block should be started")
            .1
            .push(Instruction { instruction: inst });
    }

    pub fn emit_store(&mut self, dest: Register, source: Register) {
        self.current_block
            .as_mut()
            .expect("Basic block should be started")
            .1
            .push(Instruction {
                instruction: Inst::Store(dest, source),
            })
    }

    pub fn emit_store_immediate(&mut self, dest: Register, val: Value) {
        self.current_block
            .as_mut()
            .expect("Basic block should be started")
            .1
            .push(Instruction {
                instruction: Inst::StoreI(dest, val),
            })
    }

    pub fn add_param(&mut self, symbol: Symbol) {
        self.parameters.insert(symbol, Register::new());
    }

    pub fn add_local(&mut self, symbol: Symbol) -> Register {
        let reg = Register::new();

        self.locals.insert(symbol, reg);

        reg
    }

    pub fn build_statement(&mut self, s: Spanned<t::TypedStatement>) {
        use self::t::Statement;

        let s = *s.value.statement; // prevents colleteral moves error

        match s.value {
            Statement::Block(statements) => {
                for statement in statements {
                    self.emit_instruction(Inst::StatementStart);
                    self.build_statement(statement)
                }
            }
            Statement::Break => {
                let description = self.current_loop.expect("Cannot use break outside a loop");

                let new = self.new_block();

                self.end_block(BlockEnd::Jump(description.end()));

                self.start_block(new)
            }

            Statement::Continue => {
                let description = self
                    .current_loop
                    .expect("Cannot use continue outside a loop");

                let new = self.new_block();

                self.end_block(BlockEnd::Jump(description.start()));

                self.start_block(new)
            }

            Statement::Expr(expr) => {
                self.build_expr(expr);
            }

            Statement::If {
                cond,
                then,
                otherwise: Some(otherwise),
            } => {
                let c = self.build_expr(cond);

                let body = BlockID::new(); // then body
                let other = BlockID::new(); // else body
                let after = BlockID::new();

                self.end_block(BlockEnd::Branch(c, body, other));

                self.start_block(body);

                self.build_statement(then);

                self.end_block(BlockEnd::Jump(after));

                self.start_block(other);

                self.build_statement(otherwise);

                self.end_block(BlockEnd::Jump(after));

                self.start_block(after);
            }

            Statement::If {
                cond,
                then,
                otherwise: None,
            } => {
                let c = self.build_expr(cond);

                let body = BlockID::new();

                let after = BlockID::new();

                self.end_block(BlockEnd::Branch(c, body, after));

                self.start_block(body);

                self.build_statement(then);

                self.end_block(BlockEnd::Jump(after));

                self.start_block(after);
            }

            Statement::Let { ident, expr, .. } => {
                let reg = self.add_local(ident);

                if let Some(expr) = expr {
                    let expr = self.build_expr(expr);
                    self.emit_store(reg, expr);
                }
            }

            Statement::Print(expr) => {
                let expr = self.build_expr(expr);

                self.emit_instruction(Inst::Print(expr));
            }

            Statement::Return(expr) => {
                let result = self.build_expr(expr);
                let new = self.new_block();

                self.end_block(BlockEnd::Return(result));

                self.start_block(new);
            }

            Statement::While(cond, body) => {
                let cond_block = BlockID::new();
                let body_block = BlockID::new();
                let after = BlockID::new();

                self.current_loop = Some(LoopDescription {
                    start: cond_block,
                    end: after,
                });

                self.end_block(BlockEnd::Jump(cond_block));

                self.start_block(cond_block);

                let c = self.build_expr(cond);

                self.end_block(BlockEnd::Branch(c, body_block, after));

                self.start_block(body_block);

                self.build_statement(body);

                self.end_block(BlockEnd::Jump(cond_block));

                self.start_block(after);
            }
        }
    }

    fn build_expr(&mut self, expr: Spanned<t::TypedExpression>) -> Register {
        use self::t::Expression;

        match expr.value.expr.value {
            Expression::Array(items) => {
                let temp = Register::new();

                self.emit_instruction(Inst::Array(temp, items.len()));

                for (i, item) in items.into_iter().enumerate() {
                    let result = self.build_expr(item);
                    let offset = Register::new();
                    let temp = Register::new();

                    self.emit_store_immediate(temp, Value::Const(i as i64));

                    self.emit_instruction(Inst::Binary(offset, temp, BinaryOp::Plus, temp));

                    self.emit_store(offset, result);
                }

                temp
            }

            Expression::Assign(var, op, expr) => {
                let expr = self.build_expr(expr);
                let var = self.build_var(&var).expect("Undefined Variable");

                self.emit_store(var.clone(), expr);

                var
            }

            Expression::Binary(lhs, op, rhs) => match op {
                Op::And => self.build_and(lhs, rhs),
                Op::Or => self.build_or(lhs, rhs),
                _ => {
                    let lhs = self.build_expr(lhs);
                    let rhs = self.build_expr(rhs);

                    let op = gen_bin_op(op);
                    let result = Register::new();

                    self.emit_instruction(Inst::Binary(result, lhs, op, rhs));
                    result
                }
            },

            Expression::Call(callee, args) => {
                let result = Register::new();
                let mut reg_args = Vec::with_capacity(args.len()); // Temps where the expressions are stored

                for expr in args {
                    reg_args.push(self.build_expr(expr))
                }

                self.emit_instruction(Inst::Call(result, callee, reg_args));

                result
            }

            Expression::Cast(expr, _) => {
                let dest = Register::new();
                let from = expr.value.ty.clone();
                let result = self.build_expr(expr);

                self.emit_instruction(Inst::Cast(dest, result, get_size(from)));

                dest
            }

            Expression::ClassLiteral { .. } => unimplemented!(),

            Expression::Closure(_) => unimplemented!(),

            Expression::GetProperty {
                property_name,
                property,
            } => unimplemented!(),

            Expression::GetMethod {
                method_name,
                method,
            } => unimplemented!(),

            Expression::Index(target, index) => unimplemented!(),

            Expression::Literal(literal) => {
                let tmp = Register::new();

                match literal {
                    Literal::False(_) => {
                        self.emit_store_immediate(tmp, Value::Bool(false));
                    }
                    Literal::Nil => {
                        self.emit_store_immediate(tmp, Value::Nil);
                    }

                    Literal::Int(number) => self.emit_store_immediate(tmp, Value::Const(number)),

                    Literal::Float(number) => self.emit_store_immediate(tmp, Value::Float(number)),

                    Literal::Str(string) => {
                        let mut bytes = string.into_bytes();

                        self.emit_store_immediate(tmp, Value::Mem(bytes));
                    }

                    Literal::True(_) => {
                        self.emit_store_immediate(tmp, Value::Bool(true));
                    }
                };

                tmp
            }

            Expression::Match { cond, arms } => {
                unimplemented!()
                // let after_block = self.new_block();

                // let cond = self.build_expr(cond);

                // let result = Register::new();

                // let block_ids: Vec<BlockID> = (0..arms.value.len() + 1)
                //     .map(|_| self.new_block())
                //     .collect();

                // self.end_block(BlockEnd::Link(*block_ids.first().unwrap())); //fix empty match

                // for (i, arm) in arms.value.into_iter().enumerate() {
                //     let result = Register::new();
                //     self.start_block(block_ids[i]);
                //     let pattern = self.build_expr(arm.value.pattern);
                //     self.emit_instruction(Inst::Binary(
                //         result,
                //         pattern,
                //         BinaryOp::Equal,
                //         cond.clone(),
                //     ));
                //     self.build_statement(arm.value.body);
                //     self.end_block(BlockEnd::Branch(
                //         Value::Register(result),
                //         after_block,
                //         block_ids[i + 1],
                //     ));
                // }

                // if let Some(all) = all {
                //     self.start_block(*block_ids.last().unwrap());
                //     self.build_statement(all);
                //     self.end_block(BlockEnd::Link(after_block))
                // }

                // self.start_block(after_block);

                // result
            }

            Expression::Set(name, object, value) => {
                let result = Register::new();

                let class = self.build_var(&name).expect("Undefined Variable");

                let object = self.build_expr(object);

                let value = self.build_expr(value);

                // self.emit_instruction(ae);
                //
                unimplemented!();

                result
            }

            Expression::StaticMethodCall {
                class_name,
                method_name,
                params,
            } => unimplemented!(),

            Expression::Grouping(expr) => self.build_expr(expr),

            Expression::Unary(op,val) => {
                let result = Register::new();

                let val = self.build_expr(val);

                self.emit_instruction(Inst::Unary(result,val,gen_un_op(op)));

                result
            }

            Expression::Var(ref symbol, _) => self.build_var(symbol).unwrap(),

            ref e => unimplemented!("{:?}", e),
        }
    }

    fn build_and(
        &mut self,
        l: Spanned<t::TypedExpression>,
        r: Spanned<t::TypedExpression>,
    ) -> Register {
        let built_lhs = self.build_expr(l);
        let rhs_block = self.new_block();
        let reset_block = self.new_block();
        let after_block = self.new_block();
        let result = Register::new();

        self.end_block(BlockEnd::Branch(built_lhs.clone(), reset_block, rhs_block));

        self.start_block(rhs_block);

        let built_rhs = self.build_expr(r);

        self.emit_store(result, built_rhs.clone());

        self.end_block(BlockEnd::Jump(after_block));
        self.start_block(reset_block);

        self.emit_store_immediate(result, Value::Bool(true));

        self.end_block(BlockEnd::Jump(after_block));

        self.start_block(after_block);

        result
    }

    fn build_or(
        &mut self,
        l: Spanned<t::TypedExpression>,
        r: Spanned<t::TypedExpression>,
    ) -> Register {
        unimplemented!()
    }

    fn build_var(&self, var: &Symbol) -> Option<Register> {
        if let Some(register) = self.locals.get(var).or(self.parameters.get(var)) {
            Some(*register)
        } else {
            None
        }
    }
}

fn build_function(function: t::Function, symbols: &Symbols<()>) -> Function {
    let mut builder = Builder::new(symbols);

    for param in function.params {
        builder.add_param(param.name);
    }

    let start = builder.new_block();

    builder.start_block(start);
    builder.build_statement(*function.body);

    if builder.current_block.is_some() {
        builder.end_block(BlockEnd::End);
    }

    Function {
        name: function.name,
        params: builder.parameters(),
        locals: builder.locals(),
        start_block: start,
        blocks: builder.blocks(),
    }
}

pub fn build_program(symbols: &Symbols<()>, old_program: t::Program) -> Program {
    let mut new_program = Program {
        functions: vec![],
        classes: vec![],
    };

    for function in old_program.functions {
        new_program
            .functions
            .push(build_function(function, symbols));
    }

    new_program
}

fn gen_un_op(op: ast::UnaryOp) -> UnaryOp {
    match op {
        ast::UnaryOp::Minus => UnaryOp::Minus,
        ast::UnaryOp::Bang => UnaryOp::Bang,
    }
}

fn get_size(ty: types::Type) -> Size {
    match ty {
        types::Type::App(types::TypeCon::Float, _) => Size::Bit64,
        types::Type::App(types::TypeCon::Str, _) => Size::Bit64,
        types::Type::App(types::TypeCon::Int, _) => Size::Bit64,
        _ => unreachable!(),
    }
}

fn gen_bin_op(op: Op) -> BinaryOp {
    match op {
        Op::Plus => BinaryOp::Plus,
        Op::Minus => BinaryOp::Minus,
        Op::Star => BinaryOp::Mul,
        Op::Slash => BinaryOp::Div,
        Op::LessThan => BinaryOp::Lt,
        Op::GreaterThan => BinaryOp::Gt,
        Op::LessThanEqual => BinaryOp::Lte,
        Op::GreaterThanEqual => BinaryOp::Gte,
        Op::EqualEqual => BinaryOp::Equal,
        Op::BangEqual => BinaryOp::NotEqual,
        // Op::And => BinaryOp::And,
        // Op::Or => BinaryOp::Or,
        _ => unreachable!(),
    }
}