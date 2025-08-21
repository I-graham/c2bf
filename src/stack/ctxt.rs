use super::*;

use std::collections::*;

pub enum Location {
    Global(Word),
    Local(Word),
    Function(Word),
}

pub type Label = Word;

#[derive(Default)]
pub struct CompileContext {
    pub global_offset: usize,
    label_count: Label,
    scope: HashMap<Ident, Location>,
}

impl CompileContext {
    pub fn label(&mut self) -> Label {
        self.label_count += 1;
        self.label_count
    }

    pub fn fdecl(&mut self, f: Ident) -> Label {
        let label = self.label();
        let loc = Location::Function(label);
        self.scope.insert(f, loc);
        label
    }

    pub fn global_vdecl(&mut self, v: Ident, ty: DType) {
        use Location::*;

        let size = ty.size() as usize;
        let bytes = size.max(WORD_SIZE); // always allocate at least one word;
        let words = (bytes / WORD_SIZE) + if bytes % WORD_SIZE != 0 { 1 } else { 0 }; // How many words to allocate

        self.scope.insert(v, Global(self.global_offset as Word));
        self.global_offset += words;
    }

    pub fn lookup_fn(&mut self, v: &Ident) -> Label {
        let Location::Function(label) = self.scope[v] else {
            unreachable!()
        };

        label
    }

    pub fn call_fn(&mut self, v: &Ident, args: &Vec<Expr>, stream: &mut StackProgram) {
        for arg in args {
            arg.compile(self, stream);
        }
        let ret_label = self.label();

        use StackInst::*;
        stream.extend(&[
            PushW(ret_label),
            PushW(self.lookup_fn(v)),
            Goto,
            Label(ret_label),
        ]);
    }

    pub fn push_addr(&self, v: &Ident, stream: &mut StackProgram) {
        use Location::*;
        use StackInst::*;
        let code = match self.scope[v] {
            Global(w) => &[PushW(w)],
            Local(_) => todo!(),
            Function(_) => todo!(),
        };

        stream.extend(code);
    }

    pub fn push_var(&self, v: &Ident, stream: &mut StackProgram) {
        use Location::*;
        use StackInst::*;

        match self.scope[v] {
            Global(_) => {
                self.push_addr(v, stream);
                stream.push(GlobalRead);
            }
            _ => todo!(),
        }
    }

    pub fn fdef(
        &mut self,
        f: &Ident,
        params: &Vec<ParamDecl>,
        body: &Stmt,
        stream: &mut StackProgram,
    ) {
        use StackInst::*;
        for _ in params {
            todo!(); // Read in parameters, somehow
        }

        let label = self.lookup_fn(f);

        stream.push(Comment(f.clone().leak()));
        stream.push(Label(label));

        body.compile(self, stream);

        // Return to caller, which should have pushed a return label
        stream.push(Goto);
    }
}
