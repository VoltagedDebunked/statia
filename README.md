# Statia

Zero-dependency state management for Rust applications. Thread-safe, type-safe, and built entirely on stdlib.

## Features

- Thread-safe state containers using RwLock
- Pub/sub system for state changes
- Transaction support for atomic updates
- Type-safe state registry
- Pure stdlib - no external dependencies

## Usage

```rust
use statia::{State, StateManager};

// Single state
let counter = State::new(0);
counter.set(42);
assert_eq!(counter.get(), 42);

// Subscribe to changes
counter.subscribe(|value| println!("Counter changed to: {}", value));

// Multiple states
let manager = StateManager::new();
let count_state = manager.register("count", 0);
let name_state = manager.register("name", String::from("test"));

// Transactions
let mut transaction = Transaction::new(counter);
transaction.update(|v| *v += 1);
transaction.update(|v| *v *= 2);
transaction.commit();
```

## Installation

Add to Cargo.toml:
```toml
[dependencies]
statia = "0.1.2"
```

## License

GPL-3.0

## Contributing

Pull requests welcome! Please read CONTRIBUTING.md first.