//! Event bus trait definition.
//!
//! Defines the interface for publishing and subscribing to typed events.
//! No concrete implementation is provided here — that is Tier B work deferred
//! after the [C1_INTERFACES_PUBLISHED] gate.
//!
//! # Design notes
//! - [`Event`] is a marker trait (like [`Component`]) that bounds on `Send + Sync + 'static`.
//! - [`EventBus`] is object-safe except for the generic methods; callers that need
//!   dynamic dispatch should use `Arc<dyn EventBus>` with a concrete impl.
//! - Handler registration is infallible from the caller's perspective; the concrete
//!   impl decides how to store handlers (e.g. `RwLock<Vec<Box<dyn Fn>>>`).

// ── Event marker trait ───────────────────────────────────────────────────────

/// Marker trait for values that can travel through the event bus.
///
/// Any `Send + Sync + 'static` type may be an event.
/// The bounds are required so events can be dispatched across thread boundaries
/// in parallel system schedulers (Tier 1+).
pub trait Event: Send + Sync + 'static {}

/// Blanket implementation: any `T: Send + Sync + 'static` is an `Event`.
impl<T: Send + Sync + 'static> Event for T {}

// ── EventBus trait ───────────────────────────────────────────────────────────

/// Typed publish/subscribe message bus.
///
/// Implementations must be `Send + Sync` so that systems running on worker
/// threads can publish events without holding a lock on the world.
///
/// # Thread safety contract
/// `publish` may be called from any thread simultaneously with other `publish`
/// calls (MPMC). The concrete impl is responsible for internal synchronisation.
pub trait EventBus: Send + Sync {
    /// Publishes `event` to all currently registered handlers for type `E`.
    ///
    /// Handlers are invoked synchronously within this call unless the concrete
    /// impl buffers events (document that in the impl, not here).
    fn publish<E: Event>(&self, event: E);

    /// Registers `handler` to be called whenever an event of type `E` is published.
    ///
    /// The handler must be `Send + Sync + 'static` so it can be stored and
    /// invoked from any thread.
    fn subscribe<E: Event>(&self, handler: impl Fn(&E) + Send + Sync + 'static);
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time check: Event blanket impl applies to a plain struct.
    #[allow(dead_code)]
    struct TestEvent {
        value: u32,
    }

    #[test]
    fn test_event_is_event() {
        fn assert_event<E: Event>() {}
        assert_event::<TestEvent>();
    }

    // Verify EventBus trait is object-safe enough to be boxed (concrete impl check
    // is deferred; this just ensures the trait compiles with its bounds).
    #[allow(dead_code)]
    fn _assert_send_sync<T: EventBus + ?Sized>(_: &T) {}
}
