use std::collections::HashMap;
use std::io::Read;

use super::*;

pub fn exec_bf(bf: &[BF]) {
    let fast = bf.iter().cloned().map(FastBF::from).collect::<Vec<_>>();

    let map = parse_bracs(&fast);
    let mut ip = 0;
    let mut stack = vec![0 as Word];
    let mut head = 0;
    let mut stdin = std::io::stdin();

    while ip < bf.len() {
        use BF::*;
        match bf[ip] {
            Profile(_) => (),
            Dbg(_msg) => {
                #[cfg(feature = "debugbf")]
                {
                    dbg!(head);
                    dbg!(_msg);
                }
            }
            Left => head -= 1,
            Right => {
                head += 1;
                if stack.len() == head {
                    stack.push(0);
                }
            }
            Inc => stack[head] = stack[head].wrapping_add(1),
            Dec => stack[head] = stack[head].wrapping_sub(1),
            Input => {
                let mut buf = [0];
                stdin.read_exact(&mut buf).expect("No input");
                stack[head] = buf[0] as Word;
            }
            Output => print!("{}", stack[head] as u8 as char),
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

pub fn exec_fastbf(fast: &[FastBF]) -> HashMap<StackInst, usize> {
    let map = parse_bracs(fast);
    let mut ip = 0;
    let mut stack = vec![0 as Word];
    let mut head = 0;
    let mut stdin = std::io::stdin();
    let mut inst = StackInst::Nop;
    let mut profile = HashMap::new();

    while ip < fast.len() {
        *profile.entry(inst).or_insert(0) += 1;

        use FastBF::*;
        match &fast[ip] {
            Inst(i) => inst = *i,
            Move(n) => {
                head = (head as isize).wrapping_add(*n) as usize;
                if head >= stack.len() {
                    stack.resize(head + 1, 0);
                }
            }
            Const(n) => {
                stack[head] = (stack[head] as isize).wrapping_add(*n) as _;
            }
            AddCell(vs) => {
                let val = stack[head] as isize;
                for (d, m) in vs {
                    let p = (head as isize).wrapping_add(*d) as usize;

                    if p >= stack.len() {
                        stack.resize(p + 1, 0);
                    }

                    stack[p] = (stack[p] as isize).wrapping_add(m * val) as _;
                }
                stack[head] = 0;
            }
            BinAnd => {
                stack[head - 1] &= stack[head];
                stack[head] = 0;
                head -= 1;
            }
            BinOr => {
                stack[head - 1] |= stack[head];
                stack[head] = 0;
                head -= 1;
            }
            BinXor => {
                stack[head - 1] ^= stack[head];
                stack[head] = 0;
                head -= 1;
            }
            Rem => {
                stack[head - 1] %= stack[head];
                stack[head] = 0;
                head -= 1;
            }
            Mult => {
                stack[head - 1] *= stack[head];
                stack[head] = 0;
                head -= 1;
            }
            Geq => {
                let word = if stack[head - 1] >= stack[head] { 1 } else { 0 };

                stack[head] = 0;
                head -= 1;
                stack[head] = word;
            }
            ShiftR => {
                stack[head - 1] >>= stack[head];
                stack[head] = 0;
                head -= 1;
            }
            Read => {
                let addr = stack[head];
                stack[head] = stack[head - addr as usize];
            }
            Store => {
                let addr = stack[head];
                let word = stack[head - 1];
                let addr = head - addr as usize - 1;
                stack[head] = 0;
                head -= 1;
                stack[head] = 0;
                stack[addr] = word;
                head -= 1;
            }
            In => {
                let mut buf = [0];
                stdin.read_exact(&mut buf).expect("No input");
                stack[head] = buf[0] as Word;
            }
            Out => print!("{}", stack[head] as u8 as char),
            LB => {
                if stack[head] == 0 {
                    ip = map[&ip];
                }
            }
            RB => {
                if stack[head] != 0 {
                    ip = map[&ip];
                }
            }
        }
        ip += 1;
    }

    profile
}

fn parse_bracs(code: &[FastBF]) -> HashMap<usize, usize> {
    let mut map = HashMap::default();
    let mut bracs = vec![];

    for (i, inst) in code.iter().enumerate() {
        use FastBF::*;
        match inst {
            LB => bracs.push(i),
            RB => {
                let start = bracs.pop().unwrap();
                map.insert(i, start);
                map.insert(start, i);
            }
            _ => (),
        }
    }

    map
}
