#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Token {
    INTEGER,
    PLUS,
    ASTERISK,
    END,
    Main,
    Expr,
    AddExpr,
    MulExpr,
}
