---
{"id": "functions", "title": "Functions"}
---

## Functions

```ash
fn review(FILE, MODEL) {
  do "Review ${FILE}" with opencode using MODEL
}

fn build() {
  exec npm run build
}
```

Call with parentheses — required for both built-in and user functions:

```ash
FILES = get_src_files()
review("src/auth.ts", "sonnet")
build()
```

### Scoping

Variables created inside `{ }` are local — not visible outside. Outer variables are readable but not writable inside a function.

Functions compose naturally with `try`:

```ash
fn review_files(PATTERN, MODEL) {
  for FILE in exec find . -name PATTERN {
    try {
      do "Review ${FILE}" with opencode using MODEL
    } fail {
      do "Fix errors: ${stderr}" with opencode compact "truncate 16000"
    } upto 2
  }
}
```
