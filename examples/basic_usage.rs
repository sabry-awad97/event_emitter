use colored::*;
use event_emitter::EventEmitter;

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("trace"));
    let emitter = EventEmitter::new();

    // Register a listener for the "user_login" event
    let id = emitter.on("user_login", |(username, timestamp): (String, u64)| {
        println!(
            "{} '{}' logged in at timestamp: {}",
            "User".green(),
            username.blue(),
            timestamp
        );
    });

    // Register a listener for the "user_logout" event
    emitter.on("user_logout", |(username,): (String,)| {
        println!("{} '{}' logged out", "User".green(), username.blue());
    });

    // Simulate user activities
    emitter
        .emit("user_login", ("alice".to_string(), 1623456789u64))
        .unwrap();
    emitter
        .emit("user_login", ("bob".to_string(), 1623456790u64))
        .unwrap();
    emitter.emit("user_logout", ("alice".to_string(),)).unwrap();

    emitter.remove_listener("user_login", id);

    // This login event will be logged
    emitter
        .emit("user_login", ("charlie".to_string(), 1623456791u64))
        .unwrap();

    // We can still see logout events
    emitter.emit("user_logout", ("bob".to_string(),)).unwrap();

    // Print the number of listeners for each event
    println!(
        "{} for 'user_login': {}",
        "Active listeners".yellow(),
        emitter.listeners_count("user_login").to_string().red()
    );

    println!(
        "{} for 'user_logout': {}",
        "Active listeners".yellow(),
        emitter.listeners_count("user_logout").to_string().red()
    );
}
