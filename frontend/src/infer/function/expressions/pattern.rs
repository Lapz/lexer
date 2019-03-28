use crate::ast as t;
use crate::ctx::CompileCtx;

use crate::infer::types::Type;
use crate::infer::{Infer, InferResult};
use syntax::ast::{Expression, MatchArm};
use util::pos::{Span, Spanned};

impl Infer {
    pub(crate) fn infer_match(
        &mut self,
        cond: Spanned<Expression>,
        arms: Spanned<Vec<Spanned<MatchArm>>>,
        whole_span: Span,
        ctx: &mut CompileCtx,
    ) -> InferResult<Spanned<t::TypedExpression>> {
//        let cond = self.infer_expr(cond, ctx)?;
//
//        let pattern_type = cond.value.ty.clone(); // type of the pattern
//
//        let mut return_type = Type::Nil; // Default return type is nill
//
//        let mut typed_arms = Vec::new();
//        let arms_span = arms.span;
//        let mut arms = arms.value;
//
//        if !arms.is_empty() {
//            let first = arms.remove(0);
//            let first_span = first.span;
//
//            if first.value.pattern.is_none() {
//                let first_body = self.infer_statement(first.value.body, ctx)?;
//                return_type = first_body.value.ty.clone();
//
//                typed_arms.push(Spanned::new(
//                    t::MatchArm {
//                        pattern: None,
//                        body: first_body,
//                        is_all: true,
//                    },
//                    first_span,
//                ));
//            } else {
//                let first_pattern = self.infer_expr(first.value.pattern.unwrap(), ctx)?;
//                let first_body = self.infer_statement(first.value.body, ctx)?;
//
//                return_type = first_body.value.ty.clone();
//
//                typed_arms.push(Spanned::new(
//                    t::MatchArm {
//                        pattern: Some(first_pattern),
//                        body: first_body,
//                        is_all: false,
//                    },
//                    first_span,
//                ));
//            }
//
//            for arm in arms {
//                let span = arm.span;
//                let is_all = arm.value.is_all;
//
//                if is_all {
//                    typed_arms.push(Spanned::new(
//                        t::MatchArm {
//                            pattern: None,
//                            body: self.infer_statement(arm.value.body, ctx)?,
//                            is_all: true,
//                        },
//                        span,
//                    ));
//                } else {
//                    let lhs = self.infer_expr(arm.value.pattern.unwrap(), ctx)?; // unwrap
//
//                    self.unify(&lhs.value.ty, &pattern_type, lhs.span, ctx)?;
//
//                    let rhs = self.infer_statement(arm.value.body, ctx)?;
//
//                    self.unify(&rhs.value.ty, &return_type, rhs.span, ctx)?;
//
//                    typed_arms.push(Spanned::new(
//                        t::MatchArm {
//                            pattern: Some(lhs),
//                            body: rhs,
//                            is_all: false,
//                        },
//                        span,
//                    ));
//                }
//            }
//        }
//
//        Ok(Spanned::new(
//            t::TypedExpression {
//                expr: Box::new(Spanned::new(
//                    t::Expression::Match {
//                        cond,
//                        arms: Spanned::new(typed_arms, arms_span),
//                    },
//                    whole_span,
//                )),
//                ty: return_type,
//            },
//            whole_span,
//        ))

        unimplemented!()
    }
}
