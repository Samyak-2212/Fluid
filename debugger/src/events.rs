use core::units::Seconds;

pub struct PhysicsStepEvent {
    pub dt: Seconds,
}

pub struct RenderFrameEvent {
    pub fps: f64,
}

pub struct ComponentLoadEvent {
    pub component: String,
}
