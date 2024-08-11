#[cfg(test)]
mod tests {
    use kilojoule::*;
    use serde_json::json;

    fn base_parse_and_eval(expr: &str) -> (Vec<u8>, Vec<u8>) {
        let result = EvalCtx::new().parse_and_eval(expr).val;

        let mut out0 = Vec::<u8>::new();
        EvalCtx::write_val(&result, &mut out0, true).unwrap();

        // make sure output is valid json when indent=false
        let mut out1 = Vec::<u8>::new();
        EvalCtx::write_val(&result, &mut out1, false).unwrap();

        return (out0, out1);
    }

    fn parse_and_eval_json(expr: &str) -> serde_json::Value {
        let (out0, out1) = base_parse_and_eval(expr);

        let j0: serde_json::Value =
            serde_json::from_str(std::str::from_utf8(out0.as_slice()).unwrap()).unwrap();
        let j1: serde_json::Value =
            serde_json::from_str(std::str::from_utf8(out1.as_slice()).unwrap()).unwrap();

        assert_eq!(j0, j1);

        return j0;
    }

    fn assert_json(expr: &str, value: serde_json::Value) {
        assert_eq!(parse_and_eval_json(expr), value);
    }

    #[test]
    fn basic_functionality() {
        assert_json("123", json!(123));
        assert_json("42 | . | .", json!(42));
        assert_json("{ a : 1, b : 2 }", json!({"a": 1, "b": 2}));
        assert_json("{ a : 1 , b : 2 , }", json!({"a": 1, "b": 2}));
        assert_json("{a: 1, b: 2} | . b", json!(2));
        assert_json("{a: 1, b: 2} | {a: .b, b: .a}", json!({"a": 2, "b": 1}));
        assert_json("{}", json!({}));
        assert_json("{a:{b:{c:42}}}", json!({'a': {'b': {'c': 42}}}));
        assert_json("[]", json!([]));
        assert_json("[1]", json!([1]));
        assert_json("[1,2,3]", json!([1, 2, 3]));
        assert_json("[ 1 , 2 , 3 , ]", json!([1, 2, 3]));
        assert_json(
            "{a: 1, b: 2} | [., .a, .b]",
            json!([{"a": 1, "b": 2}, 1, 2]),
        );
        assert_json("[100, 200, 300] | .[1]", json!(200));
    }

    #[test]
    fn test_chained_access() {
        assert_json("{a: {b: {c: 42}}} . a . b . c", json!(42));
        assert_json("{a: {b: {c: 42}}} | .a.b.c", json!(42));
        assert_json("{a: {b: {c: 42}}}['a']['b']['c']", json!(42));
        assert_json("{a: {b: {c: 42}}} | .['a']['b']['c']", json!(42));

        assert_json("null.a", json!(null));
    }

    #[test]
    fn test_access_expression_scope() {
        assert_json("[[1,2,3,4], 3] | .[0][:.[1]]", json!([1, 2, 3]));
        assert_json("[[1,2,3,4], 2] | .[0][:.[1]]", json!([1, 2]));
    }

