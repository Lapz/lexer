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
