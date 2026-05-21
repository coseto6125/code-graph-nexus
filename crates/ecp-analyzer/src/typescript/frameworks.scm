;; Framework-aware queries for TypeScript (Tier 1: Express subset).

;; Express: app.{get,post,put,delete,patch,use}(<path_str>, <handler_ident>)
;; Captures the handler identifier passed as second argument.
(call_expression
  function: (member_expression
    object: (identifier)
    property: (property_identifier) @express.route.method
    (#match? @express.route.method "^(get|post|put|delete|patch|use)$"))
  arguments: (arguments
    [(string) @express.route.path (MISSING) @express.route.path]
    (identifier) @express.route.handler))

;; NestJS: @Controller-decorated class with @Get/@Post/@Put/@Delete/@Patch
;; method-level decorators. Two forms — class is exported via `export class`
;; (decorator moves to export_statement) or declared directly (decorator stays
;; on class_declaration).

;; Form 1: non-exported @Controller class.
(class_declaration
  (decorator
    (call_expression
      function: (identifier) @nestjs.controller.kw
      (#eq? @nestjs.controller.kw "Controller")))
  name: (type_identifier) @nestjs.controller.class
  body: (class_body
    (decorator
      (call_expression
        function: (identifier) @nestjs.method.verb
        (#match? @nestjs.method.verb "^(Get|Post|Put|Delete|Patch)$")))
    .
    (method_definition
      name: (property_identifier) @nestjs.method.name)))

;; Form 2: exported @Controller class — decorator sits on export_statement.
(export_statement
  (decorator
    (call_expression
      function: (identifier) @nestjs.controller.kw
      (#eq? @nestjs.controller.kw "Controller")))
  declaration: (class_declaration
    name: (type_identifier) @nestjs.controller.class
    body: (class_body
      (decorator
        (call_expression
          function: (identifier) @nestjs.method.verb
          (#match? @nestjs.method.verb "^(Get|Post|Put|Delete|Patch)$")))
      .
      (method_definition
        name: (property_identifier) @nestjs.method.name))))

;; ---- TypeScript interface SchemaField (T4-4) ----
;; Captures typed property signatures on `interface X { ... }` bodies.
;; Each property_signature with a predefined_type annotation becomes one
;; RawSchemaField via the T4-1 dispatcher (TS_INTERFACE_CONFIG).
;; `predefined_type` covers: string, number, boolean, any, void, never, object,
;; symbol, bigint, undefined, null.  Union (`string | null`) and array
;; (`string[]`) are `union_type` / `array_type` — they don't match this
;; pattern and fall through to SchemaType::Other via classify_ts_type("").
;; No import gate needed: `interface` is a TS language built-in.
(interface_declaration
  name: (type_identifier) @ts.owner
  body: (interface_body
    (property_signature
      name: (property_identifier) @ts.field
      type: (type_annotation
        (predefined_type) @ts.type))))

;; NestJS / generic decorator-route: `@Get('users')` / `@Post('users/:id')` /
;; `@Put('audio/transcode')`. Captures the decorator verb AND the bare path
;; argument. Independent of `@Controller` context — gated in parser.rs by
;; `has_nestjs` (only imports of `@nestjs/*` flip the flag), so user-defined
;; `@Get(...)` decorators in non-NestJS code don't surface false routes.
;;
;; Verb list mirrors NestJS's HTTP routing decorators (omits `@All` which
;; tree-sitter captures via its own grammar path and routes to the generic
;; `app.METHOD()` matcher above).
(decorator
  (call_expression
    function: (identifier) @nestjs.decorator.verb
    (#match? @nestjs.decorator.verb "^(Get|Post|Put|Delete|Patch|Options|Head|All)$")
    arguments: (arguments
      [(string (string_fragment) @nestjs.decorator.path) (MISSING) @nestjs.decorator.path])))

;; ---- Redis TypeScript (T5-27) ----
;; Covers node-redis v4 (`client.publish/subscribe/pSubscribe(...)`) and
;; ioredis (`redis.publish/subscribe/psubscribe(...)`).
;; Import gate (`redis`, `ioredis`) is enforced by REDIS_TS.import_gate —
;; these queries fire on syntax alone; the extractor filters by import at runtime.
;;
;; `redis.direction` captures the method name so `classify_redis_direction`
;; can distinguish Subscribe from Publish.  node-redis v4 spells it `pSubscribe`
;; (camelCase); ioredis spells it `psubscribe` (lowercase) — two separate
;; `#eq?` predicates cover both forms without regex.
;;
;; Anchored to `function_declaration` and `method_definition`; sync and
;; await forms handled separately (mirrors T5-3 Kafka TS approach).
;;
;; Channel must be the first positional string literal arg (`. (string)`);
;; variable channels emit nothing — no fabrication.

;; Redis: `client.publish/subscribe/pSubscribe/psubscribe('<channel>', ...)` inside function_declaration (sync).
(function_declaration
  name: (identifier) @redis.fn
  body: (statement_block
    (_
      (call_expression
        function: (member_expression
          property: (property_identifier) @redis.direction
          (#match? @redis.direction "^(publish|subscribe|pSubscribe|psubscribe)$"))
        arguments: (arguments
          . (string) @redis.topic)))))

;; Redis: `await client.publish/subscribe/pSubscribe/psubscribe('<channel>', ...)` inside async function_declaration.
(function_declaration
  name: (identifier) @redis.fn
  body: (statement_block
    (_
      (await_expression
        (call_expression
          function: (member_expression
            property: (property_identifier) @redis.direction
            (#match? @redis.direction "^(publish|subscribe|pSubscribe|psubscribe)$"))
          arguments: (arguments
            . (string) @redis.topic))))))

;; Redis: `client.publish/subscribe/pSubscribe/psubscribe('<channel>', ...)` inside method_definition (sync).
(method_definition
  name: (property_identifier) @redis.fn
  body: (statement_block
    (_
      (call_expression
        function: (member_expression
          property: (property_identifier) @redis.direction
          (#match? @redis.direction "^(publish|subscribe|pSubscribe|psubscribe)$"))
        arguments: (arguments
          . (string) @redis.topic)))))

;; Redis: `await client.publish/subscribe/pSubscribe/psubscribe('<channel>', ...)` inside async method_definition.
(method_definition
  name: (property_identifier) @redis.fn
  body: (statement_block
    (_
      (await_expression
        (call_expression
          function: (member_expression
            property: (property_identifier) @redis.direction
            (#match? @redis.direction "^(publish|subscribe|pSubscribe|psubscribe)$"))
          arguments: (arguments
            . (string) @redis.topic))))))
