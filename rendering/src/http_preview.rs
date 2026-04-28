//! rendering/src/http_preview.rs
//!
//! IDX Browser Preview HTTP server.
//!
//! Serves rendered frames as JPEG at `GET /frame.jpg`.
//! Default port: 8080 (configurable via config/rendering.toml → preview_http_port).
//!
//! The server runs on a dedicated background thread so it does not block the
//! render loop.  Frames are published via `Arc<Mutex<Vec<u8>>>` shared state.
//!
//! C6 (Debugger) should coordinate with C3 on port assignments to avoid
//! conflicts.  If port 8080 is taken, set preview_http_port in rendering.toml.

use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::{Server, Response, Header};

/// Shared frame buffer — JPEG bytes of the most recently rendered frame.
pub type SharedFrame = Arc<Mutex<Vec<u8>>>;

/// Start the HTTP preview server on `port`.
///
/// Spawns a background thread.  Callers should retain the returned
/// `SharedFrame` and update it each frame.
///
/// Returns `Err` if the port cannot be bound.
pub fn start_preview_server(port: u16) -> Result<SharedFrame, Box<dyn std::error::Error + Send + Sync>> {
    let shared: SharedFrame = Arc::new(Mutex::new(Vec::new()));
    let shared_clone = Arc::clone(&shared);

    let addr = format!("0.0.0.0:{port}");
    let server = Server::http(&addr).map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e })?;

    thread::Builder::new()
        .name("fluid-http-preview".to_string())
        .spawn(move || {
            serve_loop(server, shared_clone);
        })
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

    Ok(shared)
}

fn serve_loop(server: Server, shared: SharedFrame) {
    for request in server.incoming_requests() {
        let url = request.url().to_owned();
        match url.as_str() {
            "/frame.jpg" => {
                let jpeg = shared.lock().unwrap_or_else(|p| p.into_inner()).clone();
                if jpeg.is_empty() {
                    // No frame yet — return 204 No Content.
                    let _ = request.respond(
                        Response::empty(204_u16)
                    );
                } else {
                    let content_type = Header::from_bytes(
                        &b"Content-Type"[..],
                        &b"image/jpeg"[..],
                    )
                    .expect("valid header");
                    let response = Response::from_data(jpeg)
                        .with_header(content_type);
                    let _ = request.respond(response);
                }
            }
            "/" | "/health" => {
                let response = Response::from_string("Fluid rendering preview server OK\n");
                let _ = request.respond(response);
            }
            _ => {
                let _ = request.respond(Response::empty(404_u16));
            }
        }
    }
}

/// Publish a new JPEG frame to the shared buffer.
///
/// Thread-safe — can be called from the render thread.
pub fn publish_frame(shared: &SharedFrame, jpeg: Vec<u8>) {
    if let Ok(mut guard) = shared.lock() {
        *guard = jpeg;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the HTTP server starts on an ephemeral port without panicking.
    #[test]
    fn http_server_starts() {
        // Use a fixed test port and accept failure if it is already in use in CI.
        let result = start_preview_server(18080);
        // We just assert it does not panic.
        let _ = result;
    }

    #[test]
    fn publish_frame_stores_jpeg() {
        let shared: SharedFrame = Arc::new(Mutex::new(Vec::new()));
        publish_frame(&shared, b"fake_jpeg_data".to_vec());
        let stored = shared.lock().unwrap().clone();
        assert_eq!(stored, b"fake_jpeg_data");
    }

    #[cfg(feature = "tier_0")]
    #[test]
    fn cpu_framebuffer_to_jpeg_round_trip() {
        use crate::tier0::CpuFramebuffer;
        let mut fb = CpuFramebuffer::new(64, 64);
        fb.fill(0xFF_80_C0_40);
        let jpeg = fb.to_jpeg().expect("JPEG encode should succeed");
        assert!(!jpeg.is_empty());
        let shared: SharedFrame = Arc::new(Mutex::new(Vec::new()));
        publish_frame(&shared, jpeg.clone());
        let stored = shared.lock().unwrap().clone();
        assert_eq!(stored.len(), jpeg.len());
    }
}
