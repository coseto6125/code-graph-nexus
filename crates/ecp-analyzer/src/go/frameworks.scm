;; Framework-aware queries for Go (Redis pub/sub).

;; ---- Redis Go pub/sub (T5-30) ----
;; Covers go-redis (v8/v9) and gomodule/redigo under one config slice.
;; Import gate (github.com/redis/go-redis / github.com/go-redis/redis /
;; github.com/gomodule/redigo) is enforced by REDIS_GO.import_gate — these
;; queries fire on syntax alone; the extractor filters by import at runtime.
;;
;; `redis.direction` captures the method name so `classify_redis_direction`
;; can distinguish Subscribe from Publish.
;;
;; go-redis uses PascalCase: `Publish`, `Subscribe`, `PSubscribe`.
;; redigo uses lowercase via psc object: `subscribe` (via PubSubConn).
;;
;; Topic literal: the second positional interpreted_string_literal arg (go-redis
;; calls pass ctx as the first arg: Publish(ctx, "channel", msg)).
;; For redigo Subscribe/PSubscribe the channel is the first positional arg.
;;
;; Variable channel args produce no `redis.topic` capture → no RawEventTopic.
;;
;; Anchored to `function_declaration` to co-capture the enclosing function name.
;; The Go block has the shape: block → statement_list → expression_statement →
;; call_expression. We use `(block (statement_list (expression_statement ...)))`
;; to traverse the exact path without wildcard ambiguity.

;; go-redis: client.Publish(ctx, "channel", msg) inside a function — Publish.
;; Channel is the second positional arg (after ctx).
(function_declaration
  name: (identifier) @redis.fn
  body: (block
    (statement_list
      (expression_statement
        (call_expression
          function: (selector_expression
            field: (field_identifier) @redis.direction (#eq? @redis.direction "Publish"))
          arguments: (argument_list
            _
            (interpreted_string_literal) @redis.topic))))))

;; go-redis: client.Subscribe(ctx, "channel") inside a function — Subscribe.
;; Channel is the second positional arg (after ctx).
(function_declaration
  name: (identifier) @redis.fn
  body: (block
    (statement_list
      (expression_statement
        (call_expression
          function: (selector_expression
            field: (field_identifier) @redis.direction (#eq? @redis.direction "Subscribe"))
          arguments: (argument_list
            _
            (interpreted_string_literal) @redis.topic))))))

;; go-redis: client.PSubscribe(ctx, "pattern.*") inside a function — Subscribe (pattern).
;; Pattern is the second positional arg (after ctx).
(function_declaration
  name: (identifier) @redis.fn
  body: (block
    (statement_list
      (expression_statement
        (call_expression
          function: (selector_expression
            field: (field_identifier) @redis.direction (#eq? @redis.direction "PSubscribe"))
          arguments: (argument_list
            _
            (interpreted_string_literal) @redis.topic))))))


;; redigo: psc.Subscribe("channel") inside a function — Subscribe.
;; Channel is the first positional arg (no ctx).
(function_declaration
  name: (identifier) @redis.fn
  body: (block
    (statement_list
      (expression_statement
        (call_expression
          function: (selector_expression
            field: (field_identifier) @redis.direction (#eq? @redis.direction "subscribe"))
          arguments: (argument_list
            . (interpreted_string_literal) @redis.topic))))))

;; go-redis: short-var: pubsub := client.Subscribe(ctx, "channel") — Subscribe.
(function_declaration
  name: (identifier) @redis.fn
  body: (block
    (statement_list
      (short_var_declaration
        right: (expression_list
          (call_expression
            function: (selector_expression
              field: (field_identifier) @redis.direction (#eq? @redis.direction "Subscribe"))
            arguments: (argument_list
              _
              (interpreted_string_literal) @redis.topic)))))))

;; go-redis: short-var: pubsub := client.PSubscribe(ctx, "pattern.*") — Subscribe.
(function_declaration
  name: (identifier) @redis.fn
  body: (block
    (statement_list
      (short_var_declaration
        right: (expression_list
          (call_expression
            function: (selector_expression
              field: (field_identifier) @redis.direction (#eq? @redis.direction "PSubscribe"))
            arguments: (argument_list
              _
              (interpreted_string_literal) @redis.topic)))))))
