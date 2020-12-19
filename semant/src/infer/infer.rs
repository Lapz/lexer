use crate::{
    hir::{
        BinOp, BlockId, ExprId, Function, FunctionAstMap, Literal, LiteralId, PatId, StmtId,
        UnaryOp, PLACEHOLDER_NAME,
    },
    infer::{
        pattern_matrix::{PatternMatrix, Row},
        InferDataCollector, Type, TypeCon,
    },
    util, HirDatabase,
};
use std::collections::HashMap;

impl<'a, DB> InferDataCollector<&'a DB>
where
    DB: HirDatabase,
{
    pub(crate) fn infer_function(&mut self, function: &Function) {
        let expected = if let Some(ty) = self.resolver.get_type(&function.name.item) {
            ty
        } else {
            Type::Unknown
        };

        self.ctx.begin_scope();

        self.returns = Some(expected);

        self.fn_name = Some(function.name.item);

        if let Some(body) = &function.body {
            self.infer_statements(&function.ast_map, &body)
        } else {
            self.infer_statements(&function.ast_map, &[])
        };

        self.ctx.end_scope();
    }

    fn assign_pattern_type(&mut self, map: &FunctionAstMap, id: &util::Span<PatId>, ty: Type) {
        let pat = map.pat(&id.item);
        match pat {
            crate::hir::Pattern::Bind { name } => {
                self.ctx
                    .insert_type(name.item, ty, crate::resolver::TypeKind::Type)
            }
            crate::hir::Pattern::Placeholder => self.ctx.insert_type(
                self.db.intern_name(PLACEHOLDER_NAME),
                ty,
                crate::resolver::TypeKind::Type,
            ),
            crate::hir::Pattern::Tuple(idents) => match ty {
                Type::Tuple(types) => {
                    for (ident, ty) in idents.iter().zip(types.into_iter()) {
                        self.assign_pattern_type(map, ident, ty)
                    }
                }

                _ => {
                    let msg = format!("Tried assigning a non tuple type to a tuple pattern");

                    self.reporter.error(
                        msg,
                        &format!("`{:?}` is not a tuple type", ty),
                        id.as_reporter_span(),
                    );
                }
            },
            crate::hir::Pattern::Literal(_) => {}
        }
    }

    fn infer_statements(&mut self, map: &FunctionAstMap, body: &[util::Span<StmtId>]) {
        for id in body {
            let _ = self.infer_statement(map, id);
        }
    }

    fn infer_literal(&mut self, lit_id: LiteralId) -> Type {
        let lit = self.db.lookup_intern_literal(lit_id);

        match lit {
            Literal::String(_) => Type::Con(TypeCon::Str),
            Literal::Nil => Type::Con(TypeCon::Void),
            Literal::True | Literal::False => Type::Con(TypeCon::Bool),
            Literal::Int(_) => Type::Con(TypeCon::Int),
            Literal::Float(_) => Type::Con(TypeCon::Float),
        }
    }

    fn infer_block(&mut self, map: &FunctionAstMap, block_id: &BlockId, has_value: bool) -> Type {
        self.ctx.begin_scope();

        let block = map.block(block_id);

        let mut returns = Type::Con(TypeCon::Void);

        for (index, stmt) in block.0.iter().enumerate() {
            let ty = self.infer_statement(map, stmt);
            if has_value && index == block.0.len() - 1 {
                returns = ty;
            }
        }

        self.ctx.end_scope();

        returns
    }

    fn infer_statement(&mut self, map: &FunctionAstMap, id: &util::Span<StmtId>) -> Type {
        let stmt = map.stmt(&id.item);

        match stmt {
            crate::hir::Stmt::Let {
                pat,
                ascribed_type,
                initializer,
            } => {
                match (ascribed_type, initializer) {
                    (Some(expected), Some(init)) => {
                        let expr = self.infer_expr(map, init);
                        let expected = self
                            .resolver
                            .lookup_intern_type(&expected.item)
                            .unwrap_or(Type::Unknown);

                        self.unify(&expected, &expr, init.as_reporter_span(), None, true);
                        self.assign_pattern_type(map, pat, expected);
                    }
                    (Some(expected), None) => {
                        self.assign_pattern_type(
                            map,
                            pat,
                            self.resolver.lookup_intern_type(&expected.item).unwrap(),
                        );
                    }
                    (None, Some(init)) => {
                        let expected = self.infer_expr(map, init);
                        self.assign_pattern_type(map, pat, expected);
                    }
                    (None, None) => {
                        self.assign_pattern_type(map, pat, Type::Con(TypeCon::Void));
                    }
                };

                Type::Con(TypeCon::Void)
            }
            crate::hir::Stmt::Expr(expr) => self.infer_expr(map, expr),
        }
    }

    fn infer_expr(&mut self, map: &FunctionAstMap, id: &util::Span<ExprId>) -> Type {
        let expr = map.expr(&id.item);

        match expr {
            crate::hir::Expr::Array(exprs) => {
                if exprs.len() == 0 {
                    return Type::Unknown;
                }

                let first = self.infer_expr(map, &exprs[0]);

                exprs.iter().skip(1).for_each(|id| {
                    let inferred = self.infer_expr(map, id);
                    self.unify(&first, &inferred, id.as_reporter_span(), None, true)
                });

                Type::Con(TypeCon::Array {
                    ty: Box::new(first),
                    size: None,
                })
            }
            crate::hir::Expr::Binary { lhs, op, rhs } => {
                let inferred_lhs = self.infer_expr(map, lhs);
                let inferred_rhs = self.infer_expr(map, rhs);

                match op {
                    BinOp::Plus | BinOp::Minus | BinOp::Mult | BinOp::Div => {
                        self.unify(
                            &inferred_lhs,
                            &inferred_rhs,
                            id.as_reporter_span(),
                            Some("`+`,`-`,`*`,`/` operators only works on i32 and f32".into()),
                            true,
                        );
                        inferred_lhs
                    }
                    BinOp::And | BinOp::Or => {
                        self.unify(
                            &inferred_lhs,
                            &Type::Con(TypeCon::Bool),
                            lhs.as_reporter_span(),
                            Some("Expected a bool".into()),
                            true,
                        );
                        self.unify(
                            &inferred_rhs,
                            &Type::Con(TypeCon::Bool),
                            rhs.as_reporter_span(),
                            Some("Expected a bool".into()),
                            true,
                        );

                        Type::Con(TypeCon::Bool)
                    }
                    BinOp::LessThan
                    | BinOp::GreaterThan
                    | BinOp::GreaterThanEqual
                    | BinOp::LessThanEqual => {
                        self.unify(
                            &inferred_lhs,
                            &inferred_rhs,
                            id.as_reporter_span(),
                            Some("Comparison operators only work on numbers".into()),
                            true,
                        );
                        Type::Con(TypeCon::Bool)
                    }
                    BinOp::Equal => {
                        self.unify(
                            &inferred_lhs,
                            &inferred_rhs,
                            id.as_reporter_span(),
                            None,
                            true,
                        );

                        inferred_rhs
                    }
                    BinOp::EqualEqual | BinOp::NotEqual => Type::Con(TypeCon::Bool),
                    BinOp::PlusEqual | BinOp::MinusEqual | BinOp::MultEqual | BinOp::DivEqual => {
                        self.unify(
                            &inferred_lhs,
                            &inferred_rhs,
                            id.as_reporter_span(),
                            None,
                            true,
                        );

                        inferred_rhs
                    }
                }
            }

            crate::hir::Expr::Block(block, has_value) => self.infer_block(map, block, *has_value),
            crate::hir::Expr::Break | crate::hir::Expr::Continue => Type::Con(TypeCon::Void),
            crate::hir::Expr::Call {
                callee,
                args,
                type_args,
            } => {
                let inferred_callee = self.infer_expr(map, callee);

                match inferred_callee {
                    Type::Poly(vars, inner) => match &*inner {
                        Type::App(arg_types) => {
                            let ret = arg_types
                                .last()
                                .unwrap_or(&Type::Con(TypeCon::Void))
                                .clone();

                            if args.len() > arg_types.len() - 1 {
                                self.reporter.error(
                                    "Missing arguments",
                                    format!(
                                        "Too many arguments,expected `{}` found `{}",
                                        arg_types.len(),
                                        args.len()
                                    ),
                                    callee.as_reporter_span(),
                                );
                            } else if args.len() < arg_types.len() - 1 {
                                self.reporter.error(
                                    "Missing arguments",
                                    format!(
                                        "Too few arguments,expected `{}` found `{}",
                                        arg_types.len(),
                                        args.len()
                                    ),
                                    callee.as_reporter_span(),
                                );
                            }

                            let mut subst = HashMap::new();

                            let mut callee_exprs = Vec::new();

                            for (var, type_arg) in vars.iter().zip(type_args.item.iter()) {
                                let ty = self
                                    .resolver
                                    .lookup_intern_type(&type_arg.item)
                                    .unwrap_or(Type::Unknown);

                                subst.insert(*var, ty);
                            }

                            if type_args.item.is_empty() {
                                for (var, expr) in vars.iter().zip(args.iter()) {
                                    let inferred_expr = self.infer_expr(map, expr);
                                    callee_exprs.push(inferred_expr.clone());

                                    subst.insert(*var, inferred_expr);
                                }
                            }

                            arg_types
                                .iter()
                                .zip(args.iter().zip(callee_exprs.iter()))
                                .for_each(|(ty, (expr, expr_ty))| {
                                    let inferred_expr = expr_ty;

                                    let inferred_expr = self.subst(&inferred_expr, &mut subst);

                                    let ty = self.subst(ty, &mut subst);

                                    self.unify(
                                        &ty,
                                        &inferred_expr,
                                        expr.as_reporter_span(),
                                        Some("test".into()),
                                        false,
                                    )
                                });

                            self.subst(&ret, &mut subst)
                        }
                        ty => {
                            match ty {
                                Type::Unknown => {} // use of undefined function
                                _ => {
                                    let msg = format!("Expected a function found `{:?}`", ty);

                                    self.reporter.error(
                                        msg,
                                        "A call expression requires a function",
                                        callee.as_reporter_span(),
                                    );
                                }
                            }

                            Type::Unknown
                        }
                    },
                    Type::App(arg_types) => {
                        // TODO check if code path is reachable
                        let ret = arg_types
                            .last()
                            .unwrap_or(&Type::Con(TypeCon::Void))
                            .clone();

                        if args.len() > arg_types.len() - 1 {
                            self.reporter.error(
                                "Missing arguments",
                                format!(
                                    "Too many arguments,expected `{}` found `{}",
                                    arg_types.len(),
                                    args.len()
                                ),
                                callee.as_reporter_span(),
                            );
                        } else if args.len() < arg_types.len() - 1 {
                            self.reporter.error(
                                "Missing arguments",
                                format!(
                                    "Too few arguments,expected `{}` found `{}",
                                    arg_types.len(),
                                    args.len()
                                ),
                                callee.as_reporter_span(),
                            );
                        }

                        arg_types.iter().zip(args.iter()).for_each(|(ty, expr)| {
                            let inferred_expr = self.infer_expr(map, expr);
                            self.unify(ty, &inferred_expr, expr.as_reporter_span(), None, true)
                        });

                        ret
                    }

                    ty => {
                        match ty {
                            Type::Unknown => {}
                            _ => {
                                let msg = format!("Expected a function found `{:?}`", ty);

                                self.reporter.error(
                                    msg,
                                    "A call expression requires a function",
                                    callee.as_reporter_span(),
                                );
                            }
                        }

                        Type::Unknown
                    }
                }
            }
            crate::hir::Expr::Cast { expr, ty } => {
                let _ = self.infer_expr(map, expr);

                // TODO implement a well formed casting check

                self.resolver
                    .lookup_intern_type(&ty.item)
                    .unwrap_or(Type::Unknown)
            }

            crate::hir::Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let inferred_cond = self.infer_expr(map, cond);

                self.unify(
                    &inferred_cond,
                    &Type::Con(TypeCon::Bool),
                    cond.as_reporter_span(),
                    Some("Expected the type of the expression to be bool".into()),
                    true,
                );

                let inferred_then = self.infer_expr(map, then_branch);

                if let Some(else_b) = else_branch {
                    let inferred_else = self.infer_expr(map, else_b);

                    self.unify(
                        &inferred_then,
                        &inferred_else,
                        id.as_reporter_span(),
                        Some("One of the branches is of different type".into()),
                        true,
                    )
                }

                inferred_then
            }
            crate::hir::Expr::Ident(name) => self.ctx.get_type(&name.item).unwrap_or(
                self.resolver
                    .get_param(&self.fn_name.unwrap(), &name.item)
                    .unwrap_or(Type::Unknown),
            ),
            crate::hir::Expr::Index { base, index } => {
                let inferred_base = self.infer_expr(map, base);

                let inferred_index = self.infer_expr(map, index);

                self.unify(
                    &inferred_index,
                    &Type::Con(TypeCon::Int),
                    index.as_reporter_span(),
                    Some("Array indexes can only be an integer".into()),
                    true,
                );

                match inferred_base {
                    Type::Con(TypeCon::Array { ty, .. }) => *ty,
                    _ => {
                        let msg = format!("Tried indexing a non array type");

                        self.reporter.error(
                            msg,
                            &format!("`{:?}` is not indexable", inferred_base),
                            id.as_reporter_span(),
                        );

                        Type::Unknown
                    }
                }
            }
            crate::hir::Expr::While { cond, body } => {
                let inferred_cond = self.infer_expr(map, cond);

                self.unify(
                    &inferred_cond,
                    &Type::Con(TypeCon::Bool),
                    cond.as_reporter_span(),
                    Some("Expected the type of the expression to be bool".into()),
                    true,
                );

                self.infer_block(map, body, false);

                Type::Con(TypeCon::Void)
            }
            crate::hir::Expr::Literal(literal) => self.infer_literal(*literal),
            crate::hir::Expr::Paren(inner) => self.infer_expr(map, inner),
            crate::hir::Expr::Tuple(exprs) => {
                let types = exprs.iter().map(|id| self.infer_expr(map, id)).collect();
                Type::Tuple(types)
            }
            crate::hir::Expr::Unary { op, expr } => {
                let inferred = self.infer_expr(map, id);
                match op {
                    UnaryOp::Minus => match inferred {
                        Type::Con(TypeCon::Int) | Type::Con(TypeCon::Float) => inferred,

                        _ => {
                            let msg = format!("Cannot use `-` operator on type `{:?}`", inferred);
                            self.reporter.error(
                                msg,
                                "`-` only works on i32,f32,",
                                expr.as_reporter_span(),
                            );

                            Type::Con(TypeCon::Int)
                        }
                    },
                    UnaryOp::Excl => {
                        self.unify(
                            &Type::Con(TypeCon::Bool),
                            &inferred,
                            expr.as_reporter_span(),
                            Some("`!` can only be used on a boolean expression".into()),
                            true,
                        );
                        Type::Con(TypeCon::Bool)
                    }
                }
            }
            crate::hir::Expr::Return(expr) => {
                let expected = self.returns.clone().unwrap();
                if let Some(id) = expr {
                    let inferred = self.infer_expr(map, id);

                    self.unify(
                        &expected,
                        &inferred,
                        id.as_reporter_span(),
                        Some(format!(
                            "{:?} is returned here but expected {:?}",
                            inferred, expected
                        )),
                        true,
                    );
                    inferred
                } else {
                    let inferred = Type::Con(TypeCon::Void);
                    self.unify(
                        &expected,
                        &inferred,
                        id.as_reporter_span(),
                        Some(format!(
                            "{:?} is returned here but expected {:?}",
                            inferred, expected
                        )),
                        true,
                    );
                    inferred
                }
            }
            crate::hir::Expr::Match { expr, arms } => {
                let inferred_expr = self.infer_expr(map, expr);

                let mut matrix = PatternMatrix::new();

                for match_arm in arms {
                    let mut patterns = vec![];

                    for pattern in &match_arm.pats {
                        let pat = map.pat(&pattern.item);

                        patterns.push(self.to_matrix_pattern(pat, map))
                    }

                    matrix.add_row(Row::new(patterns, match_arm.expr.item))
                }

                Type::Unknown
            }
            crate::hir::Expr::Enum { def, variant, expr } => {
                let inferred_def = self.ctx.get_type(&def.item).unwrap_or(Type::Unknown);
                match inferred_def {
                    Type::Enum(_, ref variants) => match variants.get(&variant.item) {
                        Some(v) => {
                            if let Some(v_ty) = &v.ty {
                                if let Some(expr) = expr {
                                    let inferred_expr = self.infer_expr(map, expr);
                                    self.unify(
                                        &inferred_expr,
                                        v_ty,
                                        expr.as_reporter_span(),
                                        None,
                                        true,
                                    )
                                } else {
                                    let msg = format!("Missing enum variant constructor",);
                                    self.reporter.error(
                                        msg,
                                        format!("Expected an enum variant constructor of type {:?} but found none",v_ty),
                                        variant.as_reporter_span(),
                                    );
                                }
                            }

                            inferred_def
                        }

                        None => {
                            // Error reported in resolver
                            inferred_def
                        }
                    },
                    _ => {
                        // Error reported in resolver
                        Type::Unknown
                    }
                }
            }
            crate::hir::Expr::RecordLiteral { def, fields } => {
                let inferred_def = self.ctx.get_type(&def.item).unwrap_or(Type::Unknown);

                match inferred_def.clone() {
                    Type::Poly(_, inner) => match &*inner {
                        Type::Class {
                            fields: field_types,
                            ..
                        } => {
                            for (field, expr) in fields {
                                let inferred_expr = self.infer_expr(map, expr);

                                let expected =
                                    field_types.get(&field.item).unwrap_or(&Type::Unknown);

                                self.unify(
                                    expected,
                                    &inferred_expr,
                                    field.as_reporter_span(),
                                    None,
                                    true,
                                );
                            }

                            inferred_def
                        }

                        _ => {
                            // Error reported in resolver
                            Type::Unknown
                        }
                    },
                    _ => {
                        // Error reported in resolver
                        Type::Unknown
                    }
                }
            }
            crate::hir::Expr::Field(fields) => {
                let record = self.infer_expr(map, &fields[0]);
                match record {
                    Type::Poly(_, inner) => match &*inner {
                        ty @ Type::Class { .. } => {
                            // resolve method chain                            // self.infer_field_exprs(&fields[1..], map);

                            for id in &fields[1..] {
                                println!("{:?}", map.expr(&id.item))
                            }

                            self.infer_field_exprs(&fields[1..], ty, map)
                        }
                        Type::Unknown => Type::Unknown,
                        ty => {
                            let msg = format!(
                                "Expected expression of type `class` instead found `{:?}`",
                                ty
                            );

                            self.reporter.error(
                                msg,
                                "Field access only works on classes",
                                fields[0].as_reporter_span(),
                            );

                            Type::Unknown
                        }
                    },

                    ty @ Type::Tuple(_) => self.infer_field_exprs(&fields[1..], &ty, map),
                    Type::Unknown => Type::Unknown,
                    _ => {
                        let msg = format!(
                            "Expected expression of type `class` instead found `{:?}`",
                            record
                        );

                        self.reporter.error(
                            msg,
                            "Field access only works on classes",
                            fields[0].as_reporter_span(),
                        );

                        Type::Unknown
                    }
                }
            }
        }
    }

    fn infer_field_exprs(
        &mut self,
        exprs: &[util::Span<ExprId>],
        ty: &Type,
        map: &FunctionAstMap,
    ) -> Type {
        if exprs.is_empty() {
            return ty.clone();
        }

        let expr = map.expr(&exprs[0].item);

        match expr {
            crate::hir::Expr::Call {
                callee,
                args,
                type_args,
            } => unimplemented!(),
            crate::hir::Expr::Ident(ident) => match ty {
                Type::Class { fields, .. } => {
                    if let Some(ty) = fields.get(&ident.item) {
                        println!("{:?}", ty);
                        self.infer_field_exprs(&exprs[1..], ty, map)
                    } else {
                        let msg = format!(
                            "Unknown record field `{}`",
                            self.db.lookup_intern_name(ident.item),
                        );

                        self.reporter.error(msg, "", exprs[0].as_reporter_span());

                        Type::Unknown
                    }
                }
                ty => {
                    let msg = format!(
                        "`{}` does not exist on `{:?}`",
                        self.db.lookup_intern_name(ident.item),
                        ty
                    );

                    self.reporter.error(msg, "", exprs[0].as_reporter_span());

                    Type::Unknown
                }
            },

            crate::hir::Expr::Literal(literal) => {
                let lit = self.db.lookup_intern_literal(*literal);
                match lit {
                    Literal::Int(int) => {
                        let index: usize = int.parse().expect("Couldn't parse to i32");

                        match ty {
                            Type::Tuple(types) => {
                                if let Some(ty) = types.get(index) {
                                    self.infer_field_exprs(&exprs[1..], ty, map)
                                } else {
                                    let msg = format!("Unknown tuple field `{}`", index);

                                    self.reporter.error(msg, "", exprs[0].as_reporter_span());

                                    Type::Unknown
                                }
                            }
                            _ => {
                                let msg = format!("`{}` does not exist on `{:?}`", index, ty);

                                self.reporter.error(
                                    msg,
                                    "Numbered field access can only be used on tuples",
                                    exprs[0].as_reporter_span(),
                                );

                                Type::Unknown
                            }
                        }
                    }
                    _ => {
                        let msg = format!("`{:?}` is not a valid tuple field ", lit);

                        self.reporter.error(
                            msg,
                            "Tuple fields can only be numbers",
                            exprs[0].as_reporter_span(),
                        );

                        Type::Unknown
                    }
                }
            }

            _ => {
                // invalid field types
                // todo error
                Type::Unknown
            }
        }
    }
}
