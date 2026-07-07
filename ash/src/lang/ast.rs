#[derive(Debug, Clone, PartialEq)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

// --- Script-level types ---

#[derive(Debug, Clone, PartialEq)]
pub struct Script {
    pub shebang: Option<ShebangDecl>,
    pub compact: Option<CompactConfig>,
    pub body: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShebangDecl {
    pub pos: Pos,
    pub engine: String,
    pub version: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompactConfig {
    pub pos: Pos,
    pub mode: String,
    pub window: String,
    pub strategy: String,
}

// --- Interpolation ---

#[derive(Debug, Clone, PartialEq)]
pub enum InterpType {
    Var(String),
    Cmd(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterpSpan {
    pub pos: Pos,
    pub typ: InterpType,
}

// --- Statement/Expression structs ---

#[derive(Debug, Clone, PartialEq)]
pub struct VarAssign {
    pub pos: Pos,
    pub name: String,
    pub value: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub pos: Pos,
    pub name: String,
    pub params: Vec<String>,
    pub body: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnCall {
    pub pos: Pos,
    pub name: String,
    pub args: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Include {
    pub pos: Pos,
    pub path: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Exec {
    pub pos: Pos,
    pub cmd: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Print {
    pub pos: Pos,
    pub message: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Exit {
    pub pos: Pos,
    pub code: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Env {
    pub pos: Pos,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub pos: Pos,
    pub cond: Box<Node>,
    pub body: Box<Node>,
    pub else_ifs: Vec<ElseIf>,
    pub else_body: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElseIf {
    pub pos: Pos,
    pub cond: Box<Node>,
    pub body: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    pub pos: Pos,
    pub var: String,
    pub list: Box<Node>,
    pub body: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub pos: Pos,
    pub cond: Box<Node>,
    pub body: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Return {
    pub pos: Pos,
    pub value: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Break {
    pub pos: Pos,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Continue {
    pub pos: Pos,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub pos: Pos,
    pub statements: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentCall {
    pub pos: Pos,
    pub prompt: Box<Node>,
    pub agent: Option<String>,
    pub subagent: String,
    pub model: Option<Box<Node>>,
    pub dir: Option<Box<Node>>,
    pub compact: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryTry {
    pub pos: Pos,
    pub body: Box<Node>,
    pub fail: Option<Box<Node>>,
    pub max: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvalTry {
    pub pos: Pos,
    pub body: Box<Node>,
    pub eval: Box<Node>,
    pub accept: Option<Box<Node>>,
    pub partial: Option<Box<Node>>,
    pub fail: Option<Box<Node>>,
    pub max: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaitBlock {
    pub pos: Pos,
    pub body: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Background {
    pub pos: Pos,
    pub stmt: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DirBlock {
    pub pos: Pos,
    pub dir: Box<Node>,
    pub body: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompactStmt {
    pub pos: Pos,
    pub arg: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionBlock {
    pub pos: Pos,
    pub body: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionToggle {
    pub pos: Pos,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WithinToggle {
    pub pos: Pos,
    pub active: bool,
    pub path: Option<Box<Node>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub pos: Pos,
    pub left: Box<Node>,
    pub op: String,
    pub right: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub pos: Pos,
    pub op: String,
    pub right: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarRef {
    pub pos: Pos,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteral {
    pub pos: Pos,
    pub value: String,
    pub interps: Vec<InterpSpan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextBlock {
    pub pos: Pos,
    pub value: String,
    pub interps: Vec<InterpSpan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilePath {
    pub pos: Pos,
    pub path: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntLiteral {
    pub pos: Pos,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatLiteral {
    pub pos: Pos,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoolLiteral {
    pub pos: Pos,
    pub value: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandSubst {
    pub pos: Pos,
    pub cmd: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLiteral {
    pub pos: Pos,
    pub elements: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexExpr {
    pub pos: Pos,
    pub object: Box<Node>,
    pub index: Box<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupExpr {
    pub pos: Pos,
    pub inner: Box<Node>,
}

// --- Node enum ---

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    VarAssign(VarAssign),
    FnDecl(FnDecl),
    FnCall(FnCall),
    Include(Include),
    Exec(Exec),
    Print(Print),
    Exit(Exit),
    Env(Env),
    IfStmt(IfStmt),
    ElseIf(ElseIf),
    ForStmt(ForStmt),
    WhileStmt(WhileStmt),
    Return(Return),
    Break(Break),
    Continue(Continue),
    Block(Block),
    AgentCall(AgentCall),
    BinaryTry(BinaryTry),
    EvalTry(EvalTry),
    WaitBlock(WaitBlock),
    Background(Background),
    DirBlock(DirBlock),
    CompactStmt(CompactStmt),
    SessionBlock(SessionBlock),
    SessionToggle(SessionToggle),
    WithinToggle(WithinToggle),
    BinaryExpr(BinaryExpr),
    UnaryExpr(UnaryExpr),
    VarRef(VarRef),
    StringLiteral(StringLiteral),
    TextBlock(TextBlock),
    FilePath(FilePath),
    IntLiteral(IntLiteral),
    FloatLiteral(FloatLiteral),
    BoolLiteral(BoolLiteral),
    CommandSubst(CommandSubst),
    ArrayLiteral(ArrayLiteral),
    IndexExpr(IndexExpr),
    GroupExpr(GroupExpr),
}

impl Node {
    pub fn pos(&self) -> &Pos {
        match self {
            Node::VarAssign(n) => &n.pos,
            Node::FnDecl(n) => &n.pos,
            Node::FnCall(n) => &n.pos,
            Node::Include(n) => &n.pos,
            Node::Exec(n) => &n.pos,
            Node::Print(n) => &n.pos,
            Node::Exit(n) => &n.pos,
            Node::Env(n) => &n.pos,
            Node::IfStmt(n) => &n.pos,
            Node::ElseIf(n) => &n.pos,
            Node::ForStmt(n) => &n.pos,
            Node::WhileStmt(n) => &n.pos,
            Node::Return(n) => &n.pos,
            Node::Break(n) => &n.pos,
            Node::Continue(n) => &n.pos,
            Node::Block(n) => &n.pos,
            Node::AgentCall(n) => &n.pos,
            Node::BinaryTry(n) => &n.pos,
            Node::EvalTry(n) => &n.pos,
            Node::WaitBlock(n) => &n.pos,
            Node::Background(n) => &n.pos,
            Node::DirBlock(n) => &n.pos,
            Node::CompactStmt(n) => &n.pos,
            Node::SessionBlock(n) => &n.pos,
            Node::SessionToggle(n) => &n.pos,
            Node::WithinToggle(n) => &n.pos,
            Node::BinaryExpr(n) => &n.pos,
            Node::UnaryExpr(n) => &n.pos,
            Node::VarRef(n) => &n.pos,
            Node::StringLiteral(n) => &n.pos,
            Node::TextBlock(n) => &n.pos,
            Node::FilePath(n) => &n.pos,
            Node::IntLiteral(n) => &n.pos,
            Node::FloatLiteral(n) => &n.pos,
            Node::BoolLiteral(n) => &n.pos,
            Node::CommandSubst(n) => &n.pos,
            Node::ArrayLiteral(n) => &n.pos,
            Node::IndexExpr(n) => &n.pos,
            Node::GroupExpr(n) => &n.pos,
        }
    }
}
