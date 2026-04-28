// [NEEDS_REVIEW: claude]
//! Concrete `RwLock`-backed event bus implementation.
//!
//! [`LocalEventBus`] stores handlers per event type in a
//! `RwLock<HashMap<TypeId, Vec<Box<dyn Fn(&dyn Any) + Send + Sync>>>>`.
//! Publishing acquires a read lock and dispatches synchronously to all handlers.
//! Subscription acquires a write lock to append the handler.
//!
//! # Thread-safety contract
//! `publish` may be called from multiple threads simultaneously because it only
//! acquires a read lock.  `subscribe` serialises with `publish` via the write
//! lock — do not call `subscribe` from within a handler (deadlock).
//!
//! # Usage
//! ```rust,ignore
//! use core::event_bus::{EventBus, LocalEventBus};
//! let bus = LocalEventBus::new();
//! bus.subscribe::<u32>(|n| println!("got {}", n));
//! bus.publish(42u32);
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::RwLock;

use crate::event_bus::{Event, EventBus};

// ── Type-erased handler storage ───────────────────────────────────────────────

type AnyHandler = Box<dyn Fn(&dyn Any) + Send + Sync + 'static>;

// ── LocalEventBus ─────────────────────────────────────────────────────────────

/// Synchronous, `RwLock`-backed publish/subscribe event bus.
///
/// Suitable for single-process use.  For cross-process or async dispatch,
/// implement `EventBus` separately behind a channel or message queue.
pub struct LocalEventBus {
    handlers: RwLock<HashMap<TypeId, Vec<AnyHandler>>>,
}

impl LocalEventBus {
    /// Creates an empty bus with no registered handlers.
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for LocalEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus for LocalEventBus {
    fn publish<E: Event>(&self, event: E) {
        let guard = self
            .handlers
            .read()
            .expect("LocalEventBus: handler lock poisoned on publish");
        if let Some(handlers) = guard.get(&TypeId::of::<E>()) {
            for handler in handlers {
                handler(&event);
            }
        }
    }

    fn subscribe<E: Event>(&self, handler: impl Fn(&E) + Send + Sync + 'static) {
        // Wrap the typed handler in an `Any`-erased closure.
        let erased: AnyHandler = Box::new(move |any_event| {
            if let Some(e) = any_event.downcast_ref::<E>() {
                handler(e);
            }
        });

        let mut guard = self
            .handlers
            .write()
            .expect("LocalEventBus: handler lock poisoned on subscribe");
        guard.entry(TypeId::of::<E>()).or_default().push(erased);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct Ping {
        value: u32,
    }

    #[allow(dead_code)]
    #[derive(Clone)]
    struct Pong {
        text: &'static str,
    }

    #[test]
    fn single_handler_receives_event() {
        let bus = LocalEventBus::new();
        let received = Arc::new(Mutex::new(Vec::new()));
        let received_clone = Arc::clone(&received);
        bus.subscribe::<Ping>(move |e| received_clone.lock().unwrap().push(e.value));
        bus.publish(Ping { value: 42 });
        assert_eq!(*received.lock().unwrap(), vec![42]);
    }

    #[test]
    fn multiple_handlers_all_called() {
        let bus = LocalEventBus::new();
        let count = Arc::new(Mutex::new(0u32));
        let c1 = Arc::clone(&count);
        let c2 = Arc::clone(&count);
        bus.subscribe::<Ping>(move |_| *c1.lock().unwrap() += 1);
        bus.subscribe::<Ping>(move |_| *c2.lock().unwrap() += 10);
        bus.publish(Ping { value: 0 });
        assert_eq!(*count.lock().unwrap(), 11);
    }

    #[test]
    fn handlers_are_type_isolated() {
        let bus = LocalEventBus::new();
        let ping_count = Arc::new(Mutex::new(0u32));
        let pong_count = Arc::new(Mutex::new(0u32));
        let pc = Arc::clone(&ping_count);
        let pg = Arc::clone(&pong_count);
        bus.subscribe::<Ping>(move |_| *pc.lock().unwrap() += 1);
        bus.subscribe::<Pong>(move |_| *pg.lock().unwrap() += 1);
        bus.publish(Ping { value: 0 });
        bus.publish(Ping { value: 1 });
        bus.publish(Pong { text: "hi" });
        assert_eq!(*ping_count.lock().unwrap(), 2);
        assert_eq!(*pong_count.lock().unwrap(), 1);
    }

    #[test]
    fn publish_with_no_handlers_is_silent() {
        let bus = LocalEventBus::new();
        // Must not panic.
        bus.publish(Ping { value: 99 });
    }

    #[test]
    fn bus_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<LocalEventBus>();
    }
}
