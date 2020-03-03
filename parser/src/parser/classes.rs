use syntax::T;

use crate::parser::Parser;

use crate::{Span, SyntaxKind::*, Token};

impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = Span<Token>>,
{
    pub(crate) fn parse_class(&mut self, checkpoint: rowan::Checkpoint) {
        self.start_node_at(checkpoint, CLASS_DEF);

        self.expect(CLASS_KW, "Expected `class`");

        self.ident();

        if self.is_ahead(|t| t == T![<]) {
            self.parse_type_params(false);
        }

        self.parse_class_body();

        self.finish_node()
    }

    fn parse_class_body(&mut self) {
        // self.start_node()
        self.expect(T!["{"], "Expected `{`");

        while !self.at(EOF) && !self.at(T!["}"]) {
            let has_visibility = self.has_visibility();

            if has_visibility {
                let checkpoint = self.checkpoint();
                self.parse_visibility();

                match self.current() {
                    IDENT => self.parse_named_field(),
                    T![fn] => self.parse_function(checkpoint),
                    T![] => {
                        self.bump();
                        continue;
                    }
                    _ => self.error(
                        "Expected an identifier | `pub` | `fn` ",
                        format!(
                            "Expected an identifier | `pub` | `fn`  but instead found `{}`",
                            self.current_string()
                        ),
                    ),
                }
            } else {
                let checkpoint = self.checkpoint();
                match self.current() {
                    IDENT => self.parse_named_field(),
                    T![fn] => self.parse_function(checkpoint),
                    T![] => {
                        self.bump();
                        continue;
                    }
                    _ => self.error(
                        "Expected an identifier | `pub` | `fn` ",
                        format!(
                            "Expected an identifier | `pub` | `fn`  but instead found `{}`",
                            self.current_string()
                        ),
                    ),
                }
            }
        }

        self.expect(T!["}"], "Expected `}`");
    }

    fn parse_named_field(&mut self) {
        self.start_node(NAMED_FIELD_DEF);
        self.ident();
        self.expect(T![:], "Expected `:`");
        self.parse_type();
        self.expect(T![;], "Expected `;`");
        self.finish_node();
    }
}

#[cfg(test)]
mod test {

    use crate::utils::parse;
    use syntax::{ClassDefOwner, NamedFieldsOwner};
    #[test]
    fn test_class_fields() {
        let source_file = parse("class Person { name:String; surname:String;}").parse_program();

        let class = source_file.classes().next().unwrap();

        assert_eq!(class.fields().count(), 2)
    }
    test_parser! {parse_empty_class,"class Foo {}"}
    test_parser! {parse_class_generic,"class Result<T,E> {}"}
    test_parser! {parse_class_fields,"class Person { name:String; surname:String;}"}
    test_parser! {parse_class_fields_methods,"class Person { name:String; surname:String; fn hello(self) {}}"}
    test_parser! {parse_class_methods,"class Person { name:String; surname:String; fn new() -> Person {}}"}
}
