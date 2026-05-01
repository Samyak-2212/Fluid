use std::sync::{Arc, Mutex};
use std::thread;
use std::fs;
#[allow(unused_imports)]
use std::io::Read;
use tiny_http::{Server, Response, Header, Method, StatusCode};
use crate::log_system::LogSystem;
use crate::bug_pool_reporter::{report_bug, BugEntry, Severity};
use serde_json::json;
use serde::Deserialize;

const INDEX_HTML: &str = include_str!("assets/index.html");

#[derive(Default, Clone)]
pub struct Stats {
    pub dt: f64,
    pub fps: f64,
    pub entities: u64,
}

pub struct DebuggerHttpServer {
    port: u16,
    logger: Arc<LogSystem>,
    stats: Arc<Mutex<Stats>>,
}

#[derive(Deserialize)]
struct BugReportPayload {
    id: String,
    severity: String,
    component: String,
    reported_by: String,
    description: String,
    reproduction: String,
}

impl DebuggerHttpServer {
    pub fn new(port: u16, logger: Arc<LogSystem>, stats: Arc<Mutex<Stats>>) -> Self {
        Self { port, logger, stats }
    }

    pub fn run(self) {
        let server = Server::http(format!("127.0.0.1:{}", self.port))
            .expect("Failed to start embedded HTTP server");
            
        thread::spawn(move || {
            for mut request in server.incoming_requests() {
                let url = request.url().to_string();
                
                match (request.method(), url.as_str()) {
                    (&Method::Get, "/") => {
                        let response = Response::from_string(INDEX_HTML)
                            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap());
                        let _ = request.respond(response);
                    }
                    (&Method::Get, "/logs") => {
                        let lines = if let Ok(content) = fs::read_to_string(self.logger.get_path()) {
                            content.lines().map(|s| s.to_string()).collect::<Vec<String>>()
                        } else {
                            Vec::new()
                        };
                        let json = json!({ "lines": lines }).to_string();
                        let response = Response::from_string(json)
                            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                        let _ = request.respond(response);
                    }
                    (&Method::Get, "/stats") => {
                        let stats = {
                            let s = self.stats.lock().unwrap();
                            s.clone()
                        };
                        let json = json!({
                            "dt": stats.dt,
                            "fps": stats.fps,
                            "entities": stats.entities
                        }).to_string();
                        let response = Response::from_string(json)
                            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                        let _ = request.respond(response);
                    }
                    (&Method::Get, "/bug_pool") => {
                        let content = fs::read_to_string("bug_pool/BUG_POOL.md").unwrap_or_else(|_| "Error loading BUG_POOL.md".to_string());
                        let response = Response::from_string(content)
                            .with_header(Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap());
                        let _ = request.respond(response);
                    }
                    (&Method::Post, "/report_bug") => {
                        let mut content = String::new();
                        let _ = request.as_reader().read_to_string(&mut content);
                        
                        if let Ok(payload) = serde_json::from_str::<BugReportPayload>(&content) {
                            let severity = match payload.severity.to_uppercase().as_str() {
                                "CRITICAL" => Severity::Critical,
                                "HIGH" => Severity::High,
                                "MEDIUM" => Severity::Medium,
                                "LOW" => Severity::Low,
                                "REVIEW" => Severity::Review,
                                "PROCESS" => Severity::Process,
                                _ => Severity::Low,
                            };
                            
                            let bug = BugEntry {
                                id: payload.id,
                                severity,
                                component: payload.component,
                                reported_by: payload.reported_by,
                                description: payload.description,
                                reproduction: payload.reproduction,
                            };
                            
                            if let Ok(_) = report_bug(bug) {
                                let response = Response::from_string("{\"status\":\"ok\"}")
                                    .with_header(Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap());
                                let _ = request.respond(response);
                            } else {
                                let response = Response::from_string("{\"error\":\"failed to write bug\"}")
                                    .with_status_code(StatusCode(500));
                                let _ = request.respond(response);
                            }
                        } else {
                            let response = Response::from_string("{\"error\":\"invalid json\"}")
                                .with_status_code(StatusCode(400));
                            let _ = request.respond(response);
                        }
                    }
                    _ => {
                        let response = Response::from_string("Not Found")
                            .with_status_code(StatusCode(404));
                        let _ = request.respond(response);
                    }
                }
            }
        });
    }
}
