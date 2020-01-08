use syntax::T;

use crate::parser::{pratt::Precedence, Parser};

use crate::{Span, SyntaxKind::*, Token};

impl<'a, I> Parser<'a, I>
where
    I: Iterator<Item = Span<Token>>,
{
    pub(crate) fn parse_block(&mut self) {
        self.start_node(BLOCK);

        self.expect(T!["{"], "Expected `{`");

        while !self.at(EOF) && !self.at(T!["}"]) {
            match self.current() {
                T![let] => {
                    // self.start_node(EXPR_STMT);
                    self.parse_let_expr();
                    // self.finish_node();
                }
                T![if] => {
                    self.start_node(EXPR_STMT);
                    self.parse_if_expr();
                    self.finish_node();
                }
                T![do] => {
                    self.start_node(EXPR_STMT);
                    self.parse_do_expr();
                    self.finish_node();
                }
                T![while] => {
                    self.start_node(EXPR_STMT);
                    self.parse_while_expr();
                    self.finish_node();
                }
                T![return] => {
                    self.start_node(EXPR_STMT);
                    self.parse_return_expr();
                    self.finish_node();
                }
                T![break] => {
                    self.start_node(EXPR_STMT);
                    self.parse_break_expr();
                    self.finish_node();
                }
                T![continue] => {
                    self.start_node(EXPR_STMT);
                    self.parse_continue_expr();
                    self.finish_node();
                }
                T![for] => {
                    self.start_node(EXPR_STMT);
                    self.parse_for_expr();
                    self.finish_node();
                }
                T![match] => {
                    self.start_node(EXPR_STMT);
                    self.parse_match_expr();
                    self.finish_node();
                }
                pat @ WHITESPACE | pat @ T!["//"] => {
                    while self.at(pat) {
                        self.bump();
                    }
                    continue;
                }
                _ => {
                    self.start_node(EXPR_STMT);
                    self.parse_expression(Precedence::Assignment);
                    self.finish_node();
                }
            }

            if !self.at(T!["}"]) && !self.expected(T![;]) {
                break;
            }
        }

        self.expect(T!["}"], "Expected `}`");

        self.finish_node()
    }
}

#[cfg(test)]
mod test {}
