use syntax::T;

use crate::parser::Parser;

use crate::{Span, SyntaxKind::*, Token};

use crate::parser::{Precedence, PrefixParser};

#[derive(Debug)]
pub struct ClosureParselet;

impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = Span<Token>>,
{
    pub(crate) fn parse_closure_expr(&mut self) {
        self.start_node(CLOSURE_EXPR);

        self.expect(T![|], "Expected `|`");

        while !self.at(EOF) && !self.at(T![|]) {
            self.func_param();
            if !self.at(T![|]) && !self.expected(T![,]) {
                break;
            }
        }

        self.expect(T![|], "Expected `|`");

        while self.at(T![]) {
            self.bump();
        }

        if self.at(T![->]) {
            self.expect(T![->], "Expected `->`");
            self.parse_type();
        }

        self.parse_block();

        self.finish_node()
    }
}

impl<I: Iterator<Item = Span<Token>>> PrefixParser<I> for ClosureParselet {
    fn parse(&self, parser: &mut Parser<I>)
    where
        I: Iterator<Item = Span<Token>>,
    {
        parser.parse_closure_expr();
    }
}

#[cfg(test)]
mod test {
    test_parser! {parse_simple_closure_expr,r"
        fn main() {
            let a = |x,y| {
                return x+y;
            }
    }"}
    test_parser! {parse_closure_with_types,"fn main() {
        let a = |x,y| -> i32 {
            return x+y; 
        }
    }"}
    test_parser! {parse_empty_closure_expr,"fn main() {
        let a = || {};
    }"}
    test_parser! {parse_free_closure_expr,"fn main() {
        || {};
    }"}
}
