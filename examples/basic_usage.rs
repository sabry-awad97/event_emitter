use event_emitter::EventEmitter;

fn main() {
    let emitter = EventEmitter::new();

    emitter.on("greet", |args| {
        if let Some(name) = args.first().and_then(|arg| arg.downcast_ref::<String>()) {
            println!("Hello, {}!", name);
        }
    });

    emitter.emit("greet", &[&"World".to_string()]);
}
