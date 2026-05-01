use crate::log_system::{Level, LogSystem};
use crate::events::{PhysicsStepEvent, RenderFrameEvent, ComponentLoadEvent};
use crate::http_server::Stats;
use std::sync::{Arc, Mutex};

pub fn wire<T: core::event_bus::EventBus + ?Sized>(event_bus: &T, logger: Arc<LogSystem>, stats: Arc<Mutex<Stats>>) {
    let log_clone = logger.clone();
    let stats_clone = stats.clone();
    event_bus.subscribe::<PhysicsStepEvent>(move |e| {
        log_clone.log(Level::Debug, "physics_core", format!("step dt={:.4}", e.dt.0));
        if let Ok(mut s) = stats_clone.lock() {
            s.dt = e.dt.0;
        }
    });
    
    let log_clone = logger.clone();
    let stats_clone = stats.clone();
    event_bus.subscribe::<RenderFrameEvent>(move |e| {
        log_clone.log(Level::Debug, "rendering", format!("frame fps={:.2}", e.fps));
        if let Ok(mut s) = stats_clone.lock() {
            s.fps = e.fps;
        }
    });
    
    let log_clone = logger.clone();
    event_bus.subscribe::<ComponentLoadEvent>(move |e| {
        log_clone.log(Level::Info, "core", format!("loaded component={}", e.component));
    });
}
