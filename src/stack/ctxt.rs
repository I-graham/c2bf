use super::*;

use std::collections::*;

pub type Label = Word;

#[derive(Default)]
pub struct CompileContext {
    pub global_offset: usize,
    local_offset: usize,
    label_count: Label,
    globals: HashMap<Ident, Word>,
    locals: HashMap<Ident, Word>,
    funcs: HashMap<Ident, Label>,
}

impl CompileContext {
    pub fn label(&mut self) -> Label {
        self.label_count += 1;
        self.label_count
    }

    pub fn fdecl(&mut self, f: Ident) -> Label {
        let label = self.label();
        self.funcs.insert(f, label);
        label
    }

    pub fn global_decl(&mut self, v: &Ident, ty: &DType) {
        let size = ty.size() as usize;
        let bytes = size.max(WORD_SIZE); // always allocate at least one word;
        let words = (bytes / WORD_SIZE) + if bytes % WORD_SIZE != 0 { 1 } else { 0 }; // How many words to allocate

        self.globals.insert(v.clone(), self.global_offset as Word);
        self.global_offset += words;
    }

    pub fn local_decl(&mut self, v: &Ident, ty: &DType) {
        let size = ty.size() as usize;
        let bytes = size.max(WORD_SIZE); // always allocate at least one word;
        let words = (bytes / WORD_SIZE) + if bytes % WORD_SIZE != 0 { 1 } else { 0 }; // How many words to allocate

        self.locals.insert(v.clone(), self.local_offset as Word);
        self.local_offset += words;
    }

    pub fn fn_label(&mut self, v: &Ident) -> Label {
        self.funcs[v]
    }

    pub fn call_fn(&mut self, v: &Ident, args: &Vec<Expr>, stream: &mut StackProgram) {
        for arg in args {
            arg.compile(self, stream);
        }
        let ret_label = self.label();

        // TODO: Incorporate stack pointer to allow indirection in BF (esp. for globals)
        use StackInst::*;
        stream.extend(&[
            PushW(ret_label),
            PushW(self.fn_label(v)),
            Goto,
            Label(ret_label),
        ]);
    }

    pub fn push_addr(&self, v: &Ident, stream: &mut StackProgram) {
        use StackInst::*;
        if let Some(&addr) = self.globals.get(v) {
            stream.push(PushW(addr));
            return;
        }

        unreachable!();
    }

    pub fn push_var(&self, v: &Ident, stream: &mut StackProgram) {
        use StackInst::*;

        if let Some(&addr) = self.globals.get(v) {
            stream.extend(&[PushW(addr), GlobalRead]);
            return;
        }

        unreachable!();
    }

    pub fn store(&self, v: &Ident, stream: &mut StackProgram) {
        use StackInst::*;

        if let Some(&addr) = self.locals.get(v) {
            stream.extend(&[PushW(addr), LocalStore]);
            return;
        }

        if let Some(&addr) = self.globals.get(v) {
            stream.extend(&[PushW(addr), GlobalStore]);
            return;
        }
    }

    pub fn fdef(
        &mut self,
        f: &Ident,
        params: &Vec<ParamDecl>,
        body: &Stmt,
        stream: &mut StackProgram,
    ) {
        // New Stack Frame
        self.locals.clear();
        self.local_offset = 0;

        // Param Declarations
        use StackInst::*;
        for (pty, pname) in params {
            let Some(pname) = pname else { continue };

            self.local_decl(pname, pty);
        }

        // Allocate space for all variables
        for (vty, v) in body.vars() {
            let Some(v) = v else { continue };

            self.local_decl(&v, &vty);
        }

        let label = self.fn_label(f);
        let frame_size = self.local_offset as Word;

        stream.extend(&[
            Comment(f.clone().leak()),
            Label(label),
            PushW(frame_size),
            StackAlloc,
        ]);

        body.compile(self, stream);

        // Return to caller, which should have pushed a return label
        stream.extend(&[PushW(frame_size), StackDealloc, Goto]);
    }
}
