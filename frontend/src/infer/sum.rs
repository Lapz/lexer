use crate::ctx::CompileCtx;

use crate::infer::types::{Type, TypeVar, Variant};
use crate::infer::{Infer, InferResult};
use std::collections::HashMap;
use syntax::ast::Enum;
use util::pos::Spanned;

impl Infer {
    pub fn infer_enum(&mut self, _enum: Spanned<Enum>, ctx: &mut CompileCtx) -> InferResult<()> {
        let mut generic_type_vars = Vec::with_capacity(_enum.value.name.value.type_params.len());
        let mut type_params = Vec::with_capacity(_enum.value.name.value.type_params.len());

        ctx.begin_scope();

        for symbol in _enum.value.name.value.type_params.iter() {
            let type_var = TypeVar::new();
            ctx.add_type(symbol.value, Type::Var(type_var));
            generic_type_vars.push(type_var);
            type_params.push(symbol.value)
        }

        ctx.add_type(
            _enum.value.name.value.name.value,
            Type::Generic(
                generic_type_vars.clone(),
                Box::new(Type::Enum {
                    name: _enum.value.name.value.name.value,
                    variants: HashMap::new(),
                }),
            ),
        ); // For recursive types we need to add the empty enum

        // let mut variants = HashMap::with_capacity(_enum.value.variants.len());

        // for (i, variant) in _enum.value.variants.into_iter().enumerate() {
        //     let inner = if let Some(ref ty) = variant.inner {
        //         Some(self.trans_type(ty, ctx)?)
        //     } else {
        //         None
        //     };

        //     let v = Variant {
        //         tag: i as u32,
        //         inner,
        //     };

        //     variants.insert(variant.name.value, v);
        // }

        ctx.end_scope();

        ctx.add_type(
            _enum.value.name.value.name.value,
            Type::Generic(
                generic_type_vars.clone(),
                Box::new(Type::Enum {
                    name: _enum.value.name.value.name.value,
                    variants: HashMap::new(),
                }),
            ),
        ); // Add the proper fully type checked enum

        Ok(())
    }
}
