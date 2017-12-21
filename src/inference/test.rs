#[cfg(test)]

mod test {
    use types::Type;
    use inference::{analyse, ExpressionType};
    use pos::WithPos;
    use ast::statement::Statement;
    use std::rc::Rc;
    use symbol::SymbolFactory;
    use env::Env;

    fn get_ast(input: &str, strings: Rc<SymbolFactory>) -> Vec<WithPos<Statement>> {
        use lexer::Lexer;
        use parser::Parser;
        use symbol::Symbols;

        let tokens = Lexer::new(input).lex().unwrap();

        let mut symbols = Symbols::new(strings);
        Parser::new(tokens, &mut symbols).parse().unwrap()
    }

    #[test]
    fn is_int() {
        let input = "123+456;";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        assert_eq!(
            analyse(&get_ast(input, strings), &mut env).unwrap(),
            vec![
                ExpressionType {
                    exp: (),
                    ty: Type::Int,
                },
            ]
        );
    }

    #[test]
    fn is_float() {
        let input = "123.0+456.0;";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        assert_eq!(
            analyse(&get_ast(input, strings), &mut env).unwrap(),
            vec![
                ExpressionType {
                    exp: (),
                    ty: Type::Float,
                },
            ]
        );
    }

    #[test]
    #[should_panic]
    fn float_int() {
        let input = "123.0+456;";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        analyse(&get_ast(input, strings), &mut env).unwrap();
    }

    #[test]
    #[should_panic]
    fn int_str() {
        let input = "10+\"h\";";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        analyse(&get_ast(input, strings), &mut env).unwrap();
    }

    #[test]
    #[should_panic]
    fn float_str() {
        let input = "10.0+\"h\";";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        analyse(&get_ast(input, strings), &mut env).unwrap();
    }

