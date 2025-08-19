use super::*;

pub type ParamDecl = (DType, Option<Ident>);

#[derive(Clone)]
pub enum Declarator {
    Abstract,
    Var(Ident),
    Deref(usize, Box<Self>),
    Call(Box<Self>, Vec<ParamDecl>),
    Index(Box<Self>, u32),
    Unsized(Box<Self>),
}

impl ASTNode for Declarator {
    fn parse(pair: Pair<Rule>) -> Self {
        use Declarator::*;
        parser_rule! {
            pair:

            abstract_declarator
            | declarator
                [decl] -> decl;
                [p:Self, d:Self] -> {
                    let Deref(n, b) = p else {
                        unreachable!()
                    };
                    Deref(n, d.into())
                };

            pointer
                [.. ps,] -> {
                    let level_of_indirection = ps.filter(|token|
                        token.as_str() == "*"
                    ).count();

                    Deref(level_of_indirection, Abstract.into())
                };

            direct_abstract_declarator
            | direct_declarator
                [.. exts,] -> {
                    let mut exts : Vec<_> = exts.collect();

                    let base_pair = &exts[0];
                    let rule = base_pair.as_rule();

                    let mut base = match rule {
                        declarator | abstract_declarator => Self::parse(exts.remove(0)),
                        IDENTIFIER => Var((exts.remove(0).as_str().into())),
                        brackets | const_sized | sized | params => Abstract,
                        r => unreachable!("{:?}", r),
                    };

                    for ext in exts {
                        base = match ext.as_rule() {
                            brackets | typequal => Unsized(base.into()),
                            const_sized | sized => {
                                let size_expr = ext.into_inner().last().unwrap();
                                let size_expr = Expr::parse(size_expr);
                                if let Some(size) = size_expr.const_arithmetic_expr() {
                                    Index(base.into(), size as u32)
                                } else {
                                    // VLA
                                    base.pointed()
                                }
                             },
                             params => {
                                 let param_decls = ext.into_inner();
                                 let mut param_vec = vec![];

                                 for param in param_decls {
                                     let mut pairs = param.into_inner();
                                     // Type of parameter
                                     let base_ty = DType::parse(pairs.next().unwrap());
                                     // declarator
                                     let mut param = pairs.next().map(Self::parse).unwrap_or(Abstract);

                                     let ty = param.set_type(base_ty);
                                     let ident = param.get_name();
                                     let param_decl = (ty, ident);

                                     param_vec.push(param_decl);
                                 }

                                 Call(Box::new(base), param_vec)
                             }
                             r => unreachable!("{:?}", r),
                        }
                    }
                    base
                };
        }
    }
}

impl Declarator {
    pub fn get_name(&self) -> Option<Ident> {
        use Declarator::*;
        match self {
            Abstract => None,
            Var(v) => Some(v.clone()),
            Deref(_, d) | Index(d, _) | Call(d, _) | Unsized(d) => d.get_name(),
        }
    }

    pub fn pointed(&self) -> Self {
        use Declarator::*;
        match self {
            Deref(n, d) => Deref(n + 1, d.clone().into()),
            _ => Deref(1, self.clone().into()),
        }
    }

    pub fn set_name(&mut self, ident: Ident) {
        use Declarator::*;
        match self {
            Abstract | Var(_) => *self = Var(ident),
            Deref(_, d) | Index(d, _) | Call(d, _) | Unsized(d) => d.set_name(ident),
        }
    }

    pub fn set_type(&self, decl_type: DType) -> DType {
        use DType::{Array, Function};
        use Declarator::*;
        match self {
            Abstract | Var(_) => decl_type,
            Deref(_, d) | Unsized(d) => d.set_type(decl_type.pointer()),
            Call(d, ps) => {
                let params = ps.iter().map(|(t, _)| t).cloned().collect();
                let func = Function(params, decl_type.into());
                d.set_type(func)
            }
            Index(d, s) => d.set_type(Array(*s, decl_type.into())),
        }
    }
}
