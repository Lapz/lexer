#[cfg(test)]
extern crate pretty_assertions;

extern crate fnv;
extern crate ir;
extern crate opcode;
extern crate rand;
extern crate syntax;
extern crate util;
extern crate vm;

#[macro_use]
// mod resolver;
// mod test;
mod ast;
mod ctx;
mod infer;
mod lower;

pub use infer::Infer;