    #[test]
    #[should_panic]
    fn too_little_instance() {
        let input = "class Person {name:str,surname:str,age:int;fun hello(a:int,b:int){nil;}}
        var lenard = Person{name:\"Lenard\"};";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        analyse(&get_ast(input, strings), &mut env).unwrap();
    }

    #[test]
    #[should_panic]
    fn wrong_set_instance_type() {
        let input = "class Person {name:str,surname:str,age:int;fun hello(a:int,b:int){nil;}}
        var lenard = Person{name:\"Lenard\"};lenard.name = 10";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        analyse(&get_ast(input, strings), &mut env).unwrap();
    }

    #[test]

    fn set_instance_type() {
        let input = "class Person {name:str;}
        var lenard = Person{name:\"Lenard\"};lenard.name = \"h\";";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        assert_eq!(
            analyse(&get_ast(input, strings), &mut env).unwrap()[2],
            ExpressionType {
                exp: (),
                ty: Type::Str,
            }
        );
    }

    #[test]
    #[should_panic]
    fn too_many_instance() {
        let input = "class Person {name:str,surname:str,age:int;fun hello(a:int,b:int){nil;}}
        var lenard = Person{name:\"Lenard\",name:\"Lenard\",name:\"Lenard\",name:\"Lenard\"};";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        analyse(&get_ast(input, strings), &mut env).unwrap();
    }

    #[test]
    #[should_panic]
    fn wrong_body_type() {
        let input = "fun add(a:int,b:int) {a+b;}";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        analyse(&get_ast(input, strings), &mut env).unwrap();
    }

    #[test]
    #[should_panic]
    fn unary() {
        let input = "!true;!false!";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        assert_eq!(
            analyse(&get_ast(input, strings), &mut env).unwrap(),
            vec![
                ExpressionType {
                    exp: (),
                    ty: Type::Bool,
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Bool,
                },
            ]
        );
    }

    #[test]
    #[should_panic]
    fn wrong_unary_str() {
        let input = "!\"h\";";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        analyse(&get_ast(input, strings), &mut env).unwrap();
    }

    #[test]
    fn array_index() {
        let input = "var a = [10]; a[0];";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        assert_eq!(
            analyse(&get_ast(input, strings), &mut env).unwrap(),
            vec![
                ExpressionType {
                    exp: (),
                    ty: Type::Array(Box::new(Type::Int)),
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Int,
                },
            ]
        );
    }

    #[test]
    fn str_index() {
        let input = "var a = \"h\"; a[0];";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        assert_eq!(
            analyse(&get_ast(input, strings), &mut env).unwrap(),
            vec![
                ExpressionType {
                    exp: (),
                    ty: Type::Str,
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Str,
                },
            ]
        );
    }

    #[test]
    #[should_panic]
    fn invalid_index_expr_int() {
        let input = "var a = 10; a[0];";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        analyse(&get_ast(input, strings), &mut env).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_index_expr_float() {
        let input = "var a = 10.0; a[0];";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        analyse(&get_ast(input, strings), &mut env).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_index_expr_true() {
        let input = "var a = true; a[0];";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        analyse(&get_ast(input, strings), &mut env).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_index_expr_false() {
        let input = "var a = false; a[0];";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        analyse(&get_ast(input, strings), &mut env).unwrap();
    }

    #[test]
    fn func_expr() {
        let input = "var add = fun(a:int,b:int) -> int {a+b;};";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        assert_eq!(
            analyse(&get_ast(input, strings), &mut env).unwrap(),
            vec![
                ExpressionType {
                    exp: (),
                    ty: Type::Int,
                },
            ]
        );
    }

    #[test]
    fn func_type_alias() {
        let input = "
        type Int = int;
        fun add(a:Int,b:Int) -> Int {
            a+b;
        }
        add(10,10);";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        assert_eq!(
            analyse(&get_ast(input, strings), &mut env).unwrap(),
            vec![
                ExpressionType {
                    exp: (),
                    ty: Type::Int,
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Int,
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Int,
                },
            ]
        );
    }

    #[test]
    fn func_type_alias_actual() {
        let input = "
        type Int = int;
        fun add(a:Int,b:Int) -> int {
            a+b;
        }
        add(10,10);";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        assert_eq!(
            analyse(&get_ast(input, strings), &mut env).unwrap(),
            vec![
                ExpressionType {
                    exp: (),
                    ty: Type::Int,
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Int,
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Int,
                },
            ]
        );
    }

    #[test]
    fn func_call() {
        let input = "fun add(a:int,b:int) -> int {a+b;} add(10,10);";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        assert_eq!(
            analyse(&get_ast(input, strings), &mut env).unwrap(),
            vec![
                ExpressionType {
                    exp: (),
                    ty: Type::Int,
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Int,
                },
            ]
        );
    }

    #[test]
    fn assign_types() {
        let input =
            "var a = 10; var b = 10.0; var c = nil; var d = \"h\"; var e = true; var f = false; ";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        assert_eq!(
            analyse(&get_ast(input, strings), &mut env).unwrap(),
            vec![
                ExpressionType {
                    exp: (),
                    ty: Type::Int,
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Float,
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Nil,
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Str,
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Bool,
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Bool,
                },
            ]
        );
    }

    #[test]
    fn array_types() {
        let input =
            "var a = [10]; var b = [10.0]; var c = [nil]; var d = [\"h\"]; var e = [true]; var f = [false]; ";
        let strings = Rc::new(SymbolFactory::new());
        let mut env = Env::new(&strings);
        assert_eq!(
            analyse(&get_ast(input, strings), &mut env).unwrap(),
            vec![
                ExpressionType {
                    exp: (),
                    ty: Type::Array(Box::new(Type::Int)),
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Array(Box::new(Type::Float)),
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Array(Box::new(Type::Nil)),
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Array(Box::new(Type::Str)),
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Array(Box::new(Type::Bool)),
                },
                ExpressionType {
                    exp: (),
                    ty: Type::Array(Box::new(Type::Bool)),
                },
            ]
        );
    }

}
