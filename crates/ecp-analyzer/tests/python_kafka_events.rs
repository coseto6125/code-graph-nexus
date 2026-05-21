//! T5-2 integration tests: Kafka Python event-topic detector.
//!
//! Verifies that `KAFKA_PYTHON` config wired into the Python parser correctly
//! emits `RawEventTopic` for kafka-python and aiokafka call sites, and correctly
//! emits nothing when the import gate is not satisfied or the topic is not a literal.

use ecp_analyzer::event_topic::{classify_kafka_direction, extract_event_topics, EventTopicConfig};
use ecp_core::analyzer::types::{FrameworkId, PubSub, RawImport};
use ecp_core::pool::StringPool;
use tree_sitter::{Parser, Query};

// ---------------------------------------------------------------------------
// Shared query — mirrors the Kafka patterns from frameworks.scm.
// ---------------------------------------------------------------------------

/// Inline version of the Kafka captures from `frameworks.scm`.
/// Using the same capture names so the config table is exercised end-to-end.
const KAFKA_QUERY: &str = r#"
;; kafka-python: sync send inside function
(function_definition
  name: (identifier) @kafka.producer_fn
  body: (block
    (_
      (call
        function: (attribute
          attribute: (identifier) @_send (#eq? @_send "send"))
        arguments: (argument_list
          . (string) @kafka.topic)))))

;; aiokafka: await send inside async function
(function_definition
  name: (identifier) @kafka.producer_fn
  body: (block
    (_
      (await
        (call
          function: (attribute
            attribute: (identifier) @_asend (#eq? @_asend "send"))
          arguments: (argument_list
            . (string) @kafka.topic))))))

;; confluent_kafka: produce inside function
(function_definition
  name: (identifier) @kafka.producer_fn
  body: (block
    (_
      (call
        function: (attribute
          attribute: (identifier) @_produce (#eq? @_produce "produce"))
        arguments: (argument_list
          . (string) @kafka.topic)))))
"#;

const KAFKA_CONFIG: EventTopicConfig = EventTopicConfig {
    framework: FrameworkId::Kafka,
    topic_capture: "kafka.topic",
    producer_capture: "kafka.producer_fn",
    direction_capture: "",
    import_gate: &["kafka", "aiokafka", "confluent_kafka", "faust"],
    direction_classifier: classify_kafka_direction,
    canonicalize: true,
};

/// Parse `src`, run the Kafka query, filter by `import_sources`.
fn run(
    src: &str,
    import_sources: &[&str],
) -> (Vec<ecp_core::analyzer::types::RawEventTopic>, StringPool) {
    let lang: tree_sitter::Language = tree_sitter_python::LANGUAGE.into();
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language");
    let tree = parser.parse(src.as_bytes(), None).expect("parse");
    let query = Query::new(&lang, KAFKA_QUERY).expect("query compile");
    let imports: Vec<RawImport> = import_sources
        .iter()
        .map(|s| RawImport {
            source: (*s).to_string(),
            imported_name: "*".to_string(),
            alias: None,
            binding_kind: None,
        })
        .collect();
    let mut pool = StringPool::new();
    let result = extract_event_topics(
        &tree,
        src.as_bytes(),
        &query,
        &[KAFKA_CONFIG],
        &imports,
        &mut pool,
    );
    (result, pool)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// kafka-python: `producer.send("orders", b"x")` inside a function
/// → 1 RawEventTopic, topic == "orders", lib == Kafka, direction Publish.
#[test]
fn test_kafka_producer_send_literal_topic() {
    let src = r#"
from kafka import KafkaProducer

def publish_order(data):
    p = KafkaProducer(bootstrap_servers="localhost:9092")
    p.send("orders", b"x")
"#;
    let (result, pool) = run(src, &["kafka"]);
    assert_eq!(result.len(), 1, "expected one RawEventTopic");
    assert_eq!(result[0].lib, FrameworkId::Kafka);
    assert_eq!(result[0].direction, PubSub::Publish);
    let lit = result[0].topic_literal.expect("topic_literal must be Some");
    // canonicalize("orders") == "orders" (no separators to normalize)
    assert_eq!(pool.resolve(&lit), "orders");
}

/// Non-literal topic variable — no RawEventTopic emitted (never fabricate).
///
/// `extract_event_topics` only fires when the tree-sitter query produces a
/// `kafka.topic` capture that binds to a `string` node.  A bare identifier
/// (`topic`) does not match the `(string)` constraint in the query, so no
/// capture is produced and the extractor correctly returns nothing.
#[test]
fn test_kafka_variable_topic_emits_nothing() {
    let src = r#"
from kafka import KafkaProducer

def publish():
    topic = "orders"
    producer = KafkaProducer()
    producer.send(topic, b"payload")
"#;
    let (result, _pool) = run(src, &["kafka"]);
    assert!(
        result.is_empty(),
        "variable topic must not produce a RawEventTopic"
    );
}

/// No kafka import → import gate blocks all captures.
#[test]
fn test_no_kafka_import_no_captures() {
    let src = r#"
import json

def publish():
    result = json.dumps({"key": "value"})
"#;
    let (result, _pool) = run(src, &["json"]);
    assert!(result.is_empty(), "non-kafka import must produce nothing");
}

/// aiokafka async variant: `await producer.send("payments", data)` inside a function.
#[test]
fn test_aiokafka_producer_send_literal() {
    let src = r#"
from aiokafka import AIOKafkaProducer

async def send_payment(payload):
    producer = AIOKafkaProducer(bootstrap_servers="localhost:9092")
    await producer.send("payments", payload)
"#;
    let (result, pool) = run(src, &["aiokafka"]);
    assert_eq!(result.len(), 1, "expected one RawEventTopic from aiokafka");
    assert_eq!(result[0].lib, FrameworkId::Kafka);
    assert_eq!(result[0].direction, PubSub::Publish);
    let lit = result[0].topic_literal.expect("topic_literal must be Some");
    assert_eq!(pool.resolve(&lit), "payments");
}

/// confluent_kafka: `producer.produce("events", value)` inside a function.
#[test]
fn test_confluent_kafka_produce_literal() {
    let src = r#"
from confluent_kafka import Producer

def emit_event(data):
    p = Producer({"bootstrap.servers": "localhost"})
    p.produce("events", data.encode())
"#;
    let (result, pool) = run(src, &["confluent_kafka"]);
    assert_eq!(
        result.len(),
        1,
        "expected one RawEventTopic from confluent_kafka"
    );
    assert_eq!(result[0].lib, FrameworkId::Kafka);
    let lit = result[0].topic_literal.expect("topic_literal must be Some");
    assert_eq!(pool.resolve(&lit), "events");
}
