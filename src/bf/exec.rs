use std::collections::HashMap;
use std::io::Read;

use super::*;

pub fn exec_bf(code: &[BFInst]) {
    let map = parse_bracs(code);
    let mut ip = 0;
    let mut stack = vec![0u8];
    let mut head = 0;
    let mut stdin = std::io::stdin();

    while ip < code.len() {
        use BFInst::*;
        match code[ip] {
            Dbg(_) => {}
            Left => head -= 1,
            Right => {
                head += 1;
                if stack.len() == head {
                    stack.push(0);
                }
            }
            Inc => stack[head] = stack[head].wrapping_add(1),
            Dec => stack[head] = stack[head].wrapping_sub(1),
            In => {
                let mut buf = [0];
                stdin.read_exact(&mut buf).expect("No input");
                stack[head] = buf[0];
            }
            Out => print!("{}", stack[head] as char),
            LBrac => {
                if stack[head] == 0 {
                    ip = map[&ip];
                }
            }
            RBrac => {
                if stack[head] != 0 {
                    ip = map[&ip];
                }
            }
        }
        ip += 1;
    }
}

fn parse_bracs(code: &[BFInst]) -> HashMap<usize, usize> {
    let mut map = HashMap::default();
    let mut bracs = vec![];

    for (i, inst) in code.iter().enumerate() {
        use BFInst::*;
        match inst {
            LBrac => bracs.push(i),
            RBrac => {
                let start = bracs.pop().unwrap();
                map.insert(i, start);
                map.insert(start, i);
            }
            _ => (),
        }
    }

    map
}
