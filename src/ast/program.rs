use super::*;

use std::collections::*;

type FDef = (DType, Vec<ParamDecl>, Box<Stmt>);

pub struct Program {
    pub funcs: HashMap<Ident, FDef>,
}

impl ASTNode for Program {
    fn parse(pair: Pair<Rule>) -> Self {
        parser_rule! {
            pair:

            translation_unit
                [..fs,] -> {
                    let parse_func = |f| {
                        let Defn::FDef(id, ty, ps, d) = Defn::parse(f) else {
                            unreachable!()
                        };
                        (id, (ty, ps, d))
                    };
                    let funcs = fs.map(parse_func).collect();

                    Program {
                        funcs
                    }
                };
        }
    }

    fn compile(&self, _context: &CompileContext, _stream: &mut Vec<StackInst>) {}
}
