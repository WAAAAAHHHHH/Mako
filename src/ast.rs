#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    Ident(String),
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    Tuple(Vec<Expr>),
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>),
    UnaryNot(Box<Expr>),
    Call { callee: String, args: Vec<Expr> },
    /// expr[index]
    Index { target: Box<Expr>, index: Box<Expr> },
    /// expr.field
    MemberAccess { object: Box<Expr>, field: String },
    /// expr.method(args)
    MethodCall { object: Box<Expr>, method: String, args: Vec<Expr> },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// !name = expr
    VarDecl { name: String, init: Expr },
    /// const name = expr
    ConstDecl { name: String, init: Expr },
    /// name = expr
    Assign { name: String, expr: Expr },
    /// name[index] = value
    IndexAssign { name: String, index: Expr, value: Expr },
    /// name.field = value  (also covers self.field = value)
    MemberAssign { name: String, field: String, value: Expr },
    /// say! expr, ...
    Say { exprs: Vec<Expr> },
    /// Just an expression evaluated for side effects
    Expr(Expr),
    /// A block of statements wrapped in begin...end
    Block(Vec<Stmt>),
    /// if / elif / else
    IfStmt {
        condition: Expr,
        then_branch: Box<Stmt>,
        elif_branches: Vec<(Expr, Stmt)>,
        else_branch: Option<Box<Stmt>>,
    },
    /// while condition begin...end
    WhileStmt { condition: Expr, body: Box<Stmt> },
    /// for var in iterable begin...end
    ForStmt { var: String, iterable: Expr, body: Box<Stmt> },
    Stop,
    Skip,
    /// fn name(params) begin...end
    FnDecl { name: String, params: Vec<String>, body: Box<Stmt> },
    /// give expr
    Give(Expr),
    /// type Name begin fn ... end
    TypeDecl { name: String, methods: Vec<Stmt> },
    /// use "file.mako"
    Use(String),
    /// try begin...end catch err begin...end
    TryCatch {
        try_block: Box<Stmt>,
        error_var: String,
        catch_block: Box<Stmt>,
    },
    /// throw expr
    Throw(Expr),
    /// match expr case val begin...end ... else begin...end
    MatchStmt {
        target: Expr,
        cases: Vec<(Expr, Stmt)>,
        else_branch: Option<Box<Stmt>>,
    },
}

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}
