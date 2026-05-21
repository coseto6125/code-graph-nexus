//! T5-5 integration tests: Kafka Java event-topic detector.
//!
//! Exercises the production `KAFKA_JAVA` const and the real `frameworks.scm`
//! query string against `org.apache.kafka` (KafkaProducer / KafkaConsumer)
//! and `org.springframework.kafka` (KafkaTemplate) patterns.

use ecp_analyzer::event_topic::{extract_event_topics, KAFKA_JAVA};
use ecp_core::analyzer::types::{FrameworkId, PubSub, RawImport};
use ecp_core::pool::StringPool;
use tree_sitter::{Parser, Query};

const QUERIES_SCM: &str = include_str!("../src/java/queries.scm");
const FRAMEWORKS_SCM: &str = include_str!("../src/java/frameworks.scm");

fn run(
    src: &str,
    import_sources: &[&str],
) -> (Vec<ecp_core::analyzer::types::RawEventTopic>, StringPool) {
    let lang: tree_sitter::Language = tree_sitter_java::LANGUAGE.into();
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language");
    let tree = parser.parse(src.as_bytes(), None).expect("parse");
    let combined = format!(
        "{}\n;; ---- framework queries ----\n{}",
        QUERIES_SCM, FRAMEWORKS_SCM
    );
    let query = Query::new(&lang, &combined).expect("query compile");
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
        &[KAFKA_JAVA],
        &imports,
        &mut pool,
    );
    (result, pool)
}

/// Apache Kafka: producer.send(new ProducerRecord<>("topic", ...)) → Publish.
#[test]
fn test_java_kafka_producer_send_producerrecord_literal_topic() {
    let src = r#"
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerRecord;

public class OrderService {
    public void publishOrder(KafkaProducer<String, String> producer, String data) {
        producer.send(new ProducerRecord<>("orders", data));
    }
}
"#;
    let (result, pool) = run(src, &["org.apache.kafka"]);
    assert_eq!(
        result.len(),
        1,
        "expected one RawEventTopic from ProducerRecord; got {:?}",
        result
    );
    assert_eq!(result[0].lib, FrameworkId::Kafka);
    assert_eq!(result[0].direction, PubSub::Publish);
    let lit = result[0].topic_literal.expect("topic_literal must be Some");
    assert_eq!(pool.resolve(&lit), "orders");
}

/// Spring Kafka: kafkaTemplate.send("topic", msg) → Publish.
#[test]
fn test_java_spring_kafka_template_send_literal_topic() {
    let src = r#"
import org.springframework.kafka.core.KafkaTemplate;

public class PaymentService {
    private KafkaTemplate<String, String> kafkaTemplate;

    public void publishPayment(String msg) {
        kafkaTemplate.send("payments", msg);
    }
}
"#;
    let (result, pool) = run(src, &["org.springframework.kafka"]);
    assert_eq!(
        result.len(),
        1,
        "expected one RawEventTopic from KafkaTemplate.send; got {:?}",
        result
    );
    assert_eq!(result[0].lib, FrameworkId::Kafka);
    assert_eq!(result[0].direction, PubSub::Publish);
    let lit = result[0].topic_literal.expect("topic_literal must be Some");
    assert_eq!(pool.resolve(&lit), "payments");
}

/// Apache Kafka consumer subscribe via Arrays.asList → Subscribe direction.
#[test]
fn test_java_kafka_consumer_subscribe_literal_topic() {
    let src = r#"
import org.apache.kafka.clients.consumer.KafkaConsumer;
import java.util.Arrays;

public class OrderListener {
    public void listen(KafkaConsumer<String, String> consumer) {
        consumer.subscribe(Arrays.asList("orders"));
    }
}
"#;
    let (result, pool) = run(src, &["org.apache.kafka"]);
    assert_eq!(
        result.len(),
        1,
        "expected one RawEventTopic from subscribe; got {:?}",
        result
    );
    assert_eq!(result[0].lib, FrameworkId::Kafka);
    assert_eq!(result[0].direction, PubSub::Subscribe);
    let lit = result[0].topic_literal.expect("topic_literal must be Some");
    assert_eq!(pool.resolve(&lit), "orders");
}

/// Variable topic argument → no capture (no fabrication).
#[test]
fn test_java_variable_topic_emits_nothing() {
    let src = r#"
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerRecord;

public class DynamicService {
    public void sendDynamic(KafkaProducer<String, String> producer, String topicName, String data) {
        producer.send(new ProducerRecord<>(topicName, data));
    }
}
"#;
    let (result, _pool) = run(src, &["org.apache.kafka"]);
    assert!(
        result.is_empty(),
        "variable topic must produce no RawEventTopic; got {:?}",
        result
    );
}

/// No Kafka import → import gate must reject all captures.
#[test]
fn test_java_no_kafka_import_emits_nothing() {
    let src = r#"
import java.util.logging.Logger;

public class LogService {
    public void sendMessage(Logger logger, String msg) {
        logger.send("events", msg);
    }
}
"#;
    let (result, _pool) = run(src, &["java.util.logging"]);
    assert!(
        result.is_empty(),
        "non-kafka import must produce nothing; got {:?}",
        result
    );
}

/// Spring Kafka: both apache and spring libraries fire correctly in same slice.
#[test]
fn test_java_spring_and_apache_kafka_both_fire() {
    let src = r#"
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerRecord;
import org.springframework.kafka.core.KafkaTemplate;

public class MultiKafkaService {
    public void sendApache(KafkaProducer<String, String> producer, String data) {
        producer.send(new ProducerRecord<>("events", data));
    }

    public void sendSpring(KafkaTemplate<String, String> template, String msg) {
        template.send("billing", msg);
    }
}
"#;
    let (result, pool) = run(src, &["org.apache.kafka", "org.springframework.kafka"]);
    let topics: Vec<&str> = result
        .iter()
        .map(|r| pool.resolve(r.topic_literal.as_ref().unwrap()))
        .collect();
    assert!(topics.contains(&"events"), "apache kafka topic must appear");
    assert!(
        topics.contains(&"billing"),
        "spring kafka topic must appear"
    );
}

/// Enclosing function name is captured correctly.
#[test]
fn test_java_enclosing_fn_name_captured() {
    let src = r#"
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerRecord;

public class UserService {
    public void publishUserEvent(KafkaProducer<String, String> producer, String data) {
        producer.send(new ProducerRecord<>("users", data));
    }
}
"#;
    let (result, pool) = run(src, &["org.apache.kafka"]);
    assert_eq!(result.len(), 1);
    let fn_name = pool.resolve(&result[0].enclosing_fn);
    assert_eq!(fn_name, "publishUserEvent");
}
