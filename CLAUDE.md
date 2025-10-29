# CLAUDE.md

## Rust Style

Always use in-place format! calls. For example:

Instead of:

```rust
let name = "Alice";
let age = 30;
println!("Hello, {}! You are {} years old.", name, age);
```

Do this:

```rust
let name = "Alice";
let age = 30;
println!("Hello, {name}! You are {age} years old.");
```
