use std::collections::HashMap;

use ramon_templates::{OwnedValue, Parser, Value};

const _A: f64 = 4.0;
const A: f64 = 8.2;
const B: f64 = 16.0;

fn eval<'a>(input: &'a str) -> String {
    let template = Parser::parse_input(input).unwrap();
    let mut vars = HashMap::new();
    vars.insert("_a".into(), OwnedValue::Number(_A));
    vars.insert("a".into(), OwnedValue::Number(A));
    vars.insert("b".into(), OwnedValue::Number(B));
    vars.insert("world".into(), OwnedValue::String("world".into()));
    template
        .evaluate(
            &vars,
            &HashMap::<String, Box<dyn Fn(Vec<Value>) -> OwnedValue>>::new(),
        )
        .unwrap()
}

#[test]
fn literals() {
    let out = eval("Hello, world!");
    assert_eq!(&out, "Hello, world!");

    let out = eval("{{'{{'}}}}");
    assert_eq!(&out, "{{}}");

    let out = eval("{a}");
    assert_eq!(&out, "{a}");
}

#[test]
fn vars() {
    let out = eval("a = {{  _a}}");
    assert_eq!(out, format!("a = {_A}"));
}

#[test]
fn math() {
    let out = eval("a*b = {{a*b\n\n\t\t  }}");
    assert_eq!(out, format!("a*b = {}", A * B));

    let out = eval("a+b = {{a + b}}");
    assert_eq!(out, format!("a+b = {}", A + B));

    let out = eval("b-a = {{b - a}}");
    assert_eq!(out, format!("b-a = {}", B - A));

    let out = eval("{{ a + a * b * (a + b) / b}}");
    assert_eq!(out, (A + A * B * (A + B) / B).to_string());

    let out = eval("{{ -42.42 }}");
    assert_eq!(out, "-42.42");

    let out = eval("{{ 2--2 }}");
    assert_eq!(out, "4");
}

#[test]
fn logic() {
    let out = eval("{{if a==  8.2}}a == 8.2{{/if}}");
    assert_eq!(out, "a == 8.2");

    let out = eval("{{if a==  8}}a == 8{{else}}a != 8{{/if}}");
    assert_eq!(out, "a != 8");

    let out = eval("{{if a==b}}a == b{{elif a!=b}}a != b{{/if}}");
    assert_eq!(out, "a != b");

    let out = eval("{{if a==b}}a == b{{elif 1==1&&a==b-2}}a==b-2{{/if}}");
    assert_eq!(out, "");

    let out = eval("{{if a!=b&&a==b}}1{{elif 1!=1}}1!=1{{else}}else{{/if}}");
    assert_eq!(out, "else");
}

#[test]
fn for_loops() {
    let out = eval(
        "{{ for arr in [[1, 2], [3, 4,],] ' ' }}{{ for n in arr ',' }}{{ n }}{{ /for }}{{ /for }}",
    );
    assert_eq!(out, "1,2 3,4");
}

#[test]
fn arrays() {
    let out = eval("{{ [1, 2] + [3, 4] }}");
    assert_eq!(out, "1, 2, 3, 4");
}
