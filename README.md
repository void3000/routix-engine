### UI Console Editor

```
+------------------------------------------------------+
| Functions Panel                                      |
| [Add Function]  | List of functions                 |
| -> Select function -> edit name, params, body       |
+------------------------------------------------------+
| Workflows Panel                                      |
| [Add Workflow]                                      |
| -> Workflow Tree View                                |
|   - Phase: score                                     |
|       - Rule: when x > 10 then score = 5            |
|   - Phase: match                                     |
|       - Rule: when y == 0 then assign to "default"  |
+------------------------------------------------------+
| Expression Editor Panel                               |
| - Inline editor for expressions                      |
| - Syntax highlighting                                 |
+------------------------------------------------------+
| Code Preview Panel                                   |
| - Shows generated DSL in real time                  |
+------------------------------------------------------+
```

## Conversion

- DSL → JSON → UI: User edits in the visual editor.
- UI → JSON → DSL: Serializer regenerates DSL code.

#### DSL → JSON (Parsing)

- Use the parser to parse DSL text into a parse tree / AST.
- Then convert the AST to a JSON-like internal model.
    - Functions → array of `{ name, params, body }`
    - Workflows → array of `{ name, phases: [...] }`
    - Rules → `{ condition, action }`

#### JSON → DSL (Serialization)

- Traverse the JSON model and generate DSL text using a serializer (tree walker).
- Example: `workflow.phase.rules` → when condition then action;