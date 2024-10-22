# EventEmitter

EventEmitter is a robust, thread-safe event emitter library for Rust, designed to implement the publish-subscribe pattern in concurrent applications efficiently.

## Features

- Thread-safe event emission and handling
- Support for multiple listeners per event
- One-time event listeners with `once` method
- Configurable maximum number of listeners
- Type-safe event arguments using generics
- Comprehensive error handling and logging
- Minimal performance overhead with `SmallVec` optimization

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
event_emitter = { git = "https://github.com/sabry-awad97/event_emitter", branch = "main" }
```

## API Overview

- `EventEmitter::new()`: Create a new EventEmitter instance
- `EventEmitter::with_capacity(max_listeners: NonZeroUsize)`: Create an EventEmitter with a custom maximum listener capacity
- `on<F, Args, E>(event: E, callback: F) -> usize`: Register a listener for an event
- `once<F, Args, E>(event: E, callback: F) -> usize`: Register a one-time listener
- `emit<A, E>(event: E, args: A)`: Emit an event with arguments
- `remove_listener<E: IntoEvent>(event: E, id: usize)`: Remove a specific listener
- `remove_listeners()`: Remove all listeners for all events
- `listeners_count<E: IntoEvent>(event: E) -> usize`: Get the number of listeners for an event
- `set_max_listeners(&mut self, max: NonZeroUsize)`: Set the maximum number of listeners
- `get_max_listeners(&self) -> NonZeroUsize`: Get the current maximum number of listeners

## Usage

### Basic Usage

Here's a basic example demonstrating the core functionality of EventEmitter:

```rust
use event_emitter::EventEmitter;
fn main() {
let emitter = EventEmitter::new();
    // Register a regular listener
    emitter.on("greet", |args: (String,)| {
        println!("Hello, {}!", args.0);
    });
    // Register a one-time listener
    emitter.once("one_time", |args: (i32,)| {
        println!("This will only be printed once: {}", args.0);
    });
    // Emit events
    emitter.emit("greet", ("World".to_string(),));
    emitter.emit("one_time", (42,));
    emitter.emit("one_time", (24,)); // This won't trigger the listener
    // Check listener count
    println!("Listeners for 'greet': {}", emitter.listeners_count("greet"));
    println!("Listeners for 'one_time': {}", emitter.listeners_count("one_time"));
}
```

### Custom Event Types

The `IntoEvent` trait allows for custom event types:

```rs
use std::borrow::Cow;

use event_emitter::{EventEmitter, IntoEvent};

enum MyEvent {
    Startup,
    Shutdown,
}

impl IntoEvent for MyEvent {
    fn into_event(self) -> Cow<'static, str> {
        match self {
            MyEvent::Startup => Cow::Borrowed("startup"),
            MyEvent::Shutdown => Cow::Borrowed("shutdown"),
        }
    }
}

fn main() {
    let emitter = EventEmitter::new();
    emitter.on(MyEvent::Startup, |_args: ()| println!("System starting up"));
    emitter.on(MyEvent::Shutdown, |_args: ()| {
        println!("System shutting down")
    });
    emitter.emit(MyEvent::Startup, ());
    emitter.emit(MyEvent::Shutdown, ());
}
```

### Thread Safety

The EventEmitter is designed to be thread-safe. You can safely share and use an `EventEmitter` instance across multiple threads:

```rs
use std::{sync::Arc, thread, time::Duration};
use event_emitter::EventEmitter;

fn main() {
    let emitter = Arc::new(EventEmitter::new());
    let emitter_clone = Arc::clone(&emitter);

    thread::spawn(move || {
        emitter_clone.emit(
            "threaded_event",
            ("Hello from another thread!".to_string(),),
        );
    });

    emitter.on("threaded_event", |args: (String,)| {
        println!("Received: {}", args.0);
    });

    // Wait for the thread to finish
    thread::sleep(Duration::from_secs(1));
}
```

### Error Handling

The EventEmitter uses the `log` crate for internal logging. Ensure you set up a logger in your application to capture any warnings or errors:

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to get started.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the Node.js EventEmitter API
- Built with ❤️ by the Rust community
