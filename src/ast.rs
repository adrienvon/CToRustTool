/// C语言AST节点定义
#[derive(Debug, Clone, PartialEq)]
pub enum CType {
    // 基本类型
    Int,
    Char,
    Float,
    Double,
    Void,
    Long,
    Short,
    UnsignedInt,
    UnsignedChar,
    UnsignedLong,
    UnsignedShort,
    SignedInt,
    SignedChar,

    // 复合类型
    Pointer(Box<CType>),
    Array {
        element_type: Box<CType>,
        size: Option<usize>,
    },
    Function {
        return_type: Box<CType>,
        params: Vec<CType>,
    },

    // 用户定义类型
    Struct(String),
    Union(String),
    Enum(String),
    Typedef(String),

    // 类型修饰符
    Const(Box<CType>),
    Volatile(Box<CType>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // 算术运算符
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // 比较运算符
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,

    // 逻辑运算符
    And,
    Or,

    // 位运算符
    BitAnd,
    BitOr,
    BitXor,
    LeftShift,
    RightShift,

    // 复合赋值运算符
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    LeftShiftAssign,
    RightShiftAssign,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,           // -x
    Not,           // !x
    BitNot,        // ~x
    Deref,         // *x
    AddressOf,     // &x
    PreIncrement,  // ++x
    PreDecrement,  // --x
    PostIncrement, // x++
    PostDecrement, // x--
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    IntLiteral(i32),
    FloatLiteral(f64),
    CharLiteral(char),
    StringLiteral(String),
    Identifier(String),
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Call {
        func: String,
        args: Vec<Expr>,
    },
    Assignment {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Cast {
        typ: CType,
        expr: Box<Expr>,
    },
    ArrayAccess {
        array: Box<Expr>,
        index: Box<Expr>,
    },
    MemberAccess {
        object: Box<Expr>,
        member: String,
    },
    PointerMemberAccess {
        object: Box<Expr>,
        member: String,
    },
    Ternary {
        cond: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
    },
    SizeOf(CType),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    VarDecl {
        typ: CType,
        name: String,
        init: Option<Expr>,
    },
    Return(Option<Expr>),
    Expr(Expr),
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    DoWhile {
        body: Vec<Stmt>,
        cond: Expr,
    },
    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        update: Option<Expr>,
        body: Vec<Stmt>,
    },
    Switch {
        expr: Expr,
        cases: Vec<SwitchCase>,
    },
    Break,
    Continue,
    Goto(String),
    Label(String),
    Block(Vec<Stmt>),
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub value: Option<Expr>, // None表示default
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub typ: CType,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub return_type: CType,
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}

// 结构体定义
#[derive(Debug, Clone, PartialEq)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub typ: CType,
    pub name: String,
}

// 联合体定义
#[derive(Debug, Clone, PartialEq)]
pub struct UnionDef {
    pub name: String,
    pub fields: Vec<StructField>,
}

// 枚举定义
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<i32>,
}

// Typedef定义
#[derive(Debug, Clone, PartialEq)]
pub struct TypedefDef {
    pub name: String,
    pub target_type: CType,
}

// 全局声明
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Function(Function),
    Struct(StructDef),
    Union(UnionDef),
    Enum(EnumDef),
    Typedef(TypedefDef),
    GlobalVar {
        typ: CType,
        name: String,
        init: Option<Expr>,
    },
    Include(String),
    Define {
        name: String,
        value: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}
