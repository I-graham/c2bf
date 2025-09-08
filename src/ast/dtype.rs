use super::*;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum DType {
    Void,
    U8,
    S8,
    U16,
    S16,
    U32,
    S32,
    U64,
    S64,
    Float,
    Double,
    Pointer(usize, Box<Self>), // Level of indirection + base type
    Array(Word, Box<DType>),
    Unsized(Box<DType>),
    Function(Vec<DType>, Box<DType>),
}

impl ASTNode for DType {
    fn parse(pair: Pair<Rule>) -> Self {
        use DType::*;
        parser_rule! {
            pair :

            type_name
                [sql] -> sql;
                [sql, decl:Declarator] -> {
                    decl.set_type(sql)
                };

            specifier_qualifier_list
                [.. sql,] -> {
                    let specs = sql.filter(|s| s.as_rule() == type_specifier);

                    let mut long = false;
                    let mut signed = true;
                    let mut ty = S16;

                    for spec in specs {
                        match spec.as_str() {
                            "void" => ty = Void,
                            "unsigned" => signed = false,
                            "signed" => signed = true,
                            "char" => ty = S8,
                            "short" => ty = S16,
                            "int" => ty = S16,
                            "long" => {
                                long = true;
                                ty = S32;
                            }
                            "long" if long => ty = S64,
                            "float" => ty = Float,
                            "double" => ty = Double,
                            _ => unreachable!(),
                        }
                    }

                    if !signed {
                        ty = ty.make_unsigned();
                    }

                    ty
                };
        }
    }
}

impl DType {
    pub fn pointer(self) -> Self {
        use DType::*;
        match self {
            Pointer(n, b) => Pointer(n + 1, b),
            b => Pointer(1, b.into()),
        }
    }

    fn make_unsigned(&self) -> Self {
        use DType::*;
        match self {
            S8 => U8,
            S16 => U16,
            S32 => U32,
            S64 => U64,
            _ => unreachable!(),
        }
    }

    pub fn size(&self) -> Word {
        use DType::*;
        match self {
            Void => 0,
            U8 | S8 => 1,
            U16 | S16 => 1,
            U32 | S32 => 4,
            U64 | S64 => 8,
            Float => 4,
            Double => 8,
            Pointer(_, _) | Unsized(_) => 1,
            Array(n, dtype) => n * dtype.size(),
            Function(_, _) => unreachable!(),
        }
    }
}
