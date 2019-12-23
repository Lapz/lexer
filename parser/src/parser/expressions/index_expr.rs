use syntax::T;

use crate::parser::pratt::{InfixParser, Precedence};
use crate::parser::Parser;

use crate::{Span, SyntaxKind::*, Token};

#[derive(Debug)]
pub struct IndexParselet(pub Precedence);

impl<I: Iterator<Item = Span<Token>>> InfixParser<I> for IndexParselet {
    fn parse(&self, parser: &mut Parser<I>, checkpoint: rowan::Checkpoint) {
        parser.start_node_at(checkpoint, INDEX_EXPR);

        parser.expect(T!["["], "Expected `[`");

        parser.parse_expression(Precedence::Assignment);

        parser.expect(T!["]"], "Expected `]`");

        parser.finish_node();
    }

    fn pred(&self) -> Precedence {
        self.0
    }
}

#[cfg(test)]
mod test {
    test_parser! {parse_simple_index_expr,"fn main() {a[1+2];}"}
    test_parser! {parse_nested_index_expr,"fn main() {a[0][1];}"}
}
