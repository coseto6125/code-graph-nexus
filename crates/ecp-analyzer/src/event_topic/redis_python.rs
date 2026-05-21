//! `EventTopicConfig` for Redis pub/sub Python clients.
//!
//! Covers two Python Redis libraries under a single config:
//! - `redis` (sync): `r.publish("channel", msg)` / `pubsub.subscribe("ch")` /
//!   `pubsub.psubscribe("pattern.*")`
//! - `aioredis` (async): `await r.publish("channel", msg)` /
//!   `await pubsub.subscribe("channel")` / `await pubsub.psubscribe("pattern")`
//!
//! Direction dispatch: `classify_redis_direction` (module-private) maps the
//! captured method name to `PubSub::Subscribe` for `subscribe` and `psubscribe`,
//! and `PubSub::Publish` for everything else (default: `publish`).
//!
//! This classifier is intentionally module-private rather than added to
//! `event_topic/extract.rs`. T5-27 (TypeScript) and T5-28 (JavaScript) Redis
//! detectors run in parallel on sibling agents; a shared classifier in extract.rs
//! would create a 3-way merge conflict. After all three PRs land, a followup can
//! consolidate if the classifiers are identical. Today: isolation > DRY.
//!
//! # Topic literal semantics
//! - Publish: the `channel` positional string literal (first arg to `publish`).
//! - Subscribe: the `channel` positional string literal (first arg to `subscribe`).
//! - Psubscribe: the `pattern` positional string literal (first arg to `psubscribe`).
//!   Pattern strings (`"orders.*"`) are glob expressions, not channel names. They
//!   are stored in `topic_literal` unchanged — see **Schema gap** below.
//!
//! Multi-arg subscribe (`pubsub.subscribe("ch1", "ch2")`) — the tree-sitter
//! query anchors on the first positional string literal only. Subsequent args
//! are not captured. Rationale: the extractor's match loop emits one
//! `RawEventTopic` per match; capturing N args would require N separate captures
//! or a post-processing pass, neither of which `EventTopicConfig` currently
//! supports. Each literal channel produces its own match row in practice when
//! written as separate subscribe calls. Documented as T5-26-followup for the
//! multi-arg form.
//!
//! # Fire-and-forget semantics (LLM-critical distinction)
//! Redis pub/sub is **fire-and-forget**: there is no persistent queue or consumer
//! group. If no subscriber is online at the moment `publish` fires, the message
//! is lost. This differs fundamentally from:
//! - Kafka: durable log; subscribers can replay from any offset.
//! - RabbitMQ: queued; broker holds messages until a consumer ACKs them.
//!
//! LLMs must NOT assume Redis channel data is retained. An `ecp impact` query
//! showing a Redis publish site with no active subscriber in the graph means the
//! message is silently dropped — not deferred. This semantic must inform any
//! refactor of publish call sites.
//!
//! # Schema gap (deferred to schema-migration PR)
//! `RawEventTopic` has no `kind` field. For Redis this loses:
//! - Whether the stored string is a plain channel name (`"orders"`) or a glob
//!   pattern (`"orders.*"` from `psubscribe`). A channel name and a glob pattern
//!   may look identical if the channel happens to contain a `.` character.
//!
//! Concrete LLM-query example that the missing field blocks:
//!   "Find all publishers that could be received by a `psubscribe('orders.*')`
//!    subscriber" — without a `kind: Option<StrRef>` field the graph cannot
//!    distinguish `topic_literal = "orders.created"` (a publish channel) from
//!    `topic_literal = "orders.*"` (a psubscribe pattern). T5-33 canonicalization
//!    receives both as bare strings; it cannot know whether to treat `"orders.*"`
//!    as a literal channel name or as a glob to expand against the channel set.
//!    The fix is `RawEventTopic { kind: Option<StrRef> }` with values
//!    `"channel"` / `"pattern"` — append-only, deferred to schema-migration PR.
//!
//! # LLM-utility justification (graph-completeness criterion A)
//! Without this config, `ecp impact` is blind to Redis message paths. A rename
//! of a `publish("orders", ...)` call site would show zero subscribers, causing
//! the LLM to declare the change safe when it silently breaks every online
//! subscriber listening on `"orders"`. Fire-and-forget semantics amplify the
//! blast radius: unlike RabbitMQ, there is no broker retry to mask the gap.

use super::config::EventTopicConfig;
use ecp_core::analyzer::types::{FrameworkId, PubSub};

/// Direction classifier for Redis pub/sub call sites.
///
/// `subscribe` and `psubscribe` are subscriber-side operations; everything
/// else (i.e. `publish`) is treated as Publish. Default-Publish matches the
/// other classifiers' convention: topic is still indexed rather than dropped
/// on unknown capture text.
fn classify_redis_direction(raw: &str) -> PubSub {
    match raw {
        "subscribe" | "psubscribe" => PubSub::Subscribe,
        _ => PubSub::Publish,
    }
}

/// Redis pub/sub Python detector — fires for `redis` and `aioredis` imports.
///
/// `direction_capture: "redis.direction"` binds the method identifier
/// (`publish`, `subscribe`, or `psubscribe`) so that `classify_redis_direction`
/// can resolve `PubSub` direction without fabrication.
///
/// `topic_capture: "redis.topic"` captures the channel name (or glob pattern
/// for `psubscribe`) as a raw string literal node. Non-literal args produce no
/// capture → no `RawEventTopic` emitted (no fabrication).
pub const REDIS_PYTHON: EventTopicConfig = EventTopicConfig {
    framework: FrameworkId::Redis,
    topic_capture: "redis.topic",
    producer_capture: "redis.fn",
    direction_capture: "redis.direction",
    import_gate: &["redis", "aioredis"],
    direction_classifier: classify_redis_direction,
    canonicalize: true,
};
