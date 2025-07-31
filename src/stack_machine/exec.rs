use super::*;

pub fn exec_stack_program(code: &StackProgram) {
    let mut stack_machine = StackMachine::default();

    stack_machine.exec(code);
}

#[derive(Default)]
pub struct StackMachine {
    pub stack: Vec<u8>,
    pub ip: usize,
}

impl StackMachine {
    pub fn exec(&mut self, code: &StackProgram) {
        for inst in code {
            use StackInst::*;
            match inst {
                Nop => (),
                Byte(b) => self.stack.push(*b),
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
                Print => println!("{}", self.stack.last().unwrap()),
            }
        }
    }
}
