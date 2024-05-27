#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use kilojoule_rust::*;
    use once_cell::sync::Lazy;
    use serde_json::json;

    static PARSER: Lazy<Parser> = Lazy::new(|| Parser::new());

    fn base_parse_and_eval(expr: &str) -> Vec<u8> {
        let ast = PARSER.parse(expr).unwrap();
        // println!("Ast: {:?}", ast);
        let result = eval_ast_node(&Val::new_null(), &ast, &HashMap::new());
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
    fn test_chained_access() {
        assert_json("{a: {b: {c: 42}}}.a.b.c", json!(42));
        assert_json("{a: {b: {c: 42}}} | .a.b.c", json!(42));
        assert_json("{a: {b: {c: 42}}}['a']['b']['c']", json!(42));
        assert_json("{a: {b: {c: 42}}} | .['a']['b']['c']", json!(42));

        assert_json("null.a", json!(null));
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
        assert_json(r#"' \{\}\n\t\r\"\'\\ '"#, json!(" {}\n\t\r\"'\\ "));
        assert_json(r#"' " '"#, json!(" \" "));
        assert_json(r#"'Name: \'{1}\''"#, json!(r#"Name: '1'"#));

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
    fn test_format_string_double_quote() {
        assert_json(r#""a {1}""#, json!("a 1"));
        assert_json(r#""{1} a""#, json!("1 a"));
        assert_json(r#""a {1} b""#, json!("a 1 b"));
        assert_json(r#""a {1} b {2} c""#, json!("a 1 b 2 c"));
        assert_json(r#""a {1}{2} c""#, json!("a 12 c"));
        assert_json(r#""{1}{2}{3}""#, json!("123"));
        assert_json(r#"" \{\}\n\t\r\"\'\\ ""#, json!(" {}\n\t\r\"'\\ "));
        assert_json(r#"" ' ""#, json!(" ' "));
        assert_json(r#""Name: \"{1}\"""#, json!(r#"Name: "1""#));

        // nested
        assert_json(r#""a {"b c"}""#, json!("a b c"));
        assert_json(r#""a {"b {42} c"}""#, json!("a b 42 c"));
        assert_json(r#""a {"b {"c {42}"} d"}""#, json!("a b c 42 d"));

        // array
        assert_json(
            r#"[1,2,3] | "before{.}after" "#,
            json!("before[1,2,3]after"),
        );

        // dict
        assert_json(
            r#"{a: 1, b: 2} | "before{.}after""#,
            json!(r#"before{"a":1,"b":2}after"#),
        );

        // boolean
        assert_json(r#"[false, true] | "{.[0]}||{.[1]}""#, json!("false||true"));
    }

    #[test]
    fn test_len() {
        assert_json("[1,2,3] | len()", json!(3));
        assert_json("{a: 1, b: 2, c: 3, d: 4} | len()", json!(4));
    }

    #[test]
    fn test_map_fcn() {
        assert_json("[[],[1,2],[1]] | map(len())", json!([0, 2, 1]));
    }

    #[test]
    fn test_group_fcn() {
        assert_json(
            "[[1,'a'], [2,'b'], [1,'c'], [3,'d'], [2,'e']] | group(.[0])",
            json!([
                {"key": 1, "rows": [
                    [1, 'a'],
                    [1, 'c']
                ]},
                {"key": 2, "rows": [
                    [2, 'b'],
                    [2, 'e']
                ]},
                {"key": 3, "rows": [
                    [3, 'd'],
                ]},
            ]),
        );
    }

    #[test]
    fn test_unique() {
        assert_json("[3,1,2,3,1,5] | unique()", json!([3, 1, 2, 5]));
    }

    #[test]
    fn test_sort() {
        assert_json("[4,1,3,2,5] | sort()", json!([1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_null() {
        assert_json("null", json!(null));
    }

    #[test]
    fn test_add_and_subtract() {
        assert_json("2 + 3", json!(5));
        assert_json("10 - 3", json!(7));
        assert_json("1 + 2 - 10", json!(-7));
        assert_json("1 - 2 + 10", json!(9));
    }

    #[test]
    fn test_equality_operators() {
        assert_json("[4, 5, 6] | map(. == 5)", json!([false, true, false]));
        assert_json("[4, 5, 6] | map(. != 5)", json!([true, false, true]));
        assert_json("[4, 5, 6] | map(. < 5)", json!([true, false, false]));
        assert_json("[4, 5, 6] | map(. <= 5)", json!([true, true, false]));
        assert_json("[4, 5, 6] | map(. > 5)", json!([false, false, true]));
        assert_json("[4, 5, 6] | map(. >= 5)", json!([false, true, true]));
    }

    #[test]
    fn test_or_and_and() {
        assert_json(
            "[
                [false, false],
                [false, true],
                [true, false],
                [true, true],
            ] | map(.[0] or .[1])",
            json!([false, true, true, true]),
        );

        assert_json(
            "[
                [false, false],
                [false, true],
                [true, false],
                [true, true],
            ] | map(.[0] and .[1])",
            json!([false, false, false, true]),
        );
    }

    #[test]
    fn test_filter() {
        assert_json(
            "[1, 2, 3, 4, 5, 6] | filter(. < 3 or . > 4)",
            json!([1, 2, 5, 6]),
        );
    }

    #[test]
    fn test_multiply_and_divide() {
        assert_json("3*4", json!(12));
        assert_json("8/2", json!(4));
        assert_json("3*5/3", json!(5));
        assert_json("3 * 4 + 100*2", json!(3 * 4 + 100 * 2));
        assert_json("1/0", json!({"ERROR": "divide by zero"}));
    }

    #[test]
    fn test_sum() {
        assert_json("[1,2,3,4] | sum()", json!(10));
        assert_json("[] | sum()", json!(0));
    }

    #[test]
    fn test_list_access() {
        assert_json("[1,2,3,4,5] | .[1]", json!(2));
        assert_json("[1,2,3,4,5] | [.[/0], .[/1], .[/4]]", json!([5, 4, 1]));

        assert_json("[1,2,3,4,5] | .[0:]", json!([1, 2, 3, 4, 5]));
        assert_json("[1,2,3,4,5] | .[1:]", json!([2, 3, 4, 5]));
        assert_json("[1,2,3,4,5] | .[2:]", json!([3, 4, 5]));
        assert_json("[1,2,3,4,5] | .[3:]", json!([4, 5]));
        assert_json("[1,2,3,4,5] | .[4:]", json!([5]));
        assert_json("[1,2,3,4,5] | .[5:]", json!([]));
        assert_json("[1,2,3,4,5] | .[6:]", json!([]));

        assert_json("[1,2,3,4,5] | .[:/0]", json!([1, 2, 3, 4, 5]));
        assert_json("[1,2,3,4,5] | .[:/1]", json!([1, 2, 3, 4]));
        assert_json("[1,2,3,4,5] | .[:/2]", json!([1, 2, 3]));
        assert_json("[1,2,3,4,5] | .[:/3]", json!([1, 2]));
        assert_json("[1,2,3,4,5] | .[:/4]", json!([1]));
        assert_json("[1,2,3,4,5] | .[:/5]", json!([]));
        assert_json("[1,2,3,4,5] | .[:/6]", json!([]));

        assert_json("[1,2,3,4,5] | .[:0]", json!([]));
        assert_json("[1,2,3,4,5] | .[:1]", json!([1]));
        assert_json("[1,2,3,4,5] | .[:2]", json!([1, 2]));
        assert_json("[1,2,3,4,5] | .[:3]", json!([1, 2, 3]));
        assert_json("[1,2,3,4,5] | .[:4]", json!([1, 2, 3, 4]));
        assert_json("[1,2,3,4,5] | .[:5]", json!([1, 2, 3, 4, 5]));
        assert_json("[1,2,3,4,5] | .[:6]", json!([1, 2, 3, 4, 5]));

        assert_json("[1,2,3,4,5] | .[/0:]", json!([]));
        assert_json("[1,2,3,4,5] | .[/1:]", json!([5]));
        assert_json("[1,2,3,4,5] | .[/2:]", json!([4, 5]));
        assert_json("[1,2,3,4,5] | .[/3:]", json!([3, 4, 5]));
        assert_json("[1,2,3,4,5] | .[/4:]", json!([2, 3, 4, 5]));
        assert_json("[1,2,3,4,5] | .[/5:]", json!([1, 2, 3, 4, 5]));
        assert_json("[1,2,3,4,5] | .[/6:]", json!([1, 2, 3, 4, 5]));

        assert_json("[1,2,3,4,5] | .[0:5]", json!([1, 2, 3, 4, 5]));
        assert_json("[1,2,3,4,5] | .[1:4]", json!([2, 3, 4]));
        assert_json("[1,2,3,4,5] | .[0:0]", json!([]));
        assert_json("[1,2,3,4,5] | .[5:5]", json!([]));
        assert_json("[1,2,3,4,5] | .[1:0]", json!([]));

        assert_json("[1,2,3,4,5] | .[/5:/0]", json!([1, 2, 3, 4, 5]));
        assert_json("[1,2,3,4,5] | .[/4:/1]", json!([2, 3, 4]));
        assert_json("[1,2,3,4,5] | .[/0:/0]", json!([]));
        assert_json("[1,2,3,4,5] | .[/5:/5]", json!([]));
        assert_json("[1,2,3,4,5] | .[/0:/1]", json!([]));

        assert_json("[1,2,3,4,5] | .[0:/0]", json!([1, 2, 3, 4, 5]));
    }

    #[test]
    fn test_variable() {
        assert_json("1 | let a = . + 5 | a + .", json!(7));
        assert_json("1 | . | let a = . + 5 | a + .", json!(7));
        assert_json("let a = 5 | a + 2", json!(7));
    }

    #[test]
    fn test_lines() {
        assert_json(
            "'line 0\n\nline 1\nline 2 \n' | lines()",
            json!(["line 0", "", "line 1", "line 2 "]),
        );
    }

    #[test]
    fn test_split() {
        assert_json(
            "'one,two,,three,' | split(',')",
            json!(["one", "two", "", "three", ""]),
        );
        assert_json("'' | split(',')", json!([""]));
    }

    #[test]
    fn test_join() {
        assert_json(
            "['one', 'two', '', 'three', ''] | join(',')",
            json!("one,two,,three,"),
        );
        assert_json("[] | join(',')", json!(""));
        assert_json("[''] | join(',')", json!(""));
    }

    #[test]
    fn test_coalesce() {
        assert_json("null ?? 'val1'", json!("val1"));
        assert_json("'val0' ?? 'val1'", json!("val0"));

        assert_json("'val0' ?? 'val1' ?? 'val2'", json!("val0"));
        assert_json("null ?? 'val1' ?? 'val2'", json!("val1"));
        assert_json("null ?? null ?? 'val2'", json!("val2"));
        assert_json("null ?? null ?? null", json!(null));
    }

    #[test]
    fn test_keys_values_entries() {
        assert_json("{a: 1, b: 2, c: 3} | keys()", json!(["a", "b", "c"]));
        assert_json("{a: 1, b: 2, c: 3} | values()", json!([1, 2, 3]));
        assert_json(
            "{a: 1, b: 2, c: 3} | entries()",
            json!([
                {"key": "a", "val": 1},
                {"key": "b", "val": 2},
                {"key": "c", "val": 3},
            ]),
        );
    }
}
