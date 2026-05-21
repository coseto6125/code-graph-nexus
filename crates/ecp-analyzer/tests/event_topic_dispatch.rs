//! T5-1 unit tests: config-table dispatch + import-gate filtering.
//!
//! Deliberately framework-agnostic — no Kafka / RabbitMQ / NATS specifics.
//! Those belong to T5-2..T5-N.

use ecp_analyzer::event_topic::{
    classify_amqp_direction, classify_kafka_direction, extract_event_topics, EventTopicConfig,
};
use ecp_core::analyzer::types::{FrameworkId, PubSub, RawImport};
use ecp_core::pool::StringPool;
use tree_sitter::{Parser, Query};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Build a tree-sitter tree for the snippet using the Python grammar.
fn python_tree(src: &str) -> (tree_sitter::Tree, tree_sitter::Language) {
    let lang: tree_sitter::Language = tree_sitter_python::LANGUAGE.into();
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language");
    let tree = parser
        .parse(src.as_bytes(), None)
        .expect("parse returned None");
    (tree, lang)
}

/// Fabricate a `RawImport` that looks like `from <source> import *`.
fn fake_import(source: &str) -> RawImport {
    RawImport {
        source: source.to_string(),
        imported_name: "*".to_string(),
        alias: None,
        binding_kind: None,
    }
}

// ---------------------------------------------------------------------------
// Query used by all dispatch tests.
//
// Pattern: Python string assignment with an optional right-hand side literal,
// modelling a synthetic topic publisher call:
//   topic = "User.Created"
//
// Captures: @topic_name = the string value node.
// ---------------------------------------------------------------------------
const TOPIC_QUERY: &str = r#"
(assignment
  left: (identifier) @var
  right: (string) @topic_name)
"#;

// ---------------------------------------------------------------------------
// Helper: strip Python string delimiters for use in assertions.
// ---------------------------------------------------------------------------
fn strip_quotes(s: &str) -> &str {
    s.trim_matches('"').trim_matches('\'')
}

// ---------------------------------------------------------------------------
// Synthetic configs — no real framework imports.
// ---------------------------------------------------------------------------

const CONFIG_KAFKA: EventTopicConfig = EventTopicConfig {
    framework: FrameworkId::Kafka,
    topic_capture: "topic_name",
    producer_capture: "",
    direction_capture: "",
    import_gate: &["kafka"],
    direction_classifier: classify_kafka_direction,
    canonicalize: true,
};

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// No configs → empty Vec, no panic.
#[test]
fn test_empty_configs_returns_empty() {
    let src = r#"topic = "user.created""#;
    let (tree, lang) = python_tree(src);
    let query = Query::new(&lang, TOPIC_QUERY).expect("query compile");
    let mut pool = StringPool::new();

    let result = extract_event_topics(&tree, src.as_bytes(), &query, &[], &[], &mut pool);

    assert!(result.is_empty(), "no configs must return empty");
}

/// Config requires `kafka` import; file has no such import → empty Vec.
#[test]
fn test_import_gate_blocks_when_absent() {
    let src = r#"topic = "order.created""#;
    let (tree, lang) = python_tree(src);
    let query = Query::new(&lang, TOPIC_QUERY).expect("query compile");
    let mut pool = StringPool::new();

    let imports = vec![fake_import("pika")]; // rabbitmq, not kafka
    let result = extract_event_topics(
        &tree,
        src.as_bytes(),
        &query,
        &[CONFIG_KAFKA],
        &imports,
        &mut pool,
    );

    assert!(result.is_empty(), "import gate must block");
}

/// Config requires `kafka` import; file imports `kafka.producer` → emits.
#[test]
fn test_import_gate_passes_when_present() {
    let src = r#"topic = "order.created""#;
    let (tree, lang) = python_tree(src);
    let query = Query::new(&lang, TOPIC_QUERY).expect("query compile");
    let mut pool = StringPool::new();

    let imports = vec![fake_import("kafka.producer")];
    let result = extract_event_topics(
        &tree,
        src.as_bytes(),
        &query,
        &[CONFIG_KAFKA],
        &imports,
        &mut pool,
    );

    assert_eq!(
        result.len(),
        1,
        "import gate should pass for kafka.producer"
    );
    assert_eq!(result[0].lib, FrameworkId::Kafka);
}

