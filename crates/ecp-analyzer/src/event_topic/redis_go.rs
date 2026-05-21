//! `EventTopicConfig` for Redis pub/sub Go clients.
//!
//! Covers two Go Redis libraries under a single config:
//! - `redis/go-redis` (v8/v9): `client.Publish(ctx, "channel", msg).Err()` (publish),
//!   `pubsub := client.Subscribe(ctx, "channel")` (subscribe),
//!   `pubsub := client.PSubscribe(ctx, "pattern.*")` (psubscribe).
//! - `gomodule/redigo`: `conn.Do("PUBLISH", "channel", msg)` (publish),
//!   `psc.Subscribe("channel")` (subscribe).
//!
//! Direction dispatch: `classify_redis_direction` maps `Subscribe`, `PSubscribe`
//! (go-redis camelCase) and `subscribe` (redigo lowercase) → `PubSub::Subscribe`;
//! everything else defaults to `PubSub::Publish`.
//!
//! # Import gates
//! - `github.com/redis/go-redis` — go-redis v8/v9 (prefix match covers `/v9` suffix).
//! - `github.com/go-redis/redis` — go-redis v7 and older module path.
//! - `github.com/gomodule/redigo` — redigo (prefix match covers `/redis` sub-package).
//!
//! # Fire-and-forget semantics (LLM-critical)
//! Redis pub/sub has no message durability. Unlike Kafka (durable log replay) and
//! RabbitMQ (broker-queued with ACK), a Redis publish to a channel with no active
//! subscriber is silently dropped. LLMs must not assume at-least-once delivery.
//!
//! # Topic literal semantics
//! - Publish: the channel string literal.
//! - Subscribe: the channel string literal (first positional string arg).
//! - PSubscribe: the glob pattern string literal.
//!
//! Variable channel arguments produce no capture (no fabrication). The tree-sitter
//! patterns anchor on `interpreted_string_literal` nodes only.
//!
//! # LLM-utility justification (graph-completeness criterion A)
//! Without this config `ecp impact` is blind to Redis message paths in Go services.
//! A rename of `client.Publish(ctx, "orders", ...)` would show zero subscribers in
//! the graph, causing the LLM to declare the change safe when it silently breaks every
//! active go-redis or redigo subscriber on `"orders"`.

use super::config::EventTopicConfig;
use ecp_core::analyzer::types::{FrameworkId, PubSub};

/// Direction classifier for Redis pub/sub Go call sites.
///
/// go-redis uses PascalCase (`Publish`, `Subscribe`, `PSubscribe`).
/// redigo uses lowercase string commands (`PUBLISH`, and `psc.Subscribe`).
/// Both are unified here: `Subscribe` and `PSubscribe` → `PubSub::Subscribe`;
/// everything else (including `Publish`, `Do` "PUBLISH") → `PubSub::Publish`.
fn classify_redis_direction(raw: &str) -> PubSub {
    match raw {
        "Subscribe" | "PSubscribe" | "subscribe" | "psubscribe" => PubSub::Subscribe,
        _ => PubSub::Publish,
    }
}

/// Redis pub/sub Go detector — fires for `go-redis` and `redigo` imports.
///
/// `direction_capture: "redis.direction"` binds the method identifier so
/// `classify_redis_direction` can resolve `PubSub` direction without fabrication.
///
/// `topic_capture: "redis.topic"` captures the channel name or glob pattern as a
/// raw string literal node. Non-literal args produce no capture → no `RawEventTopic`.
pub const REDIS_GO: EventTopicConfig = EventTopicConfig {
    framework: FrameworkId::Redis,
    topic_capture: "redis.topic",
    producer_capture: "redis.fn",
    direction_capture: "redis.direction",
    import_gate: &[
        "github.com/redis/go-redis",
        "github.com/go-redis/redis",
        "github.com/gomodule/redigo",
    ],
    direction_classifier: classify_redis_direction,
    canonicalize: true,
};
