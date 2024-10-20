use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

type Callback = Box<dyn Fn() + Send + 'static>;
type Listeners = Arc<Mutex<HashMap<String, Vec<Callback>>>>;

#[derive(Clone, Default)]
struct EventEmitter {
    listeners: Listeners,
}

impl EventEmitter {
    fn new() -> Self {
        Self::default()
    }

    fn on<F>(&self, event: impl Into<String>, callback: F)
    where
        F: Fn() + Send + 'static
    {
        let mut listeners = self.listeners.lock().unwrap();
        listeners.entry(event.into()).or_default().push(Box::new(callback));
    }

    fn once<F>(&self, event: impl Into<String>, callback: F)
    where
        F: FnOnce() + Send + 'static
    {
        let weak_self = Arc::downgrade(&Arc::new(self.clone()));
        let event = event.into();
        let event_clone = event.clone();
        let callback = Mutex::new(Some(callback));
        self.on(event, move || {
            if let Some(strong_self) = weak_self.upgrade() {
                strong_self.remove_listener(&event_clone, Box::new(|| {}));
            }
            if let Some(cb) = callback.lock().unwrap().take() {
                cb();
            }
        });
    }

    fn emit(&self, event: &str) {
        let listeners = self.listeners.lock().unwrap();
        if let Some(callbacks) = listeners.get(event) {
            for callback in callbacks {
                callback();
            }
        }
    }

    fn remove_listener(&self, event: &str, callback: Callback) {
        let mut listeners = self.listeners.lock().unwrap();
        if let Some(callbacks) = listeners.get_mut(event) {
            callbacks.retain_mut(|c| {
                !std::ptr::eq(
                    c.as_ref() as *const dyn Fn(),
                    callback.as_ref() as *const dyn Fn(),
                )
            });
        }
    }
}

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
            emitter.emit("flag_set");
            println!("Task: 'flag_set' event has been emitted.");
        }
    });

    println!("Main: Waiting for the flag to be set...");

    // Set up a one-time listener for the 'flag_set' event
    emitter.once("flag_set", || {
        println!("Main: Flag is set, one-time listener triggered.");
    });

    // Set up a regular listener for the 'flag_set' event
    emitter.on("flag_set", || {
        println!("Main: Flag is set, regular listener triggered.");
    });

    // Wait for the event to be emitted
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Emit the event again to demonstrate the difference between 'on' and 'once'
    println!("Main: Emitting 'flag_set' event again.");
    emitter.emit("flag_set");

    // Wait for the spawned task to finish
    handle.await.unwrap();
    println!("Main: Spawned task has finished.");
}
