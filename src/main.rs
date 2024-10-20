use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

type Callback = Box<dyn Fn() + Send>;
type Listeners = HashMap<String, Vec<Callback>>;

struct EventEmitter {
    listeners: Mutex<Listeners>,
}

impl EventEmitter {
    fn new() -> Self {
        EventEmitter {
            listeners: Mutex::new(Listeners::new()),
        }
    }

    fn on(&self, event: &str, callback: Callback) {
        let mut listeners = self.listeners.lock().unwrap();
        listeners
            .entry(event.to_string())
            .or_default()
            .push(callback);
    }

    fn once(&self, event: impl Into<String>, callback: Callback) {
        let weak_self = Arc::downgrade(&Arc::new(self.clone()));
        let cb = Arc::new(Mutex::new(Some(callback)));
        let event: String = event.into();
        let event_clone = event.clone();
        let cb_clone = Arc::clone(&cb);
        self.on(
            &event,
            Box::new(move || {
                if let Some(strong_self) = weak_self.upgrade() {
                    if let Some(callback) = cb.lock().unwrap().take() {
                        strong_self.remove_listener(&event_clone, &callback);
                    }
                }
                if let Some(callback) = &*cb_clone.lock().unwrap() {
                    callback();
                }
            }),
        );
    }

    fn emit(&self, event: &str) {
        let listeners = self.listeners.lock().unwrap();
        if let Some(callbacks) = listeners.get(event) {
            for callback in callbacks {
                callback();
            }
        }
    }

    fn remove_listener(&self, event: &str, callback: &Callback) {
        let mut listeners = self.listeners.lock().unwrap();
        if let Some(callbacks) = listeners.get_mut(event) {
            callbacks.retain(|c| {
                !std::ptr::eq(
                    c.as_ref() as *const dyn Fn(),
                    callback.as_ref() as *const dyn Fn(),
                )
            });
        }
    }
}

impl Clone for EventEmitter {
    fn clone(&self) -> Self {
        EventEmitter {
            listeners: Mutex::new(Listeners::new()),
        }
    }
}

fn main() {
    let emitter = Arc::new(EventEmitter::new());

    // Spawn a thread that will emit the 'flag_set' event after 1 second
    let handle = thread::spawn({
        let emitter = Arc::clone(&emitter);
        move || {
            println!("Thread: Waiting for 1 second before setting the flag...");
            thread::sleep(Duration::from_secs(1));
            println!("Thread: Flag has been set.");
            emitter.emit("flag_set");
            println!("Thread: 'flag_set' event has been emitted.");
        }
    });

    println!("Main: Waiting for the flag to be set...");

    // Set up a one-time listener for the 'flag_set' event
    emitter.once(
        "flag_set",
        Box::new(|| {
            println!("Main: Flag is set, one-time listener triggered.");
        }),
    );

    // Set up a regular listener for the 'flag_set' event
    emitter.on(
        "flag_set",
        Box::new(|| {
            println!("Main: Flag is set, regular listener triggered.");
        }),
    );

    // Wait for the event to be emitted
    thread::sleep(Duration::from_secs(2));

    // Emit the event again to demonstrate the difference between 'on' and 'once'
    println!("Main: Emitting 'flag_set' event again.");
    emitter.emit("flag_set");

    // Wait for the spawned thread to finish
    handle.join().unwrap();
    println!("Main: Spawned thread has finished.");
}
