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
        result.write_json_str(&mut out, true);
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
    fn basic_functionality() {
        assert_json("123", json!(123));
        assert_json("{a: 1, b: 2}", json!({"a": 1, "b": 2}));
        assert_json("{a: 1, b: 2,}", json!({"a": 1, "b": 2}));
        assert_json("{a: 1, b: 2} | .b", json!(2));
        assert_json("{a: 1, b: 2} | {a: .b, b: .a}", json!({"a": 2, "b": 1}));
        assert_json("{}", json!({}));
        assert_json("{a:{b:{c:42}}}", json!({'a': {'b': {'c': 42}}}));
        assert_json("[]", json!([]));
        assert_json("[1]", json!([1]));
        assert_json("[1,2,3]", json!([1, 2, 3]));
        assert_json("[1,2,3,]", json!([1, 2, 3]));
        assert_json(
            "{a: 1, b: 2} | [., .a, .b]",
            json!([{"a": 1, "b": 2}, 1, 2]),
        );
        assert_json("[100, 200, 300] | .[1]", json!(200));
    }

    #[test]
    fn string_literals() {
        assert_json(r#" '' "#, json!(""));
        assert_json(r#" 'string' "#, json!("string"));
        assert_json(r#" '"' "#, json!("\""));

        assert_json(r#" "" "#, json!(""));
        assert_json(r#" "string" "#, json!("string"));
        assert_json(r#" "'" "#, json!("'"));
    }

    #[test]
    fn test_format_string_single_quote() {
        assert_json(r#"'a {1}'"#, json!("a 1"));
        assert_json(r#"'{1} a'"#, json!("1 a"));
        assert_json(r#"'a {1} b'"#, json!("a 1 b"));
        assert_json(r#"'a {1} b {2} c'"#, json!("a 1 b 2 c"));
        assert_json(r#"'a {1}{2} c'"#, json!("a 12 c"));
        assert_json(r#"'{1}{2}{3}'"#, json!("123"));

        // nested
        assert_json(r#"'a {'b c'}'"#, json!("a b c"));
        assert_json(r#"'a {'b {42} c'}'"#, json!("a b 42 c"));
        assert_json(r#"'a {'b {'c {42}'} d'}'"#, json!("a b c 42 d"));

        // array
        assert_json(
            r#"[1,2,3] | 'before{.}after' "#,
            json!("before[1,2,3]after"),
        );

        // dict
        assert_json(
            r#"{a: 1, b: 2} | 'before{.}after'"#,
            json!(r#"before{"a":1,"b":2}after"#),
        );

        // boolean
        assert_json(r#"[false, true] | '{.[0]}||{.[1]}'"#, json!("false||true"));
    }

    #[test]
    fn test_len() {
        assert_json("[1,2,3] | len()", json!(3));
        assert_json("{a: 1, b: 2, c: 3, d: 4} | len()", json!(4));
    }
}
