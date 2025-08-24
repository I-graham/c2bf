use super::*;

use std::collections::*;

type FDef = (DType, Vec<ParamDecl>, Box<Stmt>);
type VDef = (bool, DType, Option<Expr>);

pub struct Program {
    pub funs: HashMap<Ident, FDef>,
    pub vars: HashMap<Ident, VDef>,
    pub order: Vec<Ident>, // Order of variables. Probably could be cleaner.
}

impl ASTNode for Program {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut funs = HashMap::default();
        let mut vars = HashMap::default();
        let mut order = vec![];

        // Remove SOI & EOI
        use Rule::EOI;
        let decls = pair.into_inner().filter(|r| r.as_rule() != EOI);

        for decl in decls.map(Defn::parse) {
            use Defn::*;
            match decl {
                FDef(f, ty, ps, d) => {
                    funs.insert(f, (ty, ps, d));
                }
                Vars(s, ty, vs) => {
                    for (vd, def) in vs {
                        let n = vd.get_name().expect("Unnamed variable");
                        let ty = vd.set_type(ty.clone());

                        if def.is_some() {
                            order.push(n.clone());
                        }

                        // Avoid overwriting definition with declaration
                        if !vars.contains_key(&n) || def.is_some() {
                            vars.insert(n, (s, ty, def));
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        Self { funs, vars, order }
    }

    fn compile(&self, ctxt: &mut CompileContext, stream: &mut StackProgram) {
        use StackInst::*;

        // Declarations
        for f in self.funs.keys() {
            ctxt.fdecl(f.clone());
        }

        for v in &self.order {
            let (_, ty, _) = &self.vars[v];
            ctxt.global_decl(v, ty);
        }

        stream.extend(&[
            PushW(ctxt.global_offset as Word),
            StackAlloc, // Allocate space for globals
        ]);

        for v in &self.order {
            let (_, _, e) = &self.vars[v];
            let def = e.clone().unwrap();

            stream.push(Comment(v.clone().leak()));
            def.compile(ctxt, stream);
            ctxt.push_addr(v, stream);
            stream.push(GlobalStore);
        }

        // Call main()
        ctxt.call_fn(&Expr::Var("main".into()), &vec![], stream);
        stream.push(Exit);

        // Definitions
        for (f, (_, ps, b)) in &self.funs {
            ctxt.fdef(f, ps, b, stream);
        }
    }
}
