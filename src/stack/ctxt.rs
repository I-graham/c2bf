use super::*;

use std::collections::*;

pub enum Location {
    Global(Word),
    Local(Word),
    Function(Word),
}

#[derive(Default)]
pub struct CompileContext {
    pub global_offset: usize,
    scope: HashMap<Ident, Location>,
}

impl CompileContext {
    pub fn fdecl(&mut self, f: Ident) {
        let loc = Location::Function(self.scope.len() as _);
        self.scope.insert(f, loc);
    }

    pub fn global_vdecl(&mut self, v: Ident, ty: DType) {
        use Location::*;

        let size = ty.size() as usize;
        let bytes = size.max(WORD_SIZE); // always allocate at least one word;
        let words = (bytes / WORD_SIZE) + if bytes % WORD_SIZE != 0 { 1 } else { 0 }; // How many words to allocate

        self.scope.insert(v, Global(self.global_offset as Word));
        self.global_offset += words;
    }

    pub fn push_addr(&self, v: &Ident, stream: &mut Vec<StackInst>) {
        use Location::*;
        use StackInst::*;
        let code = match self.scope[v] {
            Global(w) => &[PushW(w)],
            Local(_) => todo!(),
            Function(_) => todo!(),
        };

        stream.extend(code);
    }

    pub fn push_var(&self, v: &Ident, stream: &mut Vec<StackInst>) {
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
}
