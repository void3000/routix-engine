# Routix DSL Grammar Summary

## Supported Features

### 1. **Workflows**
- Define **workflows** that group rules and matching logic.
- Syntax:
  ```plaintext
  workflow <name> {
      <score_phase or match_phase>*
  }
  ```

---

### 2. **Phases**
1. **Score Phase**
   - Evaluates conditions to **increase scores** or **log messages**.
   - Syntax:
     ```plaintext
     score {
         when <expr> then score += <number>
         when <expr> then log "<message>"
     }
     ```

2. **Match Phase**
   - Evaluates conditions to **assign cases to agents**.
   - Syntax:
     ```plaintext
     match {
         when <expr> then assign to <agent_id>
     }
     ```

---

### 3. **Rules**
- **When–Then** structure:
  - `score` rules → `score +=` or `log`
  - `match` rules → `assign to <agent>`

---

### 4. **Functions**
- **Define reusable functions**:
  ```plaintext
  function <name>(param1, param2) = <expr>
  ```
- **Call functions** inside expressions:
  ```plaintext
  when decay(case.score, case.age) > 10 then score += 5
  ```

---

### 5. **Expressions**
- **Arithmetic**: `+` `-` `*` `/`
- **Comparison**: `==` `!=` `>` `<` `>=` `<=` `in`
- **Logical operators**: `and` / `or` / `!`
- **Parentheses** for grouping: `(expr)`

---

### 6. **Literals**
- **Numbers** → `123`
- **Strings** → `"hello"`
- **Booleans** → `true` / `false`
- **Lists** → `[1, 2, 3]` or `["a", "b"]`
- **Identifiers** → variables like `case.score`, `agent.skills`

---

### 7. **Comments**
- Start with `#` and run until the end of the line.

---

### 8. **Whitespace**
- Ignored: spaces, tabs, newlines, carriage returns.

---

## 🔹 Example

```plaintext
# Define a decay function
method decay(score, days) = score / (days + 1)

workflow auto_assign {
    score {
        when case.priority > 5 then score += 10
        when decay(case.score, case.age) > 50 then log "High decay score"
    }

    match {
        when case.category == "billing" and agent.skills in ["billing", "finance"]
        then assign to agent_42
    }
}
```


```plaintext
# Define a decay function
function decay(score, days) = score / (days + 1)

workflow auto_assign {
    SCORE {
        WHEN case.priority > 5
          THEN score += 10
        WHEN decay(case.score, case.age) > 50
          THEN log "High decay score"
    }

    MATCH {
        WHEN case.category == "billing" AND agent.skills IN ["billing", "finance"]
        THEN assign to agent_42
    }
}
```