    #[test]
    fn test_number_literals() {
        assert_json("1", json!(1));
        assert_json("1.5", json!(1.5));
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
    fn test_variable_in_f_string() {
        assert_json("let a = 42 | 'a is {a}'", json!("a is 42"))
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
                {"key": 1, "vals": [
                    [1, 'a'],
                    [1, 'c']
                ]},
                {"key": 2, "vals": [
                    [2, 'b'],
                    [2, 'e']
                ]},
                {"key": 3, "vals": [
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
        assert_json("[4,1,3,2,5] | sort(.)", json!([1, 2, 3, 4, 5]));
        assert_json(
            "[ [true,false], [false,true] ] | map(sort(.))",
            json!([[false, true], [false, true]]),
        );
        assert_json(
            "[
                bad_fcn(),
                'text',
                true,
                123,
                null,
                {},
                [],
            ] | sort(.)",
            json!([
                {"ERROR": "Unknown function \"bad_fcn\""},
                null,
                true,
                123,
                "text",
                [],
                {}
            ]),
        );
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
    fn test_and_and_or() {
        assert_json(
            "[1, 2, 3, 4, 5, 6, 7] | filter(. >= 6 or 2 <= . and . <= 4)",
            json!([2, 3, 4, 6, 7]),
        );

        // make sure expression is different with different associativity
        assert_json(
            "[1, 2, 3, 4, 5, 6, 7] | filter((. >= 6 or 2 <= .) and . <= 4)",
            json!([2, 3, 4]),
        );
    }

    #[test]
    fn test_not() {
        assert_json("not false", json!(true));
        assert_json("not true", json!(false));
        assert_json("not not false", json!(false));
        assert_json("not not true", json!(true));
        assert_json("false or not false", json!(true));
        assert_json("false or not true", json!(false));
        assert_json("not 1 > 2 or false", json!(true));
        assert_json("not 1 < 2 or false", json!(false));
        assert_json(
            "not",
            json!({"ERROR": "Parser error 'NoExprAfterUnaryOperator' at index 3"}),
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
    fn test_negative() {
        assert_json("-100", json!(-100));
        assert_json("1 + -100", json!(-99));
        assert_json("--100", json!(100));
        assert_json("4 * -3", json!(-12));
    }

    #[test]
    fn test_sum() {
        assert_json("[1,2,3,4] | sum()", json!(10));
        assert_json("[] | sum()", json!(0));
    }

    #[test]
    fn test_min() {
        assert_json("[1,2,3,4] | min()", json!(1));
        assert_json("[] | min()", json!(null));
    }

    #[test]
    fn test_max() {
        assert_json("[1,2,3,4] | max()", json!(4));
        assert_json("[] | max()", json!(null));
    }

    #[test]
    fn test_any() {
        assert_json("[] | any()", json!(false));
        assert_json("[false] | any()", json!(false));
        assert_json("[false, false] | any()", json!(false));
        assert_json("[false, true] | any()", json!(true));
        assert_json("[true, false] | any()", json!(true));
        assert_json("[true, true] | any()", json!(true));

        assert_json("[1,2,3] | any(. == 2)", json!(true));
        assert_json("[1,2,3] | any(. == 42)", json!(false));
    }

    #[test]
    fn test_all() {
        assert_json("[] | all()", json!(true));
        assert_json("[false] | all()", json!(false));
        assert_json("[true] | all()", json!(true));
        assert_json("[false, false] | all()", json!(false));
        assert_json("[false, true] | all()", json!(false));
        assert_json("[true, false] | all()", json!(false));
        assert_json("[true, true] | all()", json!(true));

        assert_json("[1,2,3] | all(. < 4)", json!(true));
        assert_json("[1,2,3] | all(. < 3)", json!(false));
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

        assert_json("[1,2,3,4,5] | .[:]", json!([1, 2, 3, 4, 5]));

        assert_json("['a'] | .[ 0 ]", json!("a"));
        assert_json("['a'] | .[ / 0 ]", json!("a"));
        assert_json("['a'] | .[ 0 : ]", json!(["a"]));
        assert_json("['a'] | .[ / 0 : ]", json!([]));
        assert_json("['a'] | .[ : 0 ]", json!([]));
        assert_json("['a'] | .[ : / 0 ]", json!(["a"]));
        assert_json("['a'] | .[ : ]", json!(["a"]));
    }

    // #[test]
    // fn test_variable() {
    //     assert_json("1 | let a = . + 5 | a + .", json!(7));
    //     assert_json("1 | . | let a = . + 5 | a + .", json!(7));
    //     assert_json("let a = 5 | a + 2", json!(7));
    // }

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

        assert_json(
            "'stuff     and\n\t\r\n\tthings' | split()",
            json!(["stuff", "and", "things"]),
        );
    }

    #[test]
    fn test_join() {
        assert_json(
            "['one', 'two', '', 'three', ''] | join(',')",
            json!("one,two,,three,"),
        );
        assert_json("[] | join(',')", json!(""));
        assert_json("[''] | join(',')", json!(""));
        assert_json(
            "['stuff', 1, 'things'] | join(' - ')",
            json!("stuff - 1 - things"),
        );
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
    fn test_keys_values_items() {
        assert_json("{a: 1, b: 2, c: 3} | keys()", json!(["a", "b", "c"]));
        assert_json("{a: 1, b: 2, c: 3} | values()", json!([1, 2, 3]));
        assert_json(
            "{a: 1, b: 2, c: 3} | items()",
            json!([
                {"key": "a", "val": 1},
                {"key": "b", "val": 2},
                {"key": "c", "val": 3},
            ]),
        );
        assert_json(
            "{a: 1, b: 2, c: 3} | items() | from_items()",
            json!(
                {"a": 1, "b": 2, "c": 3}
            ),
        );
        assert_json(
            "[['a', 1], ['b', 2]] | from_items()",
            json!({"a": 1, "b": 2}),
        );
    }

    #[test]
    fn test_recursive_functions() {
        let mock_recursive_obj = r#"{
            name: "a",
            children: [
                {name: "b", children: []},
                {
                    name: "c",
                    children: [
                        {name: "d", children: []},
                        {name: "e", children: []}
                    ],
                },
            ],
        }"#;

        assert_json(
            format!("{} | recursive_map(.children, {{name2: .node.name, count: .vals | len(), children2: .vals}})", mock_recursive_obj).as_str(),
        json!({
            "name2": "a",
            "count": 2,
            "children2": [
                {"name2": "b", "count": 0, "children2": []},
                {
                    "name2": "c",
                    "count": 2,
                    "children2": [
                        {"name2": "d", "count": 0, "children2": []},
                        {"name2": "e", "count": 0, "children2": []},
                    ],
                },
            ],
        }));

        assert_json(
            format!(
                "{} | recursive_flatten(.children) | map(.name)",
                mock_recursive_obj
            )
            .as_str(),
            json!(["a", "b", "c", "d", "e",]),
        );
    }

    #[test]
    fn test_list_spread() {
        assert_json("[0, * [1,2,3], 4]", json!([0, 1, 2, 3, 4]));
    }

    #[test]
    fn test_map_spread() {
        assert_json(
            "{a: 1, *{b: 2, a: 3}, d: 4}",
            json!({"a": 3, "b": 2, "d": 4}),
        );
    }

    #[test]
    fn test_map_delete() {
        assert_json("{a: 1, b: 2, c: 3} | {*., -'b'}", json!({"a": 1, "c": 3}));
        assert_json(
            "{a: 1, b: 2, c: 3} | {*., -'b', b: 5}",
            json!({"a": 1, "c": 3, "b": 5}),
        );
    }

    #[test]
    fn test_map_key_expression() {
        assert_json("{['a']: 1, [21*2]: 2}", json!({"a": 1, "42": 2}));
    }

    #[test]
    fn test_map_f_string_key_expression() {
        assert_json("42 | {'a': 1, 'b{.}': 2}", json!({"a": 1, "b42": 2}));
    }

    #[test]
    fn test_range() {
        assert_json("range(0)", json!([]));
        assert_json("range(1)", json!([0]));
        assert_json("range(2)", json!([0, 1]));
        assert_json("range(-2)", json!([0, -1]));

        assert_json("range(1, 4)", json!([1, 2, 3]));
        assert_json("range(4, 1)", json!([4, 3, 2]));
        assert_json("range(1, 1)", json!([]));

        assert_json("range(:step 3, 1, 10)", json!([1, 4, 7]));
        assert_json("range(1, :step 3, 10)", json!([1, 4, 7]));
        assert_json("range(1, 10, :step 3)", json!([1, 4, 7]));

        assert_json("range(-1, -10, :step -3)", json!([-1, -4, -7]));

        assert_json("range(1, 4, :step -1)", json!([]));
        assert_json("range(4, 1, :step 1)", json!([]));
    }

    #[test]
    fn test_zip() {
        assert_json("[] | zip()", json!([]));
        assert_json("[[1,2,3]] | zip()", json!([[1], [2], [3]]));
        assert_json(
            "[['a', 'b', 'c'], [1, 2, 3]] | zip()",
            json!([["a", 1], ["b", 2], ["c", 3]]),
        );
        assert_json(
            "[['a', 'b', 'c', 'd'], [1, 2, 3]] | zip()",
            json!([["a", 1], ["b", 2], ["c", 3]]),
        );
        assert_json(
            "[['a', 'b', 'c'], [1, 2, 3, 4]] | zip()",
            json!([["a", 1], ["b", 2], ["c", 3]]),
        );
        assert_json(
            "[['a', 'b', 'c'], [1, 2, 3], ['x', 'y', 'z']] | zip()",
            json!([["a", 1, "x"], ["b", 2, "y"], ["c", 3, "z"]]),
        );
    }

    #[test]
    fn test_repeat() {
        assert_json("42 | repeat(0)", json!([]));
        assert_json("42 | repeat(1)", json!([42]));
        assert_json("42 | repeat(3)", json!([42, 42, 42]));
    }

    #[test]
    fn test_from_text_table() {
        assert_json(
            "'key0 key1  key2\na b c\na\na b c d' | from_text_table()",
            json!([
                {"key0": "a", "key1": "b", "key2": "c"},
                {"key0": "a", "key1": null, "key2": null},
                {"key0": "a", "key1": "b", "key2": "c d"},
            ]),
        )
    }

    #[test]
    fn test_assign() {
        assert_json("100 | let a = 4 | . + a + 7", json!(111));
        assert_json("let a = 1 | ([] | map(let a = 2) | a)", json!(1))
    }

    #[test]
    fn test_flatten() {
        assert_json(
            "[[1,2], [3,4,5], [], [6]] | flatten()",
            json!([1, 2, 3, 4, 5, 6]),
        );
        assert_json("[]", json!([]));
    }

    #[test]
    fn test_is_type() {
        assert_json("1/0 | is_err()", json!(true));
        assert_json("1 | is_err()", json!(false));

        assert_json("1 | is_number()", json!(true));
        assert_json("null | is_number()", json!(false));

        assert_json("false | is_bool()", json!(true));
        assert_json("null | is_bool()", json!(false));

        assert_json("'string' | is_string()", json!(true));
        assert_json("null | is_string()", json!(false));

        assert_json("[] | is_list()", json!(true));
        assert_json("null | is_list()", json!(false));

        assert_json("{} | is_map()", json!(true));
        assert_json("null | is_map()", json!(false));

        assert_json("'' | bytes() | is_bytes()", json!(true));
        assert_json("null | is_map()", json!(false));
    }

    #[test]
    fn test_catch() {
        assert_json("[42, 1/0] | map(catch(100))", json!([42, 100]))
    }

    #[test]
    fn test_if() {
        assert_json("if(true, 1, 2)", json!(1));
        assert_json("if(false, 1, 2)", json!(2));
    }

    #[test]
    fn test_has() {
        assert_json("{a: 1} | has('a')", json!(true));
        assert_json("{a: 1} | has('b')", json!(false));

        assert_json("[1,2,3] | has(2)", json!(true));
        assert_json("[1,2,3] | has(42)", json!(false));
    }

    #[test]
    fn test_map_keys() {
        assert_json(
            "{a: 1, b: 2} | map_keys('key_{.}')",
            json!({"key_a": 1, "key_b": 2}),
        );
    }

    #[test]
    fn test_map_values() {
        assert_json("{a: 1, b: 2} | map_values(.*10)", json!({"a": 10, "b": 20}));
    }

    #[test]
    fn test_from_json() {
        assert_json(r#" "true" | from_json() "#, json!(true));
        assert_json(r#" "123" | from_json() "#, json!(123));
        assert_json(r#" "123.25" | from_json() "#, json!(123.25));
        assert_json(r#" '"some text"' | from_json() "#, json!("some text"));
        assert_json(r#" 'null' | from_json() "#, json!(null));
        assert_json(r#" '"\\\""' | from_json() "#, json!("\""));
        assert_json(r#" "\{\}" | from_json() "#, json!({}));
        assert_json(r#" "[]" | from_json() "#, json!([]));

        assert_json(
            r#" '\{"a": 1, "b": [2, false, null]\}' | from_json() "#,
            json!({"a": 1, "b": [2, false, null]}),
        );
    }

    #[test]
    fn test_to_json() {
        assert_json("{a: 1, b: 2} | to_json()", json!("{\"a\":1,\"b\":2}"));
    }

    #[test]
    fn test_to_toml() {
        assert_json("{a: 1, b: 2} | to_toml()", json!("a = 1\nb = 2\n"));
    }

    #[test]
    fn test_to_and_from_yaml() {
        assert_json("{a: 1, b: 2} | to_yaml()", json!("a: 1\nb: 2\n"));
        assert_json(
            "{a: 1, b: 2} | to_yaml() | from_yaml()",
            json!({"a": 1, "b": 2}),
        );
    }

    #[test]
    fn test_from_num() {
        assert_json("'123.25' | from_num()", json!(123.25));
    }

    #[test]
    fn test_combinations() {
        assert_json(
            "[ ['a', 'b'], [0, 1, 2] ] | combinations()",
            json!([['a', 0], ['a', 1], ['a', 2], ['b', 0], ['b', 1], ['b', 2],]),
        );

        assert_json("[] | combinations()", json!([[]]));
        assert_json("[[1,2,3]] | combinations()", json!([[1], [2], [3]]));

        assert_json(
            "[ ['a', 'b'], [0, 1, 2], ['x'] ] | combinations()",
            json!([
                ['a', 0, 'x'],
                ['a', 1, 'x'],
                ['a', 2, 'x'],
                ['b', 0, 'x'],
                ['b', 1, 'x'],
                ['b', 2, 'x'],
            ]),
        );
    }

    #[test]
    fn test_starts_with() {
        assert_json("'stuff' | starts_with('')", json!(true));
        assert_json("'stuff' | starts_with('s')", json!(true));
        assert_json("'stuff' | starts_with('st')", json!(true));
        assert_json("'stuff' | starts_with('stu')", json!(true));

        assert_json("'stuff' | starts_with('stx')", json!(false));
    }

    #[test]
    fn test_ends_with() {
        assert_json("'things' | ends_with('')", json!(true));
        assert_json("'things' | ends_with('s')", json!(true));
        assert_json("'things' | ends_with('gs')", json!(true));
        assert_json("'things' | ends_with('ngs')", json!(true));

        assert_json("'things' | ends_with('ngx')", json!(false));
    }

    #[test]
    fn test_lower() {
        assert_json("'StUfF' | lower()", json!("stuff"));
    }

    #[test]
    fn test_upper() {
        assert_json("'StUfF' | upper()", json!("STUFF"));
    }

    #[test]
    fn test_trim() {
        assert_json("'stuff\\r' | trim()", json!("stuff"));
        assert_json("'    \\n\\tstuff\\r' | trim()", json!("stuff"));
    }
}
