use std::collections::HashMap;

use super::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FastBF {
    Inst(StackInst),
    Move(isize),                  // </>
    Const(isize),                 // +/-
    LB,                           // [ Left Bracket
    RB,                           // ] Right Bracket
    In,                           // , input
    Out,                          // . output
    AddCell(Vec<(isize, isize)>), // Add this cell value to various locations, a certain number of times
    BinAnd,
    BinOr,
    BinXor,
    Mult,
    Rem,
    ShiftR,
    Store,
    Read,
    Geq,
}

pub fn optimize(bf: &[BF]) -> Vec<FastBF> {
    use FastBF::*;

    let mut fast = convert(bf);

    let mut ip = 0;
    'outer: loop {
        match &fast[ip..] {
            [Move(0) | Const(0), ..] => {
                fast.remove(ip);
                continue;
            }
            [Const(a), Const(b), ..] => {
                fast[ip] = Const(a + b);
                fast.remove(ip + 1);
                continue;
            }
            [Move(a), Move(b), ..] => {
                fast[ip] = Move(a + b);
                fast.remove(ip + 1);
                continue;
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
                            map.remove(&0);
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

fn convert(bf: &[BF]) -> Vec<FastBF> {
    use FastBF::*;
    use StackInst::*;
    const SNIPPETS: &[(StackInst, FastBF)] = &[
        (StkRead, Read),
        (StkStr, Store),
        (And, BinAnd),
        (Or, BinOr),
        (Xor, BinXor),
        (GrEq, Geq),
        (Mod, Rem),
        (RShift, ShiftR),
        (Mul, Mult),
    ];

    let mut fast_code = vec![];

    let mut i = 0;
    'outer: while i < bf.len() {
        for (inst, fast) in SNIPPETS.iter().cloned() {
            let mut snippet = vec![];
            emit_bf(inst, &mut snippet);

            if bf[i..].starts_with(&snippet) {
                fast_code.push(fast);
                i += snippet.len();
                continue 'outer;
            }
        }
        fast_code.push(bf[i].into());
        i += 1;
    }

    fast_code
}

impl From<BF> for FastBF {
    fn from(value: BF) -> Self {
        use FastBF::*;
        use BF::*;
        match value {
            Profile(p) => Inst(p),
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
