#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum RuleType {
    main__expr_END,
    expr__assignExpr,
    assignExpr__opPipeExpr,
    assignExpr__LET_IDENTIFIER_EQUAL_opCoalesceExpr_PIPE_assignExpr,
    assignExpr__opPipeExpr_PIPE_LET_IDENTIFIER_EQUAL_opCoalesceExpr_PIPE_assignExpr,
    opPipeExpr__opCoalesceExpr,
    opPipeExpr__opPipeExpr_PIPE_opCoalesceExpr,
    opCoalesceExpr__opOrExpr,
    opCoalesceExpr__opCoalesceExpr_DOUBLE_QUESTION_opOrExpr,
    opOrExpr__opAndExpr,
    opOrExpr__opOrExpr_OR_opAndExpr,
    opAndExpr__opEqualityExpr,
    opAndExpr__opAndExpr_AND_opEqualityExpr,
    equalityOperator__DOUBLE_EQUALS,
    equalityOperator__NOT_EQUALS,
    equalityOperator__LESS_THAN,
    equalityOperator__LESS_THAN_OR_EQUAL,
    equalityOperator__GREATER_THAN,
    equalityOperator__GREATER_THAN_OR_EQUAL,
    opEqualityExpr__opAddExpr,
    opEqualityExpr__opAddExpr_equalityOperator_opAddExpr,
    opAddOperator__PLUS,
    opAddOperator__MINUS,
    opAddExpr__opMulExpr,
    opAddExpr__opAddExpr_opAddOperator_opMulExpr,
    opMulOperator__ASTERISK,
    opMulOperator__FORWARD_SLASH,
    opMulExpr__opAccessExpr,
    opMulExpr__opMulExpr_opMulOperator_opAccessExpr,
    opAccessExpr__baseExpr,
    opAccessExpr__opAccessExpr_DOT_IDENTIFIER,
    opAccessExpr__opAccessExpr_LEFT_BRACKET_listAccessExpr_RIGHT_BRACKET,
    baseExpr__baseDotExpr,
    baseExpr__baseDotAccess,
    baseExpr__mapExpr,
    baseExpr__listExpr,
    baseExpr__LEFT_PAREN_expr_RIGHT_PAREN,
    baseExpr__INTEGER,
    baseExpr__FLOAT,
    baseExpr__stringLiteral,
    baseExpr__TRUE,
    baseExpr__FALSE,
    baseExpr__NULL,
    baseExpr__fcnCallExpr,
    baseExpr__IDENTIFIER,
    baseDotExpr__DOT,
    baseDotAccess__DOT_IDENTIFIER,
    listAccessIdx__expr,
    listAccessIdx__FORWARD_SLASH_expr,
    listAccessExpr__listAccessIdx,
    listAccessExpr__listAccessIdx_COLON,
    listAccessExpr__COLON_listAccessIdx,
    listAccessExpr__listAccessIdx_COLON_listAccessIdx,
    listAccessExpr__COLON,
    mapExpr__LEFT_BRACE_RIGHT_BRACE,
    mapExpr__LEFT_BRACE_mapContents_RIGHT_BRACE,
    mapExpr__LEFT_BRACE_mapContents_COMMA_RIGHT_BRACE,
    mapContents__mapContentsElem,
    mapContents__mapContents_COMMA_mapContentsElem,
    mapContentsElem__IDENTIFIER_COLON_expr,
    listExpr__LEFT_BRACKET_RIGHT_BRACKET,
    listExpr__LEFT_BRACKET_listExprContents_RIGHT_BRACKET,
    listExpr__LEFT_BRACKET_listExprContents_COMMA_RIGHT_BRACKET,
    listExprContents__listElem,
    listExprContents__listExprContents_COMMA_listElem,
    listElem__expr,
    stringLiteral__STRING_SINGLE_QUOTE,
    stringLiteral__F_STRING_SINGLE_QUOTE_LEFT_innerFormatStringSingleQuote_F_STRING_SINGLE_QUOTE_RIGHT,
    innerFormatStringSingleQuote__expr,
    innerFormatStringSingleQuote__innerFormatStringSingleQuote_F_STRING_SINGLE_QUOTE_MIDDLE_expr,
    stringLiteral__STRING_DOUBLE_QUOTE,
    stringLiteral__F_STRING_DOUBLE_QUOTE_LEFT_innerFormatStringDoubleQuote_F_STRING_DOUBLE_QUOTE_RIGHT,
    innerFormatStringDoubleQuote__expr,
    innerFormatStringDoubleQuote__innerFormatStringDoubleQuote_F_STRING_DOUBLE_QUOTE_MIDDLE_expr,
    fcnCallExpr__IDENTIFIER_LEFT_PAREN_RIGHT_PAREN,
    fcnCallExpr__IDENTIFIER_LEFT_PAREN_fcnCallArgs_RIGHT_PAREN,
    fcnCallExpr__IDENTIFIER_LEFT_PAREN_fcnCallArgs_COMMA_RIGHT_PAREN,
    fcnCallArgs__expr,
    fcnCallArgs__fcnCallArgs_COMMA_expr,
}
