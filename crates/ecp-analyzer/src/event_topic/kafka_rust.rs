//! `EventTopicConfig` for Kafka Rust clients (T5-7).
//!
//! Covers the `rdkafka` crate (the most widely used Rust Kafka client):
//! - Producer: `producer.send(FutureRecord::to("topic"), ...)` — captures the
//!   string literal passed to `FutureRecord::to(...)`.
//!
//! Direction dispatch: `classify_kafka_rust_direction` returns `PubSub::Publish`
//! unconditionally for producers. Subscribe-side (StreamConsumer / BaseConsumer
//! `subscribe(&["topic"])`) is captured as well — mapping to `PubSub::Subscribe`.
//!
//! This classifier is intentionally module-private. Parallel-PR isolation
//! prevents 3-way merge conflicts; followup can consolidate once all lang PRs land.
//!
//! # Topic literal semantics
//! - rdkafka producer: the string literal in `FutureRecord::to("topic")`.
//! - rdkafka consumer: the string literal in `consumer.subscribe(&["topic"])`.
//! - Variable topic arguments → no capture → no `RawEventTopic` emitted
//!   (no fabrication).
//!
//! # Schema gap (deferred)
//! `RawEventTopic` has no `kind` field — see `redis_python.rs` for the
//! schema gap note that applies equally here.
//!
//! # LLM-utility justification (graph-completeness criterion A)
//! Without this config, `ecp impact` is blind to Rust Kafka message paths.
//! A rename of a `FutureRecord::to("orders")` call site would show zero
//! consumers, causing the LLM to declare the change safe when it silently
//! breaks every consumer listening on `"orders"`.

use super::config::EventTopicConfig;
use ecp_core::analyzer::types::{FrameworkId, PubSub};

/// Direction classifier for Rust rdkafka call sites.
///
/// `subscribe` is the consumer-side method on `StreamConsumer`/`BaseConsumer`;
/// everything else (i.e. `send`, `send_result`) is treated as Publish.
fn classify_kafka_rust_direction(raw: &str) -> PubSub {
    match raw {
        "subscribe" => PubSub::Subscribe,
        _ => PubSub::Publish,
    }
}

/// Kafka Rust detector — fires for `rdkafka` imports.
///
/// `direction_capture: "kafka.rust.direction"` binds the method identifier so
/// `classify_kafka_rust_direction` can resolve `PubSub` direction without fabrication.
///
/// `topic_capture: "kafka.topic"` captures the topic name as a raw string
/// literal node. Non-literal topic values produce no capture (no fabrication).
pub const KAFKA_RUST: EventTopicConfig = EventTopicConfig {
    framework: FrameworkId::Kafka,
    topic_capture: "kafka.topic",
    producer_capture: "kafka.rust.fn",
    direction_capture: "kafka.rust.direction",
    import_gate: &["rdkafka"],
    direction_classifier: classify_kafka_rust_direction,
    canonicalize: true,
};
