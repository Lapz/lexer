mod binary;
mod block;
mod call;
mod ctx;
mod field;
mod infer;
mod pattern_matrix;
mod stacked_map;
mod subst;
mod ty;
mod unify;

use crate::{hir::NameId, infer2, resolver::Resolver, HirDatabase};
pub use ctx::Ctx;
use errors::{FileId, Reporter, WithError};
pub(crate) use stacked_map::StackedMap;
use std::{collections::HashMap, sync::Arc};
pub(crate) use ty::{Type, TypeCon, TypeVar, Variant};

#[derive(Debug)]
pub(crate) struct InferDataCollector<DB> {
    db: DB,
    ctx: Ctx,
    resolver: Arc<Resolver>,
    reporter: Reporter,
    returns: Option<Type>,
    possible_returns: Vec<Type>,
    fn_name: Option<NameId>,
}

pub fn infer_query(db: &impl HirDatabase, file: FileId) -> WithError<()> {
    let WithError(program, mut errors) = db.lower(file);
    let WithError(resolver, error) = db.resolve_source_file(file);
    let reporter = Reporter::new(file);
    errors.extend(error);

    let ctx = resolver.ctx.clone();

    let mut collector = InferDataCollector {
        db,
        ctx,
        resolver,
        reporter,
        returns: None,

        fn_name: None,
        possible_returns: Vec::new(),
    };

    let mut unify = infer2::Unify {
        context: infer2::Map::new(),
        errors: vec![],
        tvar_count: 0,
    };

    println!("{:?}", unify);

    unify.insert(TypeVar(2), infer2::Type::Con(infer2::TypeCon::Int));

    let mut fields = HashMap::new();

    fields.insert(NameId(2u32.into()), infer2::Type::Var(TypeVar(2)));

    fields.insert(
        NameId(3u32.into()),
        infer2::Type::Class {
            name: NameId(0u32.into()),
            fields: HashMap::new(),
            methods: HashMap::new(),
        },
    );

    println!(
        "{:?}",
        unify.subst(&infer2::Type::Class {
            name: NameId(0u32.into()),
            fields,
            methods: HashMap::new(),
        })
    );

    for function in &program.functions {
        collector.infer_function(function);
    }

    errors.extend(collector.reporter.finish());

    WithError((), errors)
}
