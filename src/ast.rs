/// C语言AST节点定义
#[derive(Debug, Clone, PartialEq)]
pub enum CType {
    Int,
    Char,
    Float,
    Double,
    Void,
    Pointer(Box<CType>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
    Deref,
    AddressOf,
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
    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        update: Option<Expr>,
        body: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
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

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
}
