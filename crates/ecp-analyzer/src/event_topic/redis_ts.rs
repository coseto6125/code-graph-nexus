//! `EventTopicConfig` for Redis pub/sub TypeScript clients.
//!
//! Covers two TypeScript Redis libraries under a single config:
//! - `redis` (node-redis v4): `await client.publish('channel', msg)`,
//!   `await client.subscribe('channel', handler)`,
//!   `await client.pSubscribe('pattern.*', handler)` (camelCase in node-redis v4)
//! - `ioredis`: `redis.publish('channel', msg)`,
//!   `redis.subscribe('channel')`,
//!   `redis.psubscribe('pattern.*')` (lowercase in ioredis)
//!
//! LLM-utility: Redis pub/sub is fire-and-forget with no message durability —
//! missed messages are lost. An LLM refactoring a Redis channel must know it
//! cannot assume at-least-once delivery or consumer-group replay semantics
//! (unlike Kafka). Exposing publish/subscribe call sites in the graph lets
//! `ecp impact` trace channel consumers without grep, preventing silent data-
//! loss bugs when channels are renamed or removed.
//!
//! Import gate: `redis`, `ioredis`.
//! Tree-sitter capture names: `redis.topic`, `redis.fn`, `redis.direction`.
//!
//! Schema stability note: `FrameworkId::Redis` (discriminant 16) was appended
//! at the end of the enum in this PR. Sibling agents T5-26 (Python) and T5-28
//! (JS) may also append it — first-to-merge wins; second sibling rebases and
//! drops the duplicate variant addition.

use super::config::EventTopicConfig;
use ecp_core::analyzer::types::{FrameworkId, PubSub};

/// Direction classifier for Redis pub/sub call sites.
///
/// node-redis v4 uses camelCase `pSubscribe`; ioredis uses lowercase
/// `psubscribe`. Both are mapped to `Subscribe` here so one classifier
/// covers both libraries without a second config.
///
/// Unrecognised text (e.g. `publish`) defaults to `Publish`.
fn classify_redis_direction(raw: &str) -> PubSub {
    match raw {
        "subscribe" | "psubscribe" | "pSubscribe" => PubSub::Subscribe,
        _ => PubSub::Publish,
    }
}

/// Redis TypeScript detector — fires for `redis` (node-redis) and `ioredis` imports.
///
/// LLM-utility: surfaces both publish and subscribe call sites so `ecp impact`
/// can answer "which services listen to channel X?" and "which functions emit
/// to channel Y?" — critical for fire-and-forget channels where a missing
/// subscriber means silent data loss.
///
/// Redis pub/sub carries no durability guarantee (no WAL, no consumer groups);
/// this is semantically distinct from Kafka topics and AMQP queues/exchanges,
/// so a separate `FrameworkId::Redis` variant is warranted to prevent LLM
/// confusion about delivery guarantees during refactors.
pub const REDIS_TS: EventTopicConfig = EventTopicConfig {
    framework: FrameworkId::Redis,
    topic_capture: "redis.topic",
    producer_capture: "redis.fn",
    direction_capture: "redis.direction",
    import_gate: &["redis", "ioredis"],
    direction_classifier: classify_redis_direction,
    canonicalize: true,
};
