//! Instruction in the VM;
//! Each Instruction in the VM is 32 bits and the first 8 bits will
//! contain our opcode
//! The reaming 24 bits will be used for the Operands

pub type OpCode = u8;
/// HLT
/// Stops the running of the vm
pub const HLT: u8 = 0x1;
/// JMP Dest
/// Changes the ip to the value in the register
/// Allows for jumping forward or backwards
pub const JMP: u8 = 0x2;
/// Returns the value stored the in RET register
pub const RETURN: u8 = 0x3;
/// NUMBER
pub const CONSTANT: u8 = 0x4;
/// NEGATE $DEST
/// Unary negate of the dest
pub const NEGATE: u8 = 0x5;
/// ADD SRC SRC DEST
pub const ADD: u8 = 0x6;
/// SUB SRC SRC DEST
pub const SUB: u8 = 0x7;
/// MUL SRC SRC DEST
pub const MUL: u8 = 0x8;
/// DIV SRC SRC DEST
pub const DIV: u8 = 0x9;
/// DIV SRC SRC DEST
pub const NIL: u8 = 0x10;
pub const TRUE: u8 = 0x11;
pub const FALSE: u8 = 0x12;
/// NOT
/// Set the equal_flag to !equal_flag
pub const NOT: u8 = 0x13;
/// EQUAL SRC SRC
/// Sets the equal_flag to true
pub const EQUAL: u8 = 0x14;
/// GREATER SRC SRC
/// Sets the equal_flag to true
pub const GREATER: u8 = 0x15;
/// Less SRC SRC
/// Sets the equal_flag to 1
pub const LESS: u8 = 0x17;
/// LOAD SRC DEST
pub const LOAD: u8 = 0x16;
/// JMPF DEST
/// increments the `ip` by the value stored in DEST
pub const JMPF: u8 = 0x18;
/// JMPB DEST
/// decrements the `ip` by the value stored in DEST
pub const JMPB: u8 = 0x19;
/// JMPS if the equal flag is set;
pub const JMPEQ: u8 = 0x20;

/// ILLEGAL INST
pub const IGL:u8 = 0x0;