/// config.canonicalize=true — raw topic `"User.Created"` is emitted as the
/// canonical form produced by T5-0 `canonicalize()`.
#[test]
fn test_canonicalize_applied_when_enabled() {
    use ecp_analyzer::event_topic::normalize::canonicalize;

    let raw_topic = "User.Created";
    let src = format!(r#"topic = "{raw_topic}""#);
    let (tree, lang) = python_tree(&src);
    let query = Query::new(&lang, TOPIC_QUERY).expect("query compile");
    let mut pool = StringPool::new();

    let imports = vec![fake_import("kafka")];
    let result = extract_event_topics(
        &tree,
        src.as_bytes(),
        &query,
        &[CONFIG_KAFKA], // canonicalize: true
        &imports,
        &mut pool,
    );

    assert_eq!(result.len(), 1);
    let str_ref = result[0].topic_literal.expect("topic_literal");
    let emitted = pool.resolve(&str_ref).to_string();
    let expected = canonicalize(raw_topic);
    assert_eq!(
        emitted, expected,
        "canonicalize=true must apply T5-0 normalization"
    );
}

/// config.canonicalize=false — raw text is emitted verbatim (minus the Python
/// string delimiters that the tree-sitter capture includes).
#[test]
fn test_canonicalize_skipped_when_disabled() {
    const CONFIG_RAW: EventTopicConfig = EventTopicConfig {
        framework: FrameworkId::Kafka,
        topic_capture: "topic_name",
        producer_capture: "",
        direction_capture: "",
        import_gate: &["kafka"],
        direction_classifier: classify_kafka_direction,
        canonicalize: false,
    };

    let raw_topic = "User.Created";
    let src = format!(r#"topic = "{raw_topic}""#);
    let (tree, lang) = python_tree(&src);
    let query = Query::new(&lang, TOPIC_QUERY).expect("query compile");
    let mut pool = StringPool::new();

    let imports = vec![fake_import("kafka")];
    let result = extract_event_topics(
        &tree,
        src.as_bytes(),
        &query,
        &[CONFIG_RAW],
        &imports,
        &mut pool,
    );

    assert_eq!(result.len(), 1);
    let str_ref = result[0].topic_literal.expect("topic_literal");
    let emitted = pool.resolve(&str_ref).to_string();
    // Tree-sitter captures the full string node including delimiters; strip for comparison.
    let emitted_stripped = strip_quotes(&emitted);
    assert_eq!(
        emitted_stripped, raw_topic,
        "canonicalize=false must not transform the topic"
    );
}

/// Two configs both gated on the same import; declaration order determines
/// which fires (first match wins).
#[test]
fn test_multiple_configs_first_match_wins() {
    const CONFIG_FIRST: EventTopicConfig = EventTopicConfig {
        framework: FrameworkId::Kafka,
        topic_capture: "topic_name",
        producer_capture: "",
        direction_capture: "",
        import_gate: &["kafka"],
        direction_classifier: classify_kafka_direction,
        canonicalize: true,
    };

    const CONFIG_SECOND: EventTopicConfig = EventTopicConfig {
        framework: FrameworkId::Sns, // different framework, same gate
        topic_capture: "topic_name",
        producer_capture: "",
        direction_capture: "",
        import_gate: &["kafka"],
        direction_classifier: classify_kafka_direction,
        canonicalize: true,
    };

    let src = r#"topic = "order.placed""#;
    let (tree, lang) = python_tree(src);
    let query = Query::new(&lang, TOPIC_QUERY).expect("query compile");
    let mut pool = StringPool::new();

    let imports = vec![fake_import("kafka")];
    let result = extract_event_topics(
        &tree,
        src.as_bytes(),
        &query,
        &[CONFIG_FIRST, CONFIG_SECOND],
        &imports,
        &mut pool,
    );

    assert_eq!(
        result.len(),
        1,
        "first match wins — second config must not fire"
    );
    assert_eq!(
        result[0].lib,
        FrameworkId::Kafka,
        "first config in declaration order must win"
    );
}

/// Synthetic direction capture `"consume"` with `classify_amqp_direction`
/// must produce `PubSub::Subscribe`.
///
/// Uses a Python call expression `send("consume", "order.placed")` so that
/// both @direction_capture and @topic_name land in the same tree-sitter match.
/// (Separate assignment statements produce separate matches with only one
/// capture each — they cannot satisfy a config that requires both captures.)
#[test]
fn test_direction_classifier_invoked() {
    // Query: Python call node where the first argument is the direction and the
    // second is the topic.  Both captures come from the same match.
    const CALL_QUERY: &str = r#"
(call
  arguments: (argument_list
    (string) @direction_capture
    (string) @topic_name))
"#;

    const CONFIG_AMQP: EventTopicConfig = EventTopicConfig {
        framework: FrameworkId::RabbitMq,
        topic_capture: "topic_name",
        producer_capture: "",
        direction_capture: "direction_capture",
        import_gate: &["pika"],
        direction_classifier: classify_amqp_direction,
        canonicalize: false,
    };

    let src = r#"send("consume", "order.placed")"#;
    let (tree, lang) = python_tree(src);
    let query = Query::new(&lang, CALL_QUERY).expect("query compile");
    let mut pool = StringPool::new();

    let imports = vec![fake_import("pika")];
    let result = extract_event_topics(
        &tree,
        src.as_bytes(),
        &query,
        &[CONFIG_AMQP],
        &imports,
        &mut pool,
    );

    assert_eq!(result.len(), 1, "expected one topic from call expression");
    assert_eq!(
        result[0].direction,
        PubSub::Subscribe,
        "classify_amqp_direction(\"consume\") must produce PubSub::Subscribe"
    );
    let str_ref = result[0].topic_literal.expect("topic_literal");
    let emitted = pool.resolve(&str_ref);
    assert_eq!(
        emitted, "order.placed",
        "topic text must be emitted verbatim (canonicalize=false)"
    );
}
