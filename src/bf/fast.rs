use std::collections::HashMap;

use super::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FastBF {
    Move(isize),                  // </>
    Const(isize),                 // +/-
    LB,                           // [ Left Bracket
    RB,                           // ] Right Bracket
    In,                           // , input
    Out,                          // . output
    AddCell(Vec<(isize, isize)>), // Add this cell value to various locations, a certain number of times
}

pub fn optimize(bf: &[BF]) -> Vec<FastBF> {
    use FastBF::*;

    let mut fast = bf.iter().cloned().map(FastBF::from).collect::<Vec<_>>();

    let mut ip = 0;
    'outer: loop {
        match &fast[ip..] {
            [Const(a), Const(b), ..] => {
                fast[ip] = Const(a + b);
                fast.remove(ip + 1);
            }
            [Move(a), Move(b), ..] => {
                fast[ip] = Move(a + b);
                fast.remove(ip + 1);
            }
            [LB, ..] => {
                let mut drift = 0;
                let mut rip = ip + 1;
                let mut map = HashMap::new();
                loop {
                    match fast[rip] {
                        Move(n) => drift += n,
                        Const(n) => {
                            let entry = map.entry(drift).or_insert(0);
                            *entry += n;
                        }
                        RB if drift == 0 && map.get(&0) == Some(&-1) => {
                            fast.drain(ip + 1..=rip);
                            fast[ip] = AddCell(map.into_iter().collect());
                            ip += 1;
                            continue 'outer;
                        }
                        _ => {
                            ip += 1;
                            continue 'outer;
                        }
                    }
                    rip += 1;
                }
            }
            [] => break,
            _ => (),
        }
        ip += 1;
    }

    fast
}

impl From<BF> for FastBF {
    fn from(value: BF) -> Self {
        use FastBF::*;
        use BF::*;
        match value {
            Dbg(_) => unimplemented!(),
            Left => Move(-1),
            Right => Move(1),
            Dec => Const(-1),
            Inc => Const(1),
            LBrac => LB,
            RBrac => RB,
            Input => In,
            Output => Out,
        }
    }
}
