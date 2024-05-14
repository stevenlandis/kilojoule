#[cfg(test)]
mod tests {
    use kilojoule_rust::*;
    use once_cell::sync::Lazy;
    use serde_json::json;

    static PARSER: Lazy<Parser> = Lazy::new(|| Parser::new());

    fn base_parse_and_eval(expr: &str) -> Vec<u8> {
        let ast = PARSER.parse(expr);
        // println!("Ast: {:?}", ast);
        let result = eval_ast_node(&Val::new_null(), &ast);
        let mut out = Vec::<u8>::new();
        result.write_json_str(&mut out);
        return out;
    }

    fn parse_and_eval_json(expr: &str) -> serde_json::Value {
        let out = base_parse_and_eval(expr);
        return serde_json::from_str(std::str::from_utf8(out.as_slice()).unwrap()).unwrap();
    }

    fn assert_json(expr: &str, value: serde_json::Value) {
        assert_eq!(parse_and_eval_json(expr), value);
    }

    #[test]
    fn deserialize_and_serialize() {
        assert_json("123", json!(123));
        assert_json("{a: 1, b: 2}", json!({"a": 1, "b": 2}));
        assert_json("{a: 1, b: 2} | .b", json!(2));
        assert_json("{a: 1, b: 2} | {a: .b, b: .a}", json!({"a": 2, "b": 1}));
        assert_json("{}", json!({}));
        assert_json("{a:{b:{c:42}}}", json!({'a': {'b': {'c': 42}}}));
    }
}
