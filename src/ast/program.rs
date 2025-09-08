use super::*;

use std::collections::*;

pub type FDef = (DType, Vec<ParamDecl>, Box<Stmt>);
pub type VDef = (bool, DType, Option<Expr>);

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

    fn compile(&self, ctxt: &mut CompileContext) {
        use StackInst::*;
        let init_lbl = ctxt.label(); // Always equal to 1

        // Declarations
        for (f, (ret, args, _)) in &self.funs {
            let ty = DType::Function(
                args.iter().map(|(t, _)| t.clone()).collect(),
                ret.clone().into(),
            );
            ctxt.fdecl(f.clone(), ty);
        }

        for v in &self.order {
            let (_, ty, _) = &self.vars[v];
            ctxt.global_decl(v, ty);
        }

        for (v, (_, ty, _)) in &self.vars {
            if !self.order.contains(v) {
                ctxt.global_decl(v, ty);
            }
        }

        ctxt.emit(Label(init_lbl));

        // Allocate space for globals
        ctxt.emit(Alloc(ctxt.global_offset));

        ctxt.stack_height = Some(ctxt.global_offset);
        for v in &self.order {
            let (_, _, e) = &self.vars[v];
            let def = e.clone().unwrap();

            ctxt.emit(Comment(v.clone().leak()));
            ctxt.compile(&def);
            let addr = ctxt.globals[v].0 as usize;
            ctxt.emit(LclStr(self.order.len() - addr));
        }

        // Call main()
        ctxt.stack_height = Some(0);
        let ret_lbl = ctxt.label();
        let main_lbl = ctxt.funcs.get("main").unwrap().0;
        ctxt.emit_stream(&[
            Debug("Start main call"),
            Push(ret_lbl),
            Push(ctxt.global_offset as _),
            Push(main_lbl as _),
            Goto,
            Label(ret_lbl),
        ]);
        ctxt.emit(Exit);

        // Definitions
        for (f, (r, ps, b)) in &self.funs {
            ctxt.fdef(f, r, ps, b);
        }

        // Label 0 is always Exit
        ctxt.emit(Label(0));
    }
}
