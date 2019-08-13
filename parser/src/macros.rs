#[macro_export]
macro_rules! T {
    (break) => {
        $crate::SyntaxKind::BREAK
    };
    (continue) => {
        $crate::SyntaxKind::CONTINUE
    };
    (do) => {
        $crate::SyntaxKind::DO
    };
    (return) => {
        $crate::SyntaxKind::RETURN
    };
    (print) => {
        $crate::SyntaxKind::PRINT
    };
    (class) => {
        $crate::SyntaxKind::CLASS_KW
    };
    (true) => {
        $crate::SyntaxKind::TRUE
    };
    (false) => {
        $crate::SyntaxKind::FALSE
    };
    (enum) => {
        $crate::SyntaxKind::ENUM_KW
    };
    (void) => {
        $crate::SyntaxKind::VOID_KW
    };
    (extends) => {
        $crate::SyntaxKind::EXTENDS
    };
    (export) => {
        $crate::SyntaxKind::EXPORT_KW
    };
    (match) => {
        $crate::SyntaxKind::MATCH
    };
    (else) => {
        $crate::SyntaxKind::ELSE
    };
    (if) => {
        $crate::SyntaxKind::IF
    };
    (let) => {
        $crate::SyntaxKind::LET
    };
    (type) => {
        $crate::SyntaxKind::TYPE_KW
    };
    ("{") => {
        $crate::SyntaxKind::L_CURLY
    };

    (,) => {
        $crate::SyntaxKind::COMMA
    };
    ("}") => {
        $crate::SyntaxKind::R_CURLY
    };
    ("[") => {
        $crate::SyntaxKind::L_BRACK
    };
    ("]") => {
        $crate::SyntaxKind::R_BRACK
    };
    ("(") => {
        $crate::SyntaxKind::L_PAREN
    };
    (")") => {
        $crate::SyntaxKind::R_PAREN
    };

    (=) => {
        $crate::SyntaxKind::ASSIGN
    };
    (:) => {
        $crate::SyntaxKind::COLON
    };
    (.) => {
        $crate::SyntaxKind::DOT
    };
    (;) => {
        $crate::SyntaxKind::SEMI
    };
    (_) => {
        $crate::SyntaxKind::UNDERSCORE
    };
    (+=) => {
        $crate::SyntaxKind::PLUS_ASSIGN
    };
    (!) => {
        $crate::SyntaxKind::BANG
    };
    ("//") => {
        $crate::SyntaxKind::COMMENT
    };

    (+) => {
        $crate::SyntaxKind::PLUS
    };
    (-) => {
        $crate::SyntaxKind::MINUS
    };
    (*) => {
        $crate::SyntaxKind::STAR
    };
    (/) => {
        $crate::SyntaxKind::SLASH
    };

    (&&) => {
        $crate::SyntaxKind::AND
    };
    (==) => {
        $crate::SyntaxKind::EQUAL_EQUAL
    };
    (=>) => {
        $crate::SyntaxKind::FAT_ARROW
    };
    (<) => {
        $crate::SyntaxKind::L_ANGLE
    };
    (<=) => {
        $crate::SyntaxKind::LESS_EQUAL
    };
    (>) => {
        $crate::SyntaxKind::R_ANGLE
    };
    (>=) => {
        $crate::SyntaxKind::GREATER_EQUAL
    };
    (!=) => {
        $crate::SyntaxKind::BANG_EQUAL
    };
    (||) => {
        $crate::SyntaxKind::OR
    };

    (->) => {
        $crate::SyntaxKind::FRETURN
    };

    (fn) => {
        $crate::SyntaxKind::FN_KW
    };

    (nil) => {
        $crate::SyntaxKind::NIL_KW
    };
}

#[macro_export]
macro_rules! test_parser {
    ($f_name:ident,$test:expr) => {
        #[test]
        fn $f_name() {
            let parser_output = $crate::utils::parse($test).parse_program();
            insta::assert_snapshot_matches!($crate::utils::dump_debug(&parser_output));
        }
    };
}
