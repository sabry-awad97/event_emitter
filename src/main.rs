use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering};

type Callback = Arc<dyn Fn() + Send + Sync + 'static>;
type Listeners = Arc<RwLock<HashMap<String, Vec<(usize, Callback)>>>>;

#[derive(Clone)]
struct EventEmitter {
    listeners: Listeners,
    next_id: Arc<AtomicUsize>,
}

impl EventEmitter {
    fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn on<F>(&self, event: impl Into<String>, callback: F) -> usize
    where
        F: Fn() + Send + Sync + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut listeners = self.listeners.write();
        listeners
            .entry(event.into())
            .or_default()
            .push((id, Arc::new(callback)));
        id
    }

    fn once<F>(&self, event: impl Into<String>, callback: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        let weak_self = Arc::downgrade(&Arc::new(self.clone()));
        let event = event.into();
        let event_clone = event.clone();
        let callback = Mutex::new(Some(callback));
        let id = Arc::new(AtomicUsize::new(0));
        let id_clone = Arc::clone(&id);
        self.on(event, move || {
            let current_id = id_clone.load(Ordering::SeqCst);
            if let Some(strong_self) = weak_self.upgrade() {
                strong_self.remove_listener(&event_clone, current_id);
            }
            if let Some(cb) = callback.lock().unwrap().take() {
                cb();
            }
        });
        id.load(Ordering::SeqCst);
    }

    fn emit(&self, event: &str) {
        let listeners = self.listeners.read();
        if let Some(callbacks) = listeners.get(event) {
            for (_, callback) in callbacks {
                callback();
            }
        }
    }

    fn remove_listener(&self, event: &str, id: usize) {
        let mut listeners = self.listeners.write();
        if let Some(callbacks) = listeners.get_mut(event) {
            callbacks.retain(|(callback_id, _)| *callback_id != id);
        }
    }

    #[allow(dead_code)]
    fn remove_specific_listener(&self, event: &str, callback: &Callback) {
        let mut listeners = self.listeners.write();
        if let Some(callbacks) = listeners.get_mut(event) {
            callbacks.retain(|(_, cb)| !Arc::ptr_eq(cb, callback));
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_on_and_emit() {
        let emitter = EventEmitter::new();
        let counter = Arc::new(AtomicUsize::new(0));

        emitter.on("test_event", {
            let counter = Arc::clone(&counter);
            move || {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event");
        emitter.emit("test_event");

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_once() {
        let emitter = EventEmitter::new();
        let counter = Arc::new(AtomicUsize::new(0));

        emitter.once("test_event", {
            let counter = Arc::clone(&counter);
            move || {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event");
        emitter.emit("test_event");

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_remove_listener() {
        let emitter = EventEmitter::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let callback_id = emitter.on("test_event", {
            let counter = Arc::clone(&counter);
            move || {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event");
        println!("Counter after first emit: {}", counter.load(Ordering::SeqCst));

        println!("Listeners before removal: {:?}", emitter.listeners.read().get("test_event").map(|v| v.len()));
        emitter.remove_listener("test_event", callback_id);
        println!("Listeners after removal: {:?}", emitter.listeners.read().get("test_event").map(|v| v.len()));

        emitter.emit("test_event");
        println!("Counter after second emit: {}", counter.load(Ordering::SeqCst));

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_listeners() {
        let emitter = EventEmitter::new();
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        emitter.on("test_event", {
            let counter = Arc::clone(&counter1);
            move || {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.on("test_event", {
            let counter = Arc::clone(&counter2);
            move || {
                counter.fetch_add(2, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event");

        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_non_existent_event() {
        let emitter = EventEmitter::new();
        // This should not panic or cause any errors
        emitter.emit("non_existent_event");
    }
}
