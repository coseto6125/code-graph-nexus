//! `EventTopicConfig` for Kafka JavaScript clients.
//!
//! Covers two JS Kafka libraries under a single config; all are
//! producer-only in our current model (`classify_kafka_direction` returns
//! `PubSub::Publish` unconditionally).
//!
//! Import gate: `kafkajs`, `node-rdkafka`.
//! Tree-sitter capture names: `kafka.topic`, `kafka.producer_fn`.

use super::config::EventTopicConfig;
use super::extract::classify_kafka_direction;
use ecp_core::analyzer::types::FrameworkId;

/// Kafka JavaScript detector — fires for `kafkajs` and `node-rdkafka` imports.
///
/// LLM-utility: surfaces producer call sites so `ecp impact` can trace which
/// JavaScript functions publish to a given topic — enabling cross-service
/// blast-radius queries without manual grep across repo boundaries.
pub const KAFKA_JS: EventTopicConfig = EventTopicConfig {
    framework: FrameworkId::Kafka,
    topic_capture: "kafka.topic",
    producer_capture: "kafka.producer_fn",
    direction_capture: "",
    import_gate: &["kafkajs", "node-rdkafka"],
    direction_classifier: classify_kafka_direction,
    canonicalize: true,
};
