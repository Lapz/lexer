use ast::expr::*;
use ast::statement::Statement;
use types::{Type, TypeError};
use env::Env;
use pos::WithPos;

type Exp = ();
#[derive(Debug)]
pub struct ExpressionType {
    pub exp: Exp,
    pub ty: Type,
}

pub fn analyse(expr: &WithPos<Statement>) -> Result<ExpressionType, TypeError> {
    trans_statement(expr)
}

fn trans_var(env: &mut Env, var: Expression) -> ExpressionType {
    unimplemented!()
}


fn trans_statement(expr: &WithPos<Statement>) -> Result<ExpressionType, TypeError>  {
    match *expr {
        Statement::ExpressionStmt(ref expr) => transform_expr(expr),
        _ => unimplemented!()
    }
}

fn transform_expr(expr: &Expression) -> Result<ExpressionType, TypeError> {
    match *expr {
        Expression::Grouping { ref expr } => transform_expr(expr),

        Expression::Binary {
            ref left_expr,
            ref right_expr,
            ..
        } => {
            let left = transform_expr(left_expr)?;
            let right = transform_expr(right_expr)?;

            check_binary_float(&left, &right)
        }

        Expression::Unary { ref expr, .. } => {
            let expr = transform_expr(expr)?;
            check_int(&expr)?;

            Ok(ExpressionType {
                exp: (),
                ty: Type::Int,
            })
        }

        Expression::Literal(ref literal) => {
            use ast::expr::Literal::*;
            match *literal {
                Float(_) => Ok(ExpressionType {
                    exp: (),
                    ty: Type::Float,
                }),
                Int(_) => Ok(ExpressionType {
                    exp: (),
                    ty: Type::Int,
                }),
                Str(_) => Ok(ExpressionType {
                    exp: (),
                    ty: Type::Str,
                }),
                True(_) | False(_) => Ok(ExpressionType {
                    exp: (),
                    ty: Type::Bool,
                }),
                Nil => Ok(ExpressionType {
                    exp: (),
                    ty: Type::Nil,
                }),
            }
        }

        _ => unimplemented!(),
    }
}

fn check_binary(left: &ExpressionType, right: &ExpressionType) -> Result<ExpressionType, TypeError> {
    check_int(left)?;
    check_int(right)?;

    Ok(ExpressionType {
        exp: (),
        ty: Type::Int,
    })
}

fn check_binary_float(
    left: &ExpressionType,
    right: &ExpressionType,
) -> Result<ExpressionType, TypeError> {
    if check_int(left).is_err() {
        check_float(left)?;
        check_float(right)?;

        return Ok(ExpressionType {
            exp: (),
            ty: Type::Float,
        });
    }

    check_int(right)?;

    Ok(ExpressionType {
        exp: (),
        ty: Type::Int,
    })
}


fn check_unary(right: ExpressionType) -> Result<ExpressionType, TypeError> {
    unimplemented!()
}

fn check_int(expr: &ExpressionType) -> Result<(), TypeError> {
    if expr.ty != Type::Int {
        return Err(TypeError::Expected(Type::Int));
    }
    Ok(())
}

fn check_float(expr: &ExpressionType) -> Result<(), TypeError> {
    if expr.ty != Type::Float {
        return Err(TypeError::Expected(Type::Int));
    }
    Ok(())
}



// fn trans_statement(env: &mut Env, statement: WithPos<Statement>) -> ExpressionType {
//     unimplemented!()
// }

fn trans_ty(env: &mut Env) -> Type {
    unimplemented!()
}
