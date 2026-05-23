use ecp_analyzer::ruby::parser::RubyProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use std::path::Path;

fn parse_rb(src: &str) -> ecp_core::analyzer::types::LocalGraph {
    let provider = RubyProvider::new().expect("RubyProvider::new");
    provider
        .parse_file(Path::new("test.rb"), src.as_bytes())
        .expect("parse_file")
}

fn kinds(g: &ecp_core::analyzer::types::LocalGraph) -> Vec<&str> {
    g.blind_spots.iter().map(|b| b.kind.as_str()).collect()
}

// ── eval / instance_eval: always blind ──

#[test]
fn ruby_eval_emits_blind_spot() {
    let src = "def run(code); eval(code); end";
    let g = parse_rb(src);
    assert!(
        kinds(&g).contains(&"rb-eval"),
        "expected rb-eval; got: {:?}",
        kinds(&g)
    );
}

#[test]
fn ruby_eval_with_literal_still_emits_blind_spot() {
    let src = "eval('puts 1')";
    let g = parse_rb(src);
    assert!(
        kinds(&g).contains(&"rb-eval"),
        "literal-arg eval still blind; got: {:?}",
        kinds(&g)
    );
}

#[test]
fn ruby_instance_eval_emits_blind_spot() {
    let src = "obj.instance_eval { do_stuff }";
    let g = parse_rb(src);
    assert!(
        kinds(&g).contains(&"rb-instance-eval"),
        "expected rb-instance-eval; got: {:?}",
        kinds(&g)
    );
}

// ── send: literal-symbol vs variable check ──

#[test]
fn ruby_send_with_variable_emits_blind_spot() {
    let src = "def dispatch(obj, name); obj.send(name); end";
    let g = parse_rb(src);
    assert!(
        kinds(&g).contains(&"rb-send"),
        "expected rb-send for variable arg; got: {:?}",
        kinds(&g)
    );
}

#[test]
fn ruby_send_with_literal_symbol_skipped() {
    // obj.send(:method) is statically resolvable (the method name is the
    // symbol literal) — must NOT emit per Constraint 2.
    let src = "obj.send(:to_s)";
    let g = parse_rb(src);
    assert!(
        !kinds(&g).contains(&"rb-send"),
        "literal symbol send must NOT emit; got: {:?}",
        kinds(&g)
    );
}

#[test]
fn ruby_send_with_literal_string_skipped() {
    // obj.send("to_s") — also statically resolvable.
    let src = "obj.send('to_s')";
    let g = parse_rb(src);
    assert!(
        !kinds(&g).contains(&"rb-send"),
        "literal string send must NOT emit; got: {:?}",
        kinds(&g)
    );
}

// ── negative ──

#[test]
fn ruby_ordinary_call_emits_no_blind_spot() {
    let src = "def add(a, b); a + b; end\nadd(1, 2)";
    let g = parse_rb(src);
    assert!(
        g.blind_spots.is_empty(),
        "ordinary call must not emit; got: {:?}",
        g.blind_spots
    );
}
