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
    pub funcs: HashMap<Ident, (Label, DType)>,
    pub globals: HashMap<Ident, (Word, DType)>,
    locals: HashMap<Ident, (Word, DType)>,
    label_count: Label,
}

impl CompileContext {
    pub fn compile<T: ASTNode>(&mut self, n: &T) {
        n.compile(self);
    }

    pub fn label(&mut self) -> Label {
        self.label_count += 1;
        self.label_count
    }

    pub fn fdecl(&mut self, f: Ident, ty: DType) -> Label {
        let label = self.label();
        self.funcs.insert(f, (label, ty));
        label
    }

    pub fn global_decl(&mut self, v: &Ident, ty: &DType) {
        self.globals
            .insert(v.clone(), (self.global_offset as Word, ty.clone()));
        let size = ty.size() as usize;
        self.global_offset += size;
    }

    pub fn local_decl(&mut self, v: &Ident, ty: &DType) {
        self.locals
            .insert(v.clone(), (self.local_offset as Word, ty.clone()));
        let size = ty.size() as usize;
        self.local_offset += size;
    }

    pub fn fn_label(&mut self, v: &Ident) -> Label {
        self.funcs[v].0
    }

    pub fn call_fn(&mut self, v: &Expr, args: &Vec<Expr>) {
        let height = self.stack_height.expect("Height should be known.");
        let ret_label = self.label();

        use StackInst::*;
        // Push return address & stack pointer
        self.emit_stream(&[
            Push(ret_label),
            LclRead(height),
            Push(self.local_offset as u16 + height as u16 - 1),
            Add, // stack pointer = previous stack pointer + stack frame size
        ]);

        for arg in args {
            self.compile(arg);
        }

        self.compile(v);
        self.emit_stream(&[Goto, Label(ret_label)]);
        self.stack_height = Some(height + 1);
    }

    pub fn push_addr(&mut self, v: &Ident) {
        use StackInst::*;

        if let Some((addr, _)) = self.globals.get(v) {
            self.emit(Push(*addr));
            return;
        }

        if let Some((addr, _)) = self.locals.get(v) {
            let height = self.stack_height.unwrap();
            self.emit_stream(&[LclRead(height - 1), Push(*addr), Add]);
            return;
        }

        unreachable!("{}", v);
    }

    pub fn push_var(&mut self, v: &Ident) {
        use StackInst::*;

        if let Some((addr, _)) = self.globals.get(v) {
            let height = self.stack_height.unwrap();
            // If in global scope
            if self.ret_lbl == 0 {
                self.emit(LclRead(height - *addr as usize - 1));
            } else {
                self.emit_stream(&[
                    LclRead(height - 1),
                    Push(height as Word),
                    Add,
                    Push(*addr),
                    Sub,
                    StkRead,
                ]);
            }
            return;
        }

        if let Some((addr, _)) = self.funcs.get(v) {
            self.emit(Push(*addr));
            return;
        }

        if let Some((addr, _)) = self.locals.get(v) {
            let height = self.stack_height.unwrap();
            let offset = height - 1 - *addr as usize;
            self.emit_stream(&[LclRead(offset)]);
            return;
        }

        unreachable!("{}", v);
    }

    pub fn store(&mut self, v: &Ident) {
        use StackInst::*;

        if let Some((addr, _)) = self.locals.get(v) {
            let height = self.stack_height.unwrap();
            let offset = height - 1 - *addr as usize;
            self.emit(LclStr(offset));
            return;
        }

        if let Some((addr, _)) = self.globals.get(v) {
            let height = self.stack_height.unwrap();
            self.emit_stream(&[
                LclRead(height - 1),
                Push(height as Word - 1),
                Add,
                Push(*addr),
                Sub,
                StkStr,
            ]);
            return;
        }

        unreachable!();
    }

    pub fn vty(&self, v: &Ident) -> &DType {
        if let Some((_, t)) = self.locals.get(v) {
            return t;
        }

        if let Some((_, t)) = self.globals.get(v) {
            return t;
        }

        if let Some((_, t)) = self.funcs.get(v) {
            return t;
        }

        unreachable!("{}", v)
    }

    pub fn fdef(&mut self, f: &Ident, params: &Vec<ParamDecl>, body: &Stmt) {
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

        self.stack_height = None; // Ignore stack height from this point on.
                                  // Return to caller, which should have pushed a return label
        self.emit_stream(&[
            Push(0),
            Push(self.ret_lbl),
            Goto,
            Label(self.ret_lbl),
            Move(frame_size),
            Dealloc(frame_size),
            Swap,
            Goto,
        ]);
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
