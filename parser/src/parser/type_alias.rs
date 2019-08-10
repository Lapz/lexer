use crate::T;

use crate::parser::Parser;

use crate::{Span, SyntaxKind::*, Token};

impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = Span<Token>>,
{
    pub(crate) fn parse_type_alias(&mut self) {
        self.start_node(TYPE_ALIAS_DEF);

        if self.peek(|t| t == T![export]) {
            self.parse_visibility();
        }

        self.expect(TYPE_KW, "Expected `type`");

        self.expect(IDENT, "Expected an identifier");

        if self.peek(|t| t == L_ANGLE) {
            self.parse_type_params();
        }

        self.expect(EQ, "Expected `=`");

        self.parse_type();

        self.expect(SEMI, "Expected `;`");

        self.finish_node()
    }
}

#[cfg(test)]
mod test {
    test_parser! {parse_type_alias,"type Foo = i32;"}
    test_parser! {parse_type_alias_params,"type ParseResult<T> = Result<T,void>;"}
}
