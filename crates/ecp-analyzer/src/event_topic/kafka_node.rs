//! `EventTopicConfig` for Kafka in the Node.js ecosystem (TypeScript + JavaScript).
//!
//! Covers two libraries shared by both languages:
//! - `kafkajs`: producer-side `producer.send({ topic: '...', messages: [...] })`
//! - `node-rdkafka`: producer-side `producer.produce('topic-name', ...)`
//!
//! Producer-only in this PR (`classify_kafka_direction` returns `PubSub::Publish`
//! unconditionally). Subscribe-side capture is followup-tracked.
//!
//! Import gate is identical for TS and JS — `kafkajs` and `node-rdkafka` are JS
//! packages; TypeScript only adds type declarations on top, so the runtime
//! contract that determines whether a file emits Kafka events is the same.
//!
//! Tree-sitter capture names: `kafka.topic`, `kafka.producer_fn`. Per-grammar
//! capture patterns live in `typescript/frameworks.scm` and
//! `javascript/frameworks.scm` — they differ in node kinds (e.g. `await_expression`
//! placement) but produce captures under these same names.

use super::config::EventTopicConfig;
use super::extract::classify_kafka_direction;
use ecp_core::analyzer::types::FrameworkId;

/// Kafka Node.js detector — fires for `kafkajs` and `node-rdkafka` imports.
///
/// Shared by `TypeScriptProvider` (T5-3) and `JavaScriptProvider` (T5-4): the
/// libraries are JS packages identically consumed from both languages, so a
/// single config drives both parsers. Divergence (e.g. NestJS `@MessagePattern`
/// decorator wrappers, TS-only) would warrant a separate config; until then
/// duplicating the const buys no signal and costs maintenance.
///
/// LLM-utility: surfaces producer call sites so `ecp impact` can trace which
/// functions publish to a given topic — enabling cross-service blast-radius
/// queries without manual grep across repo boundaries.
pub const KAFKA_NODE: EventTopicConfig = EventTopicConfig {
    framework: FrameworkId::Kafka,
    topic_capture: "kafka.topic",
    producer_capture: "kafka.producer_fn",
    direction_capture: "",
    import_gate: &["kafkajs", "node-rdkafka"],
    direction_classifier: classify_kafka_direction,
    canonicalize: true,
};
