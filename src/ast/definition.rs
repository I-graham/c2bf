use super::*;

pub enum Definition {
    Struct(DType, Option<Ident>),
    Union(DType, Option<Ident>),
    Enum(DType, Option<Ident>),
    TypeDef(DType, Vec<Declarator>),
    Vars(bool, DType, Vec<(Declarator, Option<Expr>)>), // bool indicates whether decl is static
    FDef(DType, Vec<ParamDecl>, Box<Stmt>),
}

impl ASTNode for Definition {
    fn parse(pair: Pair<Rule>) -> Self {
        use Declarator::*;
        use Definition::*;
        parser_rule! {
            pair:

            function_definition
                [i:(), ty, d, body] -> {
                    let Call(n, args) = d else {
                        unreachable!()
                    };
                    let Var(name) = *n else {
                        unreachable!()
                    };

                    FDef(ty, args, Box::new(body))
                };

            declaration
                [t] -> t;
                [s:(), d:Self] -> {
                    d.make_static();
                    d
                };

            typedef
                [ty, .. ds] -> {
                    let ds = ds.map(Declarator::parse).collect();
                    TypeDef(ty, ds)
                };

            vdecl
                [i:(), ty, ds:Self] -> {
                    ds.change_base_ty(ty);
                    ds
                };

            init_declarator_list
                [l] -> l;
                [i, l:Self] -> {
                    l.add_def(i);
                    l
                };

            init_declarator
                [d:Declarator] -> {
                    Vars(false, DType::Void, vec![(d, None)])
                };
                [d:Declarator, i] -> {
                    Vars(false, DType::Void, vec![(d, Some(i))])
                };
        }
    }
}

impl Definition {
    fn make_static(&mut self) {
        use Definition::*;
        match self {
            Vars(s, _, _) => *s = true,
            FDef(_, _, _) => (),
            _ => unreachable!(),
        }
    }

    fn change_base_ty(&mut self, ty: DType) {
        use Definition::*;
        match self {
            Vars(_, d, _) => *d = ty,
            FDef(t, _, _) => *t = ty,
            _ => unreachable!(),
        }
    }

    fn add_def(&mut self, o: Self) {
        use Definition::*;
        match (self, o) {
            (Vars(_, _, ds), Vars(_, _, mut d)) if d.len() == 1 => {
                let d = d.remove(0);
                ds.insert(0, d);
            }
            _ => unreachable!(),
        }
    }
}
