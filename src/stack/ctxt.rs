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
    pub funcs: HashMap<Ident, Label>,
    pub globals: HashMap<Ident, Word>,
    locals: HashMap<Ident, Word>,
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

    pub fn fdecl(&mut self, f: Ident) -> Label {
        let label = self.label();
        self.funcs.insert(f, label);
        label
    }

    pub fn global_decl(&mut self, v: &Ident, _ty: &DType) {
        self.globals.insert(v.clone(), self.global_offset as Word);
        self.global_offset += 1;
    }

    pub fn local_decl(&mut self, v: &Ident, _ty: &DType) {
        self.locals.insert(v.clone(), self.local_offset as Word);
        self.local_offset += 1;
    }

    pub fn fn_label(&mut self, v: &Ident) -> Label {
        self.funcs[v]
    }

    pub fn call_fn(&mut self, v: &Expr, args: &Vec<Expr>) {
        let height = self.stack_height.expect("Height should be known.");
        let ret_label = self.label();

        use StackInst::*;
        // Push return address & stack pointer
        self.emit_stream(&[
            Push(ret_label),
            LclRead(height),
            Push(self.local_offset as u16 + height as u16 + 2 - 1),
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
        if let Some(&addr) = self.globals.get(v) {
            self.stream.push(Push(addr));
            return;
        }

        unreachable!();
    }

    pub fn push_var(&mut self, v: &Ident) {
        use StackInst::*;

        if let Some(&addr) = self.globals.get(v) {
            let height = self.stack_height.unwrap();
            // If in global scope
            if self.ret_lbl == 0 {
                self.emit_stream(&[
                    Debug("Global access from global scope"),
                    LclRead(height - addr as usize - 1),
                ]);
            } else {
                self.emit_stream(&[
                    LclRead(height - 1),
                    Push(height as Word + 1),
                    Add,
                    Push(addr),
                    Sub,
                    StkRead,
                ]);
            }
            return;
        }

        if let Some(&addr) = self.funcs.get(v) {
            self.emit(Push(addr));
            return;
        }

        if let Some(&addr) = self.locals.get(v) {
            let height = self.stack_height.unwrap();
            let offset = height - 1 - addr as usize;
            self.emit_stream(&[LclRead(offset)]);
            return;
        }

        unreachable!();
    }

    pub fn store(&mut self, v: &Ident) {
        use StackInst::*;

        if let Some(&addr) = self.locals.get(v) {
            let height = self.stack_height.unwrap();
            let offset = height - 1 - addr as usize;
            self.emit(LclStr(offset));
            return;
        }

        if let Some(&addr) = self.globals.get(v) {
            let height = self.stack_height.unwrap();
            self.emit_stream(&[
                LclRead(height - 1),
                Push(height as Word),
                Add,
                Push(addr),
                Sub,
                StkStr,
            ]);
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
