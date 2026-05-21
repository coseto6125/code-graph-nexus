use ecp_analyzer::kotlin::parser::KotlinProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{FrameworkId, LocalGraph, RawTxScope};
use std::path::Path;

fn parse(src: &str) -> LocalGraph {
    let p = KotlinProvider::new().expect("KotlinProvider::new");
    p.parse_file(Path::new("Test.kt"), src.as_bytes())
        .expect("parse_file")
}

fn scopes(graph: &LocalGraph) -> &[RawTxScope] {
    graph.tx_scopes.as_deref().unwrap_or(&[])
}

fn fn_name_of(graph: &LocalGraph, scope_idx: usize) -> &str {
    let s = &scopes(graph)[scope_idx];
    graph.nodes[s.node_idx() as usize].name.as_str()
}

fn fn_name_of_scope<'g>(graph: &'g LocalGraph, scope: &RawTxScope) -> &'g str {
    graph.nodes[scope.node_idx() as usize].name.as_str()
}

#[test]
fn annotated_function_emits_tx_scope() {
    let src = r#"
import org.springframework.transaction.annotation.Transactional

class OrderService {
    @Transactional
    fun placeOrder() {}

    fun listOrders() {}
}
"#;
    let g = parse(src);
    assert_eq!(
        scopes(&g).len(),
        1,
        "exactly one tx_scope expected; got: {:?}",
        scopes(&g)
            .iter()
            .map(|s| fn_name_of_scope(&g, s))
            .collect::<Vec<_>>()
    );
    assert_eq!(fn_name_of(&g, 0), "placeOrder");
    assert_eq!(scopes(&g)[0].framework(), FrameworkId::SpringTransactional);
}

#[test]
fn non_annotated_function_produces_no_tx_scope() {
    let src = r#"
class UserService {
    fun getUser() {}
    fun deleteUser() {}
}
"#;
    let g = parse(src);
    assert!(scopes(&g).is_empty(), "no tx_scope expected");
    assert!(g.tx_scopes.is_none());
}

#[test]
fn parameterized_transactional_emits_tx_scope() {
    let src = r#"
import org.springframework.transaction.annotation.Transactional
import org.springframework.transaction.annotation.Propagation

class PaymentService {
    @Transactional(propagation = Propagation.REQUIRES_NEW)
    fun processPayment() {}
}
"#;
    let g = parse(src);
    assert_eq!(scopes(&g).len(), 1);
    assert_eq!(fn_name_of(&g, 0), "processPayment");
    assert_eq!(scopes(&g)[0].framework(), FrameworkId::SpringTransactional);
}

#[test]
fn multiple_annotated_functions_each_emit_tx_scope() {
    let src = r#"
import org.springframework.transaction.annotation.Transactional

class AccountService {
    @Transactional
    fun deposit() {}

    @Transactional
    fun withdraw() {}

    fun readBalance() {}
}
"#;
    let g = parse(src);
    assert_eq!(scopes(&g).len(), 2);
    let names: Vec<&str> = scopes(&g).iter().map(|s| fn_name_of_scope(&g, s)).collect();
    assert!(names.contains(&"deposit"), "deposit missing: {:?}", names);
    assert!(names.contains(&"withdraw"), "withdraw missing: {:?}", names);
}
