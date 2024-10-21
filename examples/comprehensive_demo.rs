use std::{sync::Arc, time::Duration};

use event_emitter::EventEmitter;

#[tokio::main]
async fn main() {
    let emitter = Arc::new(EventEmitter::new());

    // Spawn a task that will emit the 'flag_set' event after 1 second
    let handle = tokio::spawn({
        let emitter = Arc::clone(&emitter);
        async move {
            println!("Task: Waiting for 1 second before setting the flag...");
            tokio::time::sleep(Duration::from_secs(1)).await;
            println!("Task: Flag has been set.");
            emitter.emit("flag_set", &[]);
            println!("Task: 'flag_set' event has been emitted.");
        }
    });

    println!("Main: Waiting for the flag to be set...");

    // Set up a one-time listener for the 'flag_set' event
    emitter.once("flag_set", |_args| {
        println!("Main: Flag is set, one-time listener triggered.");
    });

    // Set up a regular listener for the 'flag_set' event
    let listener_id = emitter.on("flag_set", |_args| {
        println!("Main: Flag is set, regular listener triggered.");
    });

    // Wait for the event to be emitted
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Emit the event again to demonstrate the difference between 'on' and 'once'
    println!("Main: Emitting 'flag_set' event again.");
    emitter.emit("flag_set", &[]);

    // Remove the regular listener for the 'flag_set' event
    println!("Main: Removing regular listener for 'flag_set' event.");
    emitter.remove_listener("flag_set", listener_id);

    // Emit the event again to show that the regular listener is no longer triggered
    println!("Main: Emitting 'flag_set' event after removing regular listener.");
    emitter.emit("flag_set", &[]);

    // Set up a prepended listener for the 'flag_set' event
    emitter.prepend_listener("flag_set", |_args| {
        println!("Main: Flag is set, prepended listener triggered.");
    });

    // Emit the event to show the prepended listener is triggered first
    println!("Main: Emitting 'flag_set' event with prepended listener.");
    emitter.emit("flag_set", &[]);

    // Set up a prepended one-time listener for the 'flag_set' event
    emitter.prepend_once_listener("flag_set", |_args| {
        println!("Main: Flag is set, prepended one-time listener triggered.");
    });

    // Emit the event to show the prepended one-time listener is triggered first and only once
    println!("Main: Emitting 'flag_set' event with prepended one-time listener.");
    emitter.emit("flag_set", &[]);
    println!("Main: Emitting 'flag_set' event again to show prepended one-time listener is gone.");
    emitter.emit("flag_set", &[]);

    // Remove all listeners for the 'flag_set' event
    println!("Main: Removing all listeners for 'flag_set' event.");
    emitter.remove_listeners("flag_set");

    // Emit the event once more to show that no listeners are triggered
    println!("Main: Emitting 'flag_set' event after removing all listeners.");
    emitter.emit("flag_set", &[]);

    // Wait for the spawned task to finish
    handle.await.unwrap();
    println!("Main: Spawned task has finished.");
}
