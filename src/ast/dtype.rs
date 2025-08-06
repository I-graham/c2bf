use super::*;

pub enum DType {
    Void,
    U8,
    S8,
    U16,
    S16,
    U32,
    S32,
    U64,
    S64,
    Float,
    Double,
    Pointer(usize, Box<Self>), // Level of indirection + base type
    Array(u32, Box<DType>),
    Function(Vec<DType>, Box<DType>),
}

impl ASTNode for DType {
    fn parse(pair: Pair<Rule>) -> Self {
        use DType::*;
        parser_rule! {
            pair :

            type_name
                [sql] -> sql;
                [sql, decl:DType] -> {
                    decl.change_base(sql);
                    decl
                };

            specifier_qualifier_list
                [.. sql,] -> {
                    let specs = sql.filter(|s| s.as_rule() == type_specifier);

                    let mut long = false;
                    let mut signed = true;
                    let mut ty = S32;

                    for spec in specs {
                        match spec.as_str() {
                            "unsigned" => signed = false,
                            "signed" => signed = true,
                            "char" => ty = S8,
                            "short" => ty = S16,
                            "int" => ty = S32,
                            "long" => {
                                long = true;
                                ty = S32;
                            }
                            "long" if long => ty = S64,
                            "float" => ty = Float,
                            "double" => ty = Double,
                            _ => unreachable!(),
                        }
                    }

                    if !signed {
                        ty = ty.make_unsigned();
                    }

                    ty
                };

            abstract_declarator
            | declarator
                [decl] -> decl;
                [p, d] -> {
                    let Pointer(n, _) = p else {
                        unreachable!()
                    };

                    Pointer(n, Box::new(d))
                };

            pointer
                [.. ps,] -> {
                    let level_of_indirection = ps.filter(|token|
                        token.as_str() == "*"
                    ).count();

                    Pointer(level_of_indirection, Void.into())
                };

            direct_abstract_declarator
            | direct_declarator
                [.. exts,] -> {
                    let base_pair = exts.next().unwrap();
                    let mut base = match base_pair.as_rule() {
                        dd_base => {
                               let inner = base_pair.into_inner().next().unwrap();
                               match inner.as_rule() {
                                   IDENTIFIER => Void,
                                   declarator => DType::parse(inner),
                                   _ => unreachable!(),
                               }
                        }
                        da_base => DType::parse(base_pair),
                        _ => unreachable!(),
                    };

                    for ext in exts {
                        base = match ext.as_rule() {
                            brackets => base.pointer(),
                            sized => {
                                let const_expr = ext.into_inner().next().unwrap();
                                let const_expr = Expr::parse(const_expr);
                                let size = const_expr.evaluate_word();

                                Array(size, Box::new(base))
                             },
                             params => {
                                 let param_types = ext.into_inner().map(Self::parse).collect();

                                 Function(param_types, Box::new(base))
                             }
                             _ => unreachable!(),
                        }
                    }
                    base
                };
        }
    }
}

impl DType {
    pub fn pointer(self) -> Self {
        use DType::*;
        match self {
            Pointer(n, b) => Pointer(n + 1, b),
            b => Pointer(1, b.into()),
        }
    }

    fn make_unsigned(&self) -> Self {
        use DType::*;
        match self {
            S8 => U8,
            S16 => U16,
            S32 => U32,
            S64 => U64,
            _ => unreachable!(),
        }
    }

    fn change_base(&mut self, base: Self) {
        use DType::*;
        match self {
            Array(_, b) => b.change_base(base),
            Pointer(_, b) => b.change_base(base),
            Function(_, r) => r.change_base(base),
            _ => *self = base,
        }
    }
}
