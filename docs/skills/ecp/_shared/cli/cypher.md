# ecp cypher

Execute arbitrary graph queries using a subset of the Cypher query language.

## Usage
```bash
ecp cypher "MATCH (a)-[r]->(b) RETURN a,b" [--repo <PATH>]
```

## Subset Support
- **Boolean WHERE**: `AND`, `OR`, `NOT`.
- **Comparisons**: `=`, `!=`, `<`, `<=`, `>`, `>=`.
- **String Ops**: `STARTS WITH`, `ENDS WITH`, `CONTAINS`, `=~`, `IN [...]`.
- **Aggregations**: `COUNT(*)`.
- **Pathing**: Variable-length paths `[:Rel*1..2]`, reverse arrows `<-[r]-`.

## NodeKinds
`Function`, `Method`, `Class`, `Property`, `Constructor`, `Interface`, `Const`, `Variable`, `Import`, `Route`, `Process`, `File`, `Struct`, `Enum`, `Trait`, `Impl`, `Module`, `Namespace`, `Typedef`, `Macro`, `Annotation`, `SchemaField`, `EventTopic`, `TransactionScope`, `PathLiteral`.

## RelTypes
`Calls`, `Extends`, `Imports`, `Implements`, `HasMethod`, `HasProperty`, `Accesses`, `HandlesRoute`, `References`, `Defines`, `Fetches`, `MirrorsField`, `Publishes`, `Subscribes`, `EventTopicMirror`, `OpensTxScope`, `Overrides`, `UsesPathLiteral`.

## Common patterns

### Path-literal split-brain (find filenames written one way, read another)
```cypher
MATCH (n:PathLiteral) WHERE n.name =~ ".*meta\\.json" RETURN n.name, n.file_path
```

### Who reads / writes a specific file?
```cypher
MATCH (s)-[r:USES_PATH_LITERAL]->(n:PathLiteral)
WHERE n.name = "session_meta.json"
RETURN s.name, r.reason
```
The `r.reason` payload is `sink:read|confidence:high`, `sink:write|confidence:high`, `sink:join|confidence:medium`, `sink:free|confidence:high`, etc. — split readers from writers without re-parsing.
