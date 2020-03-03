use syntax::T;

use crate::parser::Parser;

use crate::SyntaxKind::*;

impl<'a> Parser<'a> {
    pub(crate) fn parse_import(&mut self) {
        self.start_node(IMPORT_DEF);

        self.expect(T![import]);

        self.parse_import_segment();

        self.expect(T![;]);

        self.finish_node();
    }

    pub(crate) fn parse_import_segment(&mut self) {
        self.start_node(IMPORT_SEGMENT);

        self.ident();

        if self.at(T![::]) {
            self.bump();

            if self.at(T!["{"]) {
                self.bump();

                while !self.at(EOF) && self.at(T!["}"]) {
                    self.parse_import_segment();

                    if !self.at(T!["}"]) && !self.expected(T![,]) {
                        break;
                    }
                }

                self.expect(T!["}"]);
            }

            self.parse_import_segment();
        }

        self.finish_node();
    }
}

#[cfg(test)]
mod test {
    test_parser! {parse_empty_import,"import foo;"}
    test_parser! {parse_import_with_multiple_segments,"import foo::bar::baz;"}
    test_parser! {parse_import_with_brace,"import foo::{bar,baz};"}
    test_parser! {parse_import_with_brace_nested_segments,"import foo::{bar::baz,fubar};"}
}
