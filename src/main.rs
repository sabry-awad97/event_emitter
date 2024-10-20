use parking_lot::RwLock;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

type Callback = Arc<dyn Fn(&[Box<dyn Any + Send + Sync>]) + Send + Sync + 'static>;
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
        F: Fn(&[Box<dyn Any + Send + Sync>]) + Send + Sync + 'static,
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
        F: FnOnce(&[Box<dyn Any + Send + Sync>]) + Send + Sync + 'static,
    {
        let weak_self = Arc::downgrade(&Arc::new(self.clone()));
        let event = event.into();
        let event_clone = event.clone();
        let callback = Mutex::new(Some(callback));
        let id = Arc::new(AtomicUsize::new(0));
        let id_clone = Arc::clone(&id);
        self.on(event, move |args| {
            let current_id = id_clone.load(Ordering::SeqCst);
            if let Some(strong_self) = weak_self.upgrade() {
                strong_self.remove_listener(&event_clone, current_id);
            }
            if let Some(cb) = callback.lock().unwrap().take() {
                cb(args);
            }
        });
        id.load(Ordering::SeqCst);
    }

    fn emit(&self, event: &str, args: &[Box<dyn Any + Send + Sync>]) {
        let listeners = self.listeners.read();
        if let Some(callbacks) = listeners.get(event) {
            for (_, callback) in callbacks {
                callback(args);
            }
        }
    }

    fn listeners(&self, event: &str) -> Vec<Callback> {
        let listeners = self.listeners.read();
        listeners
            .get(event)
            .map(|callbacks| callbacks.iter().map(|(_, cb)| Arc::clone(cb)).collect())
            .unwrap_or_default()
    }

    
    fn listener_count(&self, event: &str) -> usize {
        let listeners = self.listeners.read();
        listeners
            .get(event)
            .map(|callbacks| callbacks.len())
            .unwrap_or(0)
    }

    fn event_names(&self) -> Vec<String> {
        let listeners = self.listeners.read();
        listeners.keys().cloned().collect()
    }

    fn get_max_listeners(&self) -> usize {
        // In this implementation, we don't have a max listeners limit
        // You can add a field to EventEmitter to store this value if needed
        usize::MAX
    }

    fn set_max_listeners(&mut self, _n: usize) {
        // In this implementation, we don't enforce a max listeners limit
        // You can add a field to EventEmitter to store this value if needed
    }

    fn raw_listeners(&self, event: &str) -> Vec<Callback> {
        // In this implementation, raw_listeners is the same as listeners
        self.listeners(event)
    }

    fn prepend_listener<F>(&self, event: impl Into<String>, callback: F) -> usize
    where
        F: Fn(&[Box<dyn Any + Send + Sync>]) + Send + Sync + 'static,
    {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut listeners = self.listeners.write();
        listeners
            .entry(event.into())
            .or_default()
            .insert(0, (id, Arc::new(callback)));
        id
    }

    fn prepend_once_listener<F>(&self, event: impl Into<String>, callback: F)
    where
        F: FnOnce(&[Box<dyn Any + Send + Sync>]) + Send + Sync + 'static,
    {
        let weak_self = Arc::downgrade(&Arc::new(self.clone()));
        let event = event.into();
        let event_clone = event.clone();
        let callback = Mutex::new(Some(callback));
        let id = Arc::new(AtomicUsize::new(0));
        let id_clone = Arc::clone(&id);
        self.prepend_listener(event, move |args| {
            let current_id = id_clone.load(Ordering::SeqCst);
            if let Some(strong_self) = weak_self.upgrade() {
                strong_self.remove_listener(&event_clone, current_id);
            }
            if let Some(cb) = callback.lock().unwrap().take() {
                cb(args);
            }
        });
        id.load(Ordering::SeqCst);
    }

    fn remove_listener(&self, event: &str, id: usize) {
        let mut listeners = self.listeners.write();
        if let Some(callbacks) = listeners.get_mut(event) {
            callbacks.retain(|(callback_id, _)| *callback_id != id);
        }
    }

    fn remove_listeners(&self, event: &str) {
        let mut listeners = self.listeners.write();
        listeners.remove(event);
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
            move |_args| {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event", &[]);
        emitter.emit("test_event", &[]);

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_once() {
        let emitter = EventEmitter::new();
        let counter = Arc::new(AtomicUsize::new(0));

        emitter.once("test_event", {
            let counter = Arc::clone(&counter);
            move |_args| {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event", &[]);
        emitter.emit("test_event", &[]);

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_remove_listener() {
        let emitter = EventEmitter::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let callback_id = emitter.on("test_event", {
            let counter = Arc::clone(&counter);
            move |_args| {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event", &[]);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        emitter.remove_listener("test_event", callback_id);

        emitter.emit("test_event", &[]);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_listeners() {
        let emitter = EventEmitter::new();
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        emitter.on("test_event", {
            let counter = Arc::clone(&counter1);
            move |_args| {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.on("test_event", {
            let counter = Arc::clone(&counter2);
            move |_args| {
                counter.fetch_add(2, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event", &[]);

        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_non_existent_event() {
        let emitter = EventEmitter::new();
        // This should not panic or cause any errors
        emitter.emit("non_existent_event", &[]);
    }

    #[test]
    fn test_remove_listeners() {
        let emitter = EventEmitter::new();
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        emitter.on("test_event", {
            let counter = Arc::clone(&counter1);
            move |_args| {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.on("test_event", {
            let counter = Arc::clone(&counter2);
            move |_args| {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        });

        emitter.emit("test_event", &[]);
        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 1);

        emitter.remove_listeners("test_event");
        emitter.emit("test_event", &[]);
        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_listener_count() {
        let emitter = EventEmitter::new();
        assert_eq!(emitter.listener_count("test_event"), 0);

        emitter.on("test_event", |_args| {});
        assert_eq!(emitter.listener_count("test_event"), 1);

        emitter.on("test_event", |_args| {});
        assert_eq!(emitter.listener_count("test_event"), 2);

        emitter.remove_listeners("test_event");
        assert_eq!(emitter.listener_count("test_event"), 0);
    }

    #[test]
    fn test_event_names() {
        let emitter = EventEmitter::new();
        assert!(emitter.event_names().is_empty());

        emitter.on("event1", |_args| {});
        emitter.on("event2", |_args| {});
        emitter.on("event3", |_args| {});

        let names = emitter.event_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"event1".to_string()));
        assert!(names.contains(&"event2".to_string()));
        assert!(names.contains(&"event3".to_string()));
    }

    #[test]
    fn test_prepend_listener() {
        let emitter = EventEmitter::new();
        let order = Arc::new(Mutex::new(Vec::new()));

        emitter.on("test_event", {
            let order = Arc::clone(&order);
            move |_args| {
                order.lock().unwrap().push(2);
            }
        });

        emitter.prepend_listener("test_event", {
            let order = Arc::clone(&order);
            move |_args| {
                order.lock().unwrap().push(1);
            }
        });

        emitter.emit("test_event", &[]);

        assert_eq!(*order.lock().unwrap(), vec![1, 2]);
    }
}
