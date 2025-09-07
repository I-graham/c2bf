use super::*;

use std::collections::*;

pub type Label = Word;

#[derive(Default)]
pub struct CompileContext {
    pub global_offset: usize,
    pub local_offset: usize,
    pub stack_height: Option<usize>,
    pub stream: Vec<StackInst>,
    pub ret_lbl: Label,
    pub loop_exit: (Label, Label), // continue & break labels, respectively
    label_count: Label,
    globals: HashMap<Ident, Word>,
    locals: HashMap<Ident, Word>,
    funcs: HashMap<Ident, Label>,
}

impl CompileContext {
    pub fn compile<T: ASTNode>(&mut self, n: &T) {
        n.compile(self);
    }

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

    pub fn call_fn(&mut self, v: &Expr, args: &Vec<Expr>) {
        let height = self.stack_height.expect("Height should be known.");
        let ret_label = self.label();

        use StackInst::*;
        // Push stack pointer & return address
        self.emit_stream(&[PushB(ret_label), LclReadB(self.local_offset)]);

        for arg in args {
            self.compile(arg);
        }

        self.compile(v);
        self.emit_stream(&[Goto, Label(ret_label)]);
        self.stack_height = Some(height + 1);
    }

    pub fn push_addr(&mut self, v: &Ident) {
        use StackInst::*;
        if let Some(&addr) = self.globals.get(v) {
            self.stream.push(PushB(addr));
            return;
        }

        unreachable!();
    }

    pub fn push_var(&mut self, v: &Ident) {
        use StackInst::*;

        if let Some(&addr) = self.globals.get(v) {
            self.emit_stream(&[PushB(addr), GblReadB]);
            return;
        }

        if let Some(&addr) = self.funcs.get(v) {
            self.emit(PushB(addr));
            return;
        }

        if let Some(&addr) = self.locals.get(v) {
            let Some(height) = self.stack_height else {
                unreachable!()
            };
            let offset = height - 1 - addr as usize;
            self.emit_stream(&[LclReadB(offset)]);
            return;
        }

        unreachable!();
    }

    pub fn store(&mut self, v: &Ident) {
        use StackInst::*;

        if let Some(&addr) = self.locals.get(v) {
            let Some(height) = self.stack_height else {
                unreachable!()
            };
            let offset = height - 1 - addr as usize;
            self.emit(LclStrB(offset));
            return;
        }

        if let Some(&addr) = self.globals.get(v) {
            self.emit_stream(&[PushB(addr), GblStrB]);
            return;
        }

        unreachable!();
    }

    pub fn fdef(&mut self, f: &Ident, r: &DType, params: &Vec<ParamDecl>, body: &Stmt) {
        // New Stack Frame
        self.ret_lbl = self.label();

        self.locals.clear();
        self.local_offset = 1; // Include stack pointer in stack frame

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
        let frame_size = self.local_offset;

        self.emit_stream(&[
            Comment(f.clone().leak()),
            Label(label),
            Alloc(frame_size - params.len() - 1), // Stack pointer is already allocated
        ]);

        self.stack_height = Some(frame_size);

        self.compile(body);

        if r == &DType::Void {
            self.emit_stream(&[Dealloc(frame_size), Goto]);
        } else {
            self.stack_height = None; // Ignore stack height from this point on.
                                      // Return to caller, which should have pushed a return label
            self.emit_stream(&[
                PushB(0),
                PushB(self.ret_lbl),
                Goto,
                Label(self.ret_lbl),
                Move(frame_size),
                Dealloc(frame_size),
                SwapB,
                Goto,
            ]);
        }
    }

    pub fn emit(&mut self, inst: StackInst) {
        if let (Some(height), (args, Some(output))) = (self.stack_height, inst.signature()) {
            self.stack_height = Some(height - args + output);
        }
        self.stream.push(inst);
    }

    pub fn emit_stream(&mut self, code: &[StackInst]) {
        for &inst in code {
            self.emit(inst);
        }
    }
}
