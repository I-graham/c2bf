use std::collections::HashMap;

use super::*;

pub fn exec_stack_program(code: &[StackInst]) {
    let mut stack_machine = StackMachine::default();

    stack_machine.exec(code);
}

#[derive(Default)]
pub struct StackMachine {
    pub stack: Vec<Word>,
    pub ip: usize,
}

impl StackMachine {
    pub fn exec(&mut self, code: &[StackInst]) {
        use StackInst::*;
        let labels: HashMap<Word, usize> = code
            .iter()
            .enumerate()
            .filter_map(|(i, op)| match op {
                Label(l) => Some((*l, i)),
                _ => None,
            })
            .collect();

        let mut ip = 0;
        loop {
            let inst = code[ip];

            let (args, _) = inst.signature();
            if self.stack.len() < args {
                panic!("#{}: {:?} got insufficient args.", ip, inst);
            }

            match inst {
                Exit | Label(0) => {
                    break;
                }
                Debug(l) => {
                    println!("Stack @ {}: {:?}", l, self.stack);
                }
                Nop | Label(_) | Comment(_) => (),
                Push(b) => self.stack.push(b),
                PrintChar => {
                    println!("{}", self.stack.pop().unwrap());
                }

                Alloc(n) => {
                    let len = self.stack.len();
                    self.stack.resize(len + n, 0);
                }
                Dealloc(n) => {
                    let len = self.stack.len();
                    self.stack.resize(len - n, 0);
                }
                Move(d) => {
                    let n = self.stack.len() - 1;
                    let word = *self.stack.last().unwrap();
                    self.stack[n - d] = word;
                }
                Swap => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(a);
                    self.stack.push(b);
                }
                Copy => {
                    let top = *self.stack.last().unwrap();
                    self.stack.push(top);
                }

                GblRead => {
                    let addr = self.stack.pop().unwrap();
                    let value = self
                        .stack
                        .get(addr as usize)
                        .expect("Global address does not exist.");
                    self.stack.push(*value)
                }
                GblStr => {
                    let addr = self.stack.pop().unwrap();
                    let word = self.stack.pop().unwrap();

                    *self
                        .stack
                        .get_mut(addr as usize)
                        .expect("Global address does not exist.") = word;
                }
                LclRead(addr) => {
                    let addr = self.stack.len() - 1 - addr;
                    let value = *self.stack.get(addr).expect("Address does not exist");

                    self.stack.push(value);
                }
                LclStr(addr) => {
                    let word = self.stack.pop().unwrap();
                    let addr = self.stack.len() - addr;

                    *self.stack.get_mut(addr).expect("Address does not exist") = word;
                }

                Branch(t, f) => {
                    let word = self.stack.pop().unwrap();
                    let lbl = if word != 0 { t } else { f };
                    ip = labels[&lbl];
                }
                Goto => {
                    let addr = self.stack.pop().unwrap();
                    ip = labels[&addr];
                    continue;
                }

                o @ (Add | Sub | Mul | Div | LShift | RShift | And | Or | Xor) => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let out = match o {
                        Add => a + b,
                        Sub => a - b,
                        Mul => a * b,
                        Div => a / b,
                        LShift => a << b,
                        RShift => a >> b,
                        And => a & b,
                        Or => a | b,
                        Xor => a ^ b,
                        _ => unreachable!(),
                    };
                    self.stack.push(out)
                }
                LNot => {
                    let word = self.stack.pop().unwrap();
                    let not = if word == 0 { 1 } else { 0 };
                    self.stack.push(not);
                }
                Not => {
                    let word = self.stack.pop().unwrap();
                    self.stack.push(!word);
                }

                o @ (Eq | Neq | Lt | LtEq | Gr | GrEq | LAnd | LOr) => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let cmp = match o {
                        Eq => a == b,
                        Neq => a != b,
                        Lt => a < b,
                        LtEq => a <= b,
                        Gr => a > b,
                        GrEq => a >= b,
                        LAnd => a != 0 && b != 0,
                        LOr => a != 0 || b != 0,
                        _ => unreachable!(),
                    };
                    let word = if cmp { 1 } else { 0 };
                    self.stack.push(word);
                }
            }

            ip += 1;
        }
    }
}
