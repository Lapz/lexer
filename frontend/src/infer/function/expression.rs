use ast as t;
use ctx::CompileCtx;
use infer::{Infer, InferResult};
use syntax::ast::{Call, ClassLiteral, Expression};
use util::pos::Spanned;

impl Infer {
    pub(crate) fn infer_expr(
        &mut self,
        expr: Spanned<Expression>,
        ctx: &mut CompileCtx,
    ) -> InferResult<Spanned<t::TypedExpression>> {
        match expr.value {
            Expression::Array { items } => self.infer_array(items, expr.span, ctx),

            Expression::Assign {
                name, kind, value, ..
            } => self.infer_assign(name, kind, value, expr.span, ctx),

            Expression::Binary { lhs, op, rhs } => self.infer_binary(lhs, op, rhs, expr.span, ctx),

            Expression::Call(call) => {
                let whole_span = expr.span;
                match call.value {
                    Call::Simple { args, callee } => {
                        self.infer_call(*callee, args, whole_span, ctx)
                    }
                    Call::Instantiation {
                        types,
                        callee,
                        args,
                    } => self.infer_call_instantiated(*callee, args, types, whole_span, ctx),
                }
            }

            Expression::Closure(function) => {
                // let returns = if let Some(ref ty) = function.value.returns {
                //     self.trans_type(&ty, ctx)?
                // } else {
                //     Type::Nil
                // };

                // let mut param_types = Vec::with_capacity(function.value.params.value.len());
                // let mut env_types = Vec::with_capacity(function.value.params.value.len());

                // for param in function.value.params.value.iter() {
                //     let ty = self.trans_type(&param.value.ty, ctx)?;

                //     env_types.push(ty.clone());
                //     param_types.push(t::FunctionParam {
                //         name: param.value.name.value,
                //         ty,
                //     })
                // }

                // let fn_signature = Type::Fun(env_types.clone(), Box::new(returns.clone()), true);

                // ctx.add_var(
                //     function.value.name.value,
                //     VarEntry::Fun {
                //         ty: fn_signature.clone(),
                //     },
                // );

                // ctx.begin_scope();

                // for param in param_types.iter() {
                //     ctx.add_var(param.name, VarEntry::Var(param.ty.clone()))
                // }

                // let span = function.value.body.span;
                // let name = function.value.name.value;
                // let body = self.infer_statement(function.value.body, ctx)?;

                // ctx.end_scope();

                // (
                //     Spanned::new(
                //         t::Expression::Closure(Box::new(t::Function {
                //             name,
                //             params: param_types,
                //             body: Box::new(body),
                //             returns: returns.clone(),
                //         })),
                //         span,
                //     ),
                //     fn_signature,
                // )

                unimplemented!()
            }

            Expression::ClassLiteral(class_literal) => {
                let whole_span = expr.span;

                match class_literal.value {
                    ClassLiteral::Simple { symbol, props } => {
                        self.infer_class_literal(symbol, props, whole_span, ctx)
                    }
                    ClassLiteral::Instantiation {
                        symbol,
                        types,
                        props,
                    } => {
                        self.infer_class_instantiated_literal(symbol, props, types, whole_span, ctx)
                    }
                }
            }

            Expression::Grouping { expr: inner } => self.infer_grouping(*inner, expr.span, ctx),

            Expression::Get { object, property } => {
                self.infer_get(*object, property, expr.span, ctx)
            }

            Expression::SubScript { target, index } => {
                self.infer_subscript(*target, *index, expr.span, ctx)
            }

            Expression::Literal(literal) => self.infer_literal(literal, expr.span),

            Expression::Set {
                object,
                name,
                value,
            } => self.infer_set(*object, name, *value, expr.span, ctx),

            Expression::Ternary {
                condition,
                then_branch,
                else_branch,
            } => self.infer_ternary(*condition, *then_branch, *else_branch, expr.span, ctx),

            Expression::Unary { expr: operand, op } => {
                self.infer_unary(op, *operand, expr.span, ctx)
            }

            Expression::Var(var) => self.infer_var(var, expr.span, ctx),
        }
    }

    //             Expression::Closure(function) => {
    //                 let returns = if let Some(ref ty) = function.value.returns {
    //                     self.trans_type(&ty, ctx)?
    //                 } else {
    //                     Type::Nil
    //                 };

    //                 let mut param_types = Vec::with_capacity(function.value.params.value.len());
    //                 let mut env_types = Vec::with_capacity(function.value.params.value.len());

    //                 for param in function.value.params.value.iter() {
    //                     let ty = self.trans_type(&param.value.ty, ctx)?;

    //                     env_types.push(ty.clone());
    //                     param_types.push(t::FunctionParam {
    //                         name: param.value.name.value,
    //                         ty,
    //                     })
    //                 }

    //                 let fn_signature =
    //                     Type::Fun(env_types.clone(), Box::new(returns.clone()), true);

    //                 ctx.add_var(
    //                     function.value.name.value,
    //                     VarEntry::Fun {
    //                         ty: fn_signature.clone(),
    //                     },
    //                 );

    //                 ctx.begin_scope();

    //                 for param in param_types.iter() {
    //                     ctx.add_var(param.name, VarEntry::Var(param.ty.clone()))
    //                 }

    //                 let span = function.value.body.span;
    //                 let name = function.value.name.value;
    //                 let body = self.infer_statement(function.value.body, ctx)?;

    //                 ctx.end_scope();

    //                 Ok((
    //                     Spanned::new(
    //                         t::Expression::Closure(Box::new(t::Function {
    //                             name,
    //                             params: param_types,
    //                             body: Box::new(body),
    //                             returns: returns.clone(),
    //                         })),
    //                         span,
    //                     ),
    //                     fn_signature,
    //                 )) // todo move to a method
    //             }
    //             _ => {
    //                 ctx.error(" Not callable", callee.span);
    //                 return Err(());
    //             }
    //         },

    //         _ => {
    //             ctx.error(" Not callable", call.span);
    //             Err(())
    //         }
    //     }
    // }
}
