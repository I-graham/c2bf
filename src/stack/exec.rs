use std::collections::HashMap;

use super::*;

pub fn exec_stack_program(code: &StackProgram) {
    let mut stack_machine = StackMachine::default();

    stack_machine.exec(code);
}

#[derive(Default)]
pub struct StackMachine {
    pub stack: Vec<u32>,
    pub ip: usize,
}

impl StackMachine {
    pub fn exec(&mut self, code: &StackProgram) {
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
            match inst {
                Debug(l) => {
                    println!("Stack @ {}: {:?}", l, self.stack);
                }
                Nop | Label(_) | Comment(_) => (),
                PushW(b) => self.stack.push(b),
                DiscardW => {
                    self.stack.pop();
                }
                PrintI32 => {
                    println!("{}", self.stack.pop().unwrap());
                }

                StackAlloc => {
                    let size = self.stack.pop().unwrap() as usize;
                    let len = self.stack.len();
                    self.stack.resize(len + size, 0);
                }
                StackDealloc => {
                    let size = self.stack.pop().unwrap() as usize;
                    let len = self.stack.len();
                    self.stack.resize(len - size, 0);
                }
                CopyDown(d) => {
                    let n = self.stack.len() - 1;
                    let word = *self.stack.last().unwrap();
                    self.stack[n - d] = word;
                }
                SwapW => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(a);
                    self.stack.push(b);
                }

                GlobalRead => {
                    let addr = self.stack.pop().expect("Global read on empty stack");
                    let value = self
                        .stack
                        .get(addr as usize)
                        .expect("Global address does not exist.");
                    self.stack.push(*value)
                }
                GlobalStore => {
                    let addr = self.stack.pop().expect("Global store on empty stack");
                    let word = self.stack.pop().expect("Global store on empty stack");

                    *self
                        .stack
                        .get_mut(addr as usize)
                        .expect("Global address does not exist.") = word;
                }
                LocalRead(addr) => {
                    let addr = self.stack.len() - 1 - addr;
                    let value = *self.stack.get(addr).expect("Address does not exist");

                    self.stack.push(value);
                }
                LocalStore(addr) => {
                    let word = self.stack.pop().expect("Local store on empty stack");
                    let addr = self.stack.len() - addr;

                    *self.stack.get_mut(addr).expect("Address does not exist") = word;
                }

                Goto => {
                    let addr = self.stack.pop().expect("Goto nil");
                    ip = labels[&addr];
                    continue;
                }
                Exit => {
                    break;
                }

                o @ (Add | Sub | Mul | Div) => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    let out = match o {
                        Add => a + b,
                        Sub => a - b,
                        Mul => a + b,
                        Div => a / b,
                        _ => unreachable!(),
                    };
                    self.stack.push(out)
                }
            }

            ip += 1;
        }
    }
}
