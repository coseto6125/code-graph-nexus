# Anonymous-callback call edges — design

## Problem

Calls made *inside an anonymous function passed as a call argument* (callback
registration) are silently dropped from the graph when no **named** enclosing
function/method/constructor contains them.

Reproduced on `army_project/static/js/app.js`:

```js
document.getElementById('modalCloseBtn')
  .addEventListener('click', () => guardedClose('itemsModal', closeItemsModal));
```

`ecp impact guardedClose` → `0 incoming references`.
`MATCH (caller)-[:CALLS]->(n {name:'guardedClose'})` → 0 rows.

### Root cause (verified)

1. `<lang>/queries.scm` only creates a Function node for an anonymous
   function when it is **bound to a name** (`variable_declarator`, object
   `pair`). An arrow/closure in argument position gets **no node**.
   (JS: `javascript/queries.scm:17-19`, `79-81`.)
2. `calls.rs::attach_to_enclosing` (shared across all languages) attaches each
   extracted call to the *smallest enclosing* `Function | Method | Constructor`
   by span containment, and **silently drops** the call when none contains the
   line — `if let Some(i) = best { … }` has no `else` (`calls.rs:123`).

So a call inside a **top-level** anonymous callback (no named enclosing scope)
is extracted, named correctly, then dropped at attach time.

### Trigger condition (important nuance)

The drop only happens when the anonymous function has **no named
function/method/constructor ancestor**. An anonymous callback *inside* a named
function attributes its inner calls to that named function (coarser, but not
dropped). The reproduction is a clean 0 only because every `guardedClose` call
site is a module-top-level listener.

### LLM-utility justification (CLAUDE.md filter)

Passes **filter (A) — graph completeness**: callback registration is named
explicitly. Without it `ecp impact` lies about callers, so an LLM refactor of
`guardedClose` misses every DOM-wiring call site.

## Approach (chosen)

Create a lightweight **`<anonymous>` Function node** for any anonymous
function/closure in argument (or trailing-block) position **whose body
contains a call**. The existing `attach_to_enclosing` span logic then attaches
inner calls automatically.

**`calls.rs` is not modified** — node creation alone makes the existing attach
logic work. This keeps the change per-language and low-risk.

### Common rules (all 14 languages)

1. **Only emit when the body contains a call_expression** — empty callbacks
   (`arr.map(x => x * 2)`) add no node, bounding graph bloat to nodes that
   actually host call edges.
2. Node: `name = "<anonymous>"`, `kind = Function`, `span = closure body`.
   Reuses the existing `<anonymous>` convention (Express handler path,
   `javascript/parser.rs:293`).
3. **Do not touch `attach_to_enclosing`.**
4. One parity test per language:
   `crates/ecp-analyzer/tests/<lang>_anonymous_callbacks.rs`, asserting a call
   inside an anonymous callback is attached to an `<anonymous>` Function node.

### Per-language node kinds & query shape

Exact node kinds are verified against each grammar during implementation; the
table is the starting hypothesis.

**Class 1 — argument-embedded** (closure is a child of the call's argument list):
query `(<arg-list> <closure-kind> @function.anonymous)`

| Lang  | Anonymous node kind(s)                                         |
|-------|----------------------------------------------------------------|
| JS    | `arrow_function`, `function_expression`                        |
| TS    | `arrow_function`, `function_expression`                        |
| Python| `lambda` (single-expr body, may contain a call)                |
| Java  | `lambda_expression`, anonymous `object_creation_expression`    |
| C#    | `lambda_expression`, `anonymous_method_expression`             |
| Go    | `func_literal`                                                 |
| Rust  | `closure_expression`                                           |
| PHP   | `anonymous_function`, `arrow_function`                         |
| C++   | `lambda_expression`                                            |
| Dart  | `function_expression`                                          |

**Class 2 — trailing-block** (closure attaches to the call, outside arg parens):
query matches the block as the call's sibling, not inside arguments.

| Lang   | Construct                                                      |
|--------|----------------------------------------------------------------|
| Kotlin | trailing lambda `annotated_lambda` / `lambda_literal`          |
| Swift  | trailing closure                                               |
| Ruby   | `do_block` / `block` attached to a method call                 |

**Class 3 — no anonymous-callable construct**

| Lang | Handling                                                          |
|------|-------------------------------------------------------------------|
| C    | Standard C has no closures. Test asserts the *reverse*: a named function-pointer callback (`register(&handler)`) is unaffected; nothing to drop. |

## Reference implementation

JS is implemented first and verified end-to-end against the `army_project`
reproduction (`ecp impact guardedClose` must list a `<anonymous>` caller at
`static/js/app.js` line of the listener). TS mirrors JS. The remaining
languages follow the same pattern via parallel sub-agents, each using the JS
implementation as the template.

## Out of scope

- DOM element → handler semantic edges (`'modalCloseBtn'` button → handler):
  element ids are string literals, not modeled by the call graph.
- Treating a function *passed by reference* (`f(callback)` where `callback` is a
  named identifier) as a call — that is a reference, not a call.
- Changing `attach_to_enclosing` drop behavior for non-anonymous cases.

## Testing

- Per-language `crates/ecp-analyzer/tests/<lang>_anonymous_callbacks.rs`.
- Build: `cargo build -p egent-code-plexus --bin ecp --release`.
- Parser tests: `cargo test -p ecp-analyzer`.
- End-to-end: reindex `army_project`, confirm `ecp impact guardedClose` lists
  the `<anonymous>` caller(s).
