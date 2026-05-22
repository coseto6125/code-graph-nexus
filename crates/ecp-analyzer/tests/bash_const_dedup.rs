//! Bash `Const` deduplication — each shell variable name is emitted at most once,
//! regardless of how many times it is re-assigned (top-level, inside `if`, inside
//! `while`, etc.).  This prevents uid collisions for files like test.sh that do
//! `X=; if ...; X=v; fi`.

use ecp_analyzer::bash::parser::BashProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::graph::NodeKind;
use std::path::Path;

fn parse_nodes(src: &str) -> Vec<(String, NodeKind)> {
    let provider = BashProvider::new().expect("BashProvider::new");
    let graph = provider
        .parse_file(Path::new("t.sh"), src.as_bytes())
        .expect("parse_file");
    graph
        .nodes
        .iter()
        .map(|n| (n.name.clone(), n.kind))
        .collect()
}

fn const_names(src: &str) -> Vec<String> {
    parse_nodes(src)
        .into_iter()
        .filter(|(_, k)| *k == NodeKind::Const)
        .map(|(n, _)| n)
        .collect()
}

/// Reassignment at top-level must not produce a duplicate Const node.
#[test]
fn test_bash_const_top_level_reassign_dedup() {
    let src = "X=1\nX=2\n";
    let names = const_names(src);
    assert_eq!(
        names,
        vec!["X"],
        "X should appear exactly once, got {names:?}"
    );
}

/// Reassignment inside an `if` block must not duplicate the top-level declaration.
#[test]
fn test_bash_const_if_block_reassign_dedup() {
    // Mirrors the hiredis test.sh pattern:
    //   ENABLE_DEBUG_CMD=
    //   if [ ... ]; then
    //       ENABLE_DEBUG_CMD="value"
    //   fi
    let src = "ENABLE_DEBUG_CMD=\nif [ \"$X\" -gt 0 ]; then\n    ENABLE_DEBUG_CMD=value\nfi\n";
    let names = const_names(src);
    assert_eq!(
        names,
        vec!["ENABLE_DEBUG_CMD"],
        "ENABLE_DEBUG_CMD should appear exactly once, got {names:?}"
    );
}

/// Multiple distinct variable names still produce one Const each.
#[test]
fn test_bash_const_distinct_names_all_emitted() {
    let src = "A=1\nB=2\nC=3\n";
    let mut names = const_names(src);
    names.sort();
    assert_eq!(
        names,
        vec!["A", "B", "C"],
        "each distinct name must be emitted once"
    );
}

/// Variable that is only assigned once emits exactly one Const.
#[test]
fn test_bash_const_single_assignment_emits_one() {
    let src = "MY_VAR=hello\n";
    let names = const_names(src);
    assert_eq!(names, vec!["MY_VAR"]);
}

/// Reassignment inside a while loop must not duplicate.
#[test]
fn test_bash_const_while_loop_reassign_dedup() {
    let src = "COUNTER=0\nwhile [ \"$COUNTER\" -lt 5 ]; do\n    COUNTER=$((COUNTER + 1))\ndone\n";
    let names = const_names(src);
    assert_eq!(
        names,
        vec!["COUNTER"],
        "loop re-assignment must not produce a duplicate"
    );
}

/// Aliases (Typedef) are unaffected by the Const dedup logic.
#[test]
fn test_bash_alias_unaffected_by_const_dedup() {
    let src = "X=1\nX=2\nalias ll='ls -la'\n";
    let nodes = parse_nodes(src);
    let consts: Vec<_> = nodes
        .iter()
        .filter(|(_, k)| *k == NodeKind::Const)
        .collect();
    let aliases: Vec<_> = nodes
        .iter()
        .filter(|(_, k)| *k == NodeKind::Typedef)
        .collect();
    assert_eq!(consts.len(), 1, "X must appear exactly once as Const");
    assert_eq!(aliases.len(), 1, "alias ll must still be emitted");
}

/// Functions are unaffected: a function with the same name as a variable emits both.
#[test]
fn test_bash_function_not_affected_by_const_dedup() {
    let src = "setup() { echo hi; }\nsetup=value\n";
    let nodes = parse_nodes(src);
    let funcs: Vec<_> = nodes
        .iter()
        .filter(|(_, k)| *k == NodeKind::Function)
        .collect();
    let consts: Vec<_> = nodes
        .iter()
        .filter(|(_, k)| *k == NodeKind::Const)
        .collect();
    assert_eq!(funcs.len(), 1, "function `setup` must be emitted");
    assert_eq!(consts.len(), 1, "const `setup` must be emitted once");
}
