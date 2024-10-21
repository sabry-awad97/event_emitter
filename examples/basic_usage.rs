use event_emitter::EventEmitter;

fn main() {
    let emitter = EventEmitter::new();

    emitter.on("greet", |args| {
        if let Some(name) = args.first().and_then(|arg| arg.downcast_ref::<String>()) {
            println!("Hello, {}!", name);
        } else {
            println!("Hello, world!");
        }
    });

    emitter.emit("greet", ());
    emitter.emit("greet", ("Sabry".to_string(),));
}
