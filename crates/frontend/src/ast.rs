#![forbid(unsafe_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub decls: Vec<Decl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decl {
    Import(ImportDecl),
    Global(Binding),
    Let(Binding),
    Type(TypeDecl),
    Func(FuncDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDecl {
    pub module: Ident,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub mutable: bool,
    pub name: Ident,
    pub ty: Type,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDecl {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncDecl {
    pub name: Ident,
    pub params: Vec<Param>,
    pub ret: Option<Type>,
    pub body: Expr, // block or expression
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub mutable: bool,
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stmt {
    Binding(Binding),
    Assign(Assign),
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assign {
    pub target: Path,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub tail: Option<Box<Expr>>, // if None, unit is implied
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Literal(Literal),
    Path(Path),
    Copy(Box<Expr>),
    Ref(Box<Expr>),
    FuncCall(FuncCall),
    If(Box<IfExpr>),
    Block(Block),
    RecordLit(RecordLit),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncCall {
    pub callee: Path,
    pub args: Vec<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfExpr {
    pub cond: Expr,
    pub then_branch: Expr,
    pub else_branch: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordLit {
    pub fields: Vec<FieldInit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldInit {
    pub name: Ident,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: BinaryOp,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Mul,
    Div,
    Add,
    Sub,
    Lt,
    Eq,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Int(i64),
    Bool(bool),
    Str(String),
    Unit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Named(Ident),
    Ref(Box<Type>),
    Record(Vec<FieldType>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldType {
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(pub Vec<Ident>);
