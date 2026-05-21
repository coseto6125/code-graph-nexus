;; Framework-aware queries for JavaScript.

;; ---- Kafka JavaScript (T5-4) ----
;; Covers kafkajs (`producer.send({ topic: '...', messages: [...] })`) and
;; node-rdkafka (`producer.produce('<topic>', partition, payload, ...)`).
;; Import gate (`kafkajs`, `node-rdkafka`) is enforced by KAFKA_JS.import_gate —
;; these queries fire on syntax alone; the extractor filters by import presence.
;;
;; Anchored to `function_declaration` / `method_definition` to co-capture the
;; enclosing function identifier alongside the topic literal in a single match.
;; Module-level Kafka calls are omitted — they represent <1% of production usage
;; and would produce a topic with empty enclosing_fn, offering no LLM
;; disambiguation value (same rationale as KAFKA_PYTHON).
;;
;; Two forms per anchor: sync and async/await.  The `await_expression` node wraps
;; the `call_expression` inside `expression_statement` for async calls, so it
;; requires a dedicated pattern separate from the plain sync form.  Sharing a
;; wildcard `_` for both would require an extra depth level that mismatches on
;; the sync case.

;; ── kafkajs: function_declaration ──

;; Sync: `producer.send({ topic: '<literal>', messages: [...] })`
(function_declaration
  name: (identifier) @kafka.producer_fn
  body: (statement_block
    (expression_statement
      (call_expression
        function: (member_expression
          property: (property_identifier) @_send (#eq? @_send "send"))
        arguments: (arguments
          (object
            (pair
              key: (property_identifier) @_tk (#eq? @_tk "topic")
              value: (string) @kafka.topic)))))))

;; Async: `await producer.send({ topic: '<literal>', messages: [...] })`
(function_declaration
  name: (identifier) @kafka.producer_fn
  body: (statement_block
    (expression_statement
      (await_expression
        (call_expression
          function: (member_expression
            property: (property_identifier) @_asend (#eq? @_asend "send"))
          arguments: (arguments
            (object
              (pair
                key: (property_identifier) @_atk (#eq? @_atk "topic")
                value: (string) @kafka.topic))))))))

;; ── kafkajs: method_definition ──

;; Sync: `producer.send({ topic: '<literal>', messages: [...] })`
(method_definition
  name: (property_identifier) @kafka.producer_fn
  body: (statement_block
    (expression_statement
      (call_expression
        function: (member_expression
          property: (property_identifier) @_msend (#eq? @_msend "send"))
        arguments: (arguments
          (object
            (pair
              key: (property_identifier) @_mtk (#eq? @_mtk "topic")
              value: (string) @kafka.topic)))))))

;; Async: `await producer.send({ topic: '<literal>', messages: [...] })`
(method_definition
  name: (property_identifier) @kafka.producer_fn
  body: (statement_block
    (expression_statement
      (await_expression
        (call_expression
          function: (member_expression
            property: (property_identifier) @_masend (#eq? @_masend "send"))
          arguments: (arguments
            (object
              (pair
                key: (property_identifier) @_matk (#eq? @_matk "topic")
                value: (string) @kafka.topic))))))))

;; ── node-rdkafka: function_declaration ──

;; Sync: `producer.produce('<topic>', partition, payload, ...)`
(function_declaration
  name: (identifier) @kafka.producer_fn
  body: (statement_block
    (expression_statement
      (call_expression
        function: (member_expression
          property: (property_identifier) @_produce (#eq? @_produce "produce"))
        arguments: (arguments
          . (string) @kafka.topic)))))

;; ── node-rdkafka: method_definition ──

;; Sync: `producer.produce('<topic>', ...)`
(method_definition
  name: (property_identifier) @kafka.producer_fn
  body: (statement_block
    (expression_statement
      (call_expression
        function: (member_expression
          property: (property_identifier) @_mproduce (#eq? @_mproduce "produce"))
        arguments: (arguments
          . (string) @kafka.topic)))))
