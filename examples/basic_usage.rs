use event_emitter::EventEmitter;

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("trace"));
    let emitter = EventEmitter::new();

    // Register a listener for the "user_login" event
    emitter.on("user_login", |(username, timestamp): (String, u64)| {
        println!("User '{}' logged in at timestamp: {}", username, timestamp);
    });

    // Register a listener for the "user_logout" event
    emitter.on("user_logout", |(username,): (String,)| {
        println!("User '{}' logged out", username);
    });

    // Simulate user activities
    emitter.emit("user_login", ("alice".to_string(), 1623456789u64));
    emitter.emit("user_login", ("bob".to_string(), 1623456790u64));
    emitter.emit("user_logout", ("alice".to_string(),));

    // This login event will be logged
    emitter.emit("user_login", ("charlie".to_string(), 1623456791u64));

    // We can still see logout events
    emitter.emit("user_logout", ("bob".to_string(),));

    // Print the number of listeners for each event
    println!(
        "Active listeners for 'user_login': {}",
        emitter.listeners("user_login").len()
    );
    println!(
        "Active listeners for 'user_logout': {}",
        emitter.listeners("user_logout").len()
    );
}
