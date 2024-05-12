#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum RuleType {
    Main__Expr_END,
    Expr__AddExpr,
    AddExpr__MulExpr,
    AddExpr__AddExpr_PLUS_MulExpr,
    MulExpr__INTEGER,
    MulExpr__MulExpr_ASTERISK_INTEGER,
}
