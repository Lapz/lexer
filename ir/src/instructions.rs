use std::collections::HashMap;
use std::fmt::{self, Debug, Display};
use util::symbol::Symbol;

static mut LABEL_COUNT: u32 = 0;

static mut REGISTER_COUNT: u32 = 0;

static mut BLOCK_COUNT: u32 = 0;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy)]
pub struct BlockID(pub u32);

/// A label in the code.
pub struct Label(u32);

/// A register
#[derive(Clone, Copy, Hash, PartialEq, Default, Eq)]
pub struct Register(pub u32);

pub struct Program {
    pub functions: Vec<Function>,
    pub classes: Vec<Class>,
}

pub struct Function {
    pub name: Symbol,
    pub params: Vec<Register>,
    pub locals: Vec<Register>,
    pub blocks: HashMap<BlockID, Block>,
    pub start_block: BlockID,
}
#[derive(Debug, Clone)]
pub struct StructLayout {
    fields: HashMap<Symbol, usize>,
}

#[derive(Debug, Clone)]
pub struct Layout {
    /// the size of the field
    size_of: usize,
    /// the alignment
    align_of: usize,
}
#[derive(Debug, Clone)]
pub struct Class {
    ident: Symbol,
}

#[derive(Debug, Clone, Copy)]
pub struct LoopDescription {
    pub start: BlockID,
    pub end: BlockID,
}

#[derive(Clone)]
pub struct Block {
    pub instructions: Vec<Instruction>,
    pub end: BlockEnd,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockEnd {
    Jump(BlockID),
    Return(Register),
    Branch(Register, BlockID, BlockID),
    Link(BlockID),
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub instruction: Inst,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Size {
    Bit8,
    Bit32,
    Bit64,
}
/// Instruction used in the IR
/// Instructions are of the form i <- a op b
#[derive(Debug, Clone, PartialEq)]
pub enum Inst {
    /// A stack allocated array of size whatever
    /// Stored at a location
    Array(Register, usize),

    Drop(Register),
    /// $dest = $lhs $op $rhs
    Binary(Register, Register, BinaryOp, Register),
    ///Cast the value from one type to another
    /// $dest = $val as type
    Cast(Register, Register, Size),
    /// Call the function with the provided args in registers and store in in the dest
    /// $dest = call $func
    Call(Register, Symbol, Vec<Register>),
    /// Print the value stored in the register
    Print(Register),

    StatementStart,

    /// $dest = val
    StoreI(Register, Value),
    /// $dest = val
    Store(Register, Register),

    /// $dest = op $val
    Unary(Register, Register, UnaryOp),

    Return(Register),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    /// Integer Constant
    Const(i64),
    /// Float Constant
    Float(f64),
    /// One of many registers
    Register(Register),
    ///  Contents of a word of memory at address
    Mem(Vec<u8>),

    Bool(bool),

    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Plus,
    Minus,
    Mul,
    Div,
    Gt,
    Gte,
    Lt,
    Lte,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Bang,
    Minus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CmpOp {
    LT,
    GT,
    LTE,
    GTE,
    EQ,
    NE,
}

impl BlockID {
    pub fn new() -> BlockID {
        let count = unsafe { BLOCK_COUNT };

        let id = BlockID(count);

        unsafe {
            BLOCK_COUNT += 1;
        }

        id
    }
}

impl LoopDescription {
    pub fn start(&self) -> BlockID {
        self.start
    }

    pub fn end(&self) -> BlockID {
        self.end
    }
}
impl Label {
    pub fn new() -> Label {
        let count = unsafe { LABEL_COUNT };

        let label = Label(count);

        unsafe {
            LABEL_COUNT += 1;
        }

        label
    }
}

impl Register {
    pub fn new() -> Register {
        let count = unsafe { REGISTER_COUNT };

        let temp = Register(count);

        unsafe {
            REGISTER_COUNT += 1;
        }

        temp
    }
}

impl Layout {
    pub fn new(size_of: usize, align_of: usize) -> Layout {
        Layout { align_of, size_of }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Value::Const(ref v) => write!(f, "{}", v),
            Value::Float(ref v) => write!(f, "{}", v),
            Value::Register(ref name) => write!(f, "{}", name),
            Value::Bool(ref b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Mem(ref bytes) => {
                write!(f, "[")?;

                for (i, byte) in bytes.iter().enumerate() {
                    if i + 1 == bytes.len() {
                        write!(f, "{}", byte)?;
                    } else {
                        write!(f, "{},", byte)?;
                    }
                }

                write!(f, "]")
            }
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BinaryOp::Plus => write!(f, "+"),
            BinaryOp::Minus => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Lte => write!(f, "<="),
            BinaryOp::Gte => write!(f, ">="),
            BinaryOp::Equal => write!(f, "=="),
            BinaryOp::NotEqual => write!(f, "!="),
        }
    }
}

impl Display for CmpOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CmpOp::LT => write!(f, "<"),
            CmpOp::LTE => write!(f, "<="),
            CmpOp::GT => write!(f, ">"),
            CmpOp::GTE => write!(f, ">="),
            CmpOp::NE => write!(f, "!="),
            CmpOp::EQ => write!(f, "=="),
        }
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UnaryOp::Bang => write!(f, "!"),
            UnaryOp::Minus => write!(f, "-"),
        }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

impl Debug for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "l{}", self.0)
    }
}
impl Display for BlockID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "label_{}", self.0)
    }
}

impl Display for BlockEnd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BlockEnd::End => write!(f, "end"),
            BlockEnd::Jump(ref id) => write!(f, "goto {}", id),
            BlockEnd::Return(ref id) => write!(f, "return {}", id),
            BlockEnd::Branch(ref v, ref t_branch, ref f_branch) => {
                write!(f, "branch {} {} {}", v, t_branch, f_branch)
            }

            BlockEnd::Link(ref link) => write!(f, "next {}", link),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.instruction == Inst::StatementStart {
            write!(f, "")
        } else {
            write!(f, "{}", self.instruction)
        }
    }
}

impl Display for Inst {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Inst::Array(ref l, ref s) => write!(out, "{} <- [{}]", l, s),
            Inst::StatementStart => write!(out, ""),
            Inst::Binary(ref res, ref lhs, ref op, ref rhs) => {
                write!(out, "{} <- {} {} {}", res, lhs, op, rhs)
            }
            Inst::Print(ref v) => write!(out, "print {}", v),
            Inst::Drop(ref reg) => write!(out, "drop {}", reg),
            Inst::Store(ref dest, ref source) => write!(out, "{} <- {}", dest, source),
            Inst::StoreI(ref dest, ref val) => write!(out, "{} <- {}", dest, val),
            Inst::Cast(ref dest, _, ref ty) => write!(out, "{} as {}", dest, ty),
            Inst::Unary(ref dest, ref source, ref op) => {
                write!(out, "{} <- {}{}", dest, op, source)
            }
            Inst::Return(ref label) => write!(out, "return @{}", label),
            Inst::Call(ref dest, ref callee, ref args) => {
                for arg in args {
                    write!(out, "arg: {}\n", arg)?;
                }

                write!(out, "{} <- call {}", dest, callee)?;

                Ok(())
            }
        }
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for inst in self.instructions.iter() {
            writeln!(f, "{}", inst)?;
        }

        writeln!(f, "{}", self.end)?;

        Ok(())
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Size::Bit8 => write!(f, "i8"),
            Size::Bit32 => write!(f, "i32"),
            Size::Bit64 => write!(f, "i64"),
        }
    }
}
