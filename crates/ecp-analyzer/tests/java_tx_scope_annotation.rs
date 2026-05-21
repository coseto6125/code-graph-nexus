use ecp_analyzer::java::parser::JavaProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{FrameworkId, LocalGraph};
use std::path::Path;

fn parse(src: &str) -> LocalGraph {
    let p = JavaProvider::new().expect("JavaProvider::new");
    p.parse_file(Path::new("Test.java"), src.as_bytes())
        .expect("parse_file")
}

fn scopes(graph: &LocalGraph) -> &[ecp_core::analyzer::types::RawTxScope] {
    graph.tx_scopes.as_deref().unwrap_or(&[])
}

fn fn_name_of(graph: &LocalGraph, scope_idx: usize) -> &str {
    let s = &scopes(graph)[scope_idx];
    graph.nodes[s.node_idx() as usize].name.as_str()
}

#[test]
fn annotated_method_emits_tx_scope() {
    let src = r#"
import org.springframework.transaction.annotation.Transactional;

public class OrderService {
    @Transactional
    public void placeOrder() {}

    public void listOrders() {}
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
fn non_annotated_method_produces_no_tx_scope() {
    let src = r#"
public class UserService {
    public void getUser() {}
    public void deleteUser() {}
}
"#;
    let g = parse(src);
    assert!(
        scopes(&g).is_empty(),
        "no tx_scope expected; got {}",
        scopes(&g).len()
    );
    assert!(
        g.tx_scopes.is_none(),
        "empty case should be None, not Some(empty slice)"
    );
}

#[test]
fn parameterized_transactional_emits_tx_scope() {
    let src = r#"
import org.springframework.transaction.annotation.Transactional;
import org.springframework.transaction.annotation.Propagation;

public class PaymentService {
    @Transactional(propagation = Propagation.REQUIRES_NEW)
    public void processPayment() {}
}
"#;
    let g = parse(src);
    assert_eq!(
        scopes(&g).len(),
        1,
        "tx_scope expected for @Transactional with args"
    );
    assert_eq!(fn_name_of(&g, 0), "processPayment");
    assert_eq!(scopes(&g)[0].framework(), FrameworkId::SpringTransactional);
}

#[test]
fn multiple_annotated_methods_each_emit_tx_scope() {
    let src = r#"
import org.springframework.transaction.annotation.Transactional;

public class AccountService {
    @Transactional
    public void deposit() {}

    @Transactional
    public void withdraw() {}

    public void readBalance() {}
}
"#;
    let g = parse(src);
    assert_eq!(
        scopes(&g).len(),
        2,
        "two tx_scopes expected; got {}",
        scopes(&g).len()
    );
    let names: Vec<&str> = scopes(&g).iter().map(|s| fn_name_of_scope(&g, s)).collect();
    assert!(names.contains(&"deposit"), "deposit missing: {:?}", names);
    assert!(names.contains(&"withdraw"), "withdraw missing: {:?}", names);
}

fn fn_name_of_scope<'g>(
    graph: &'g LocalGraph,
    scope: &ecp_core::analyzer::types::RawTxScope,
) -> &'g str {
    graph.nodes[scope.node_idx() as usize].name.as_str()
}
