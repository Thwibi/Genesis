use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

use crate::domain::entities::session::Session;

pub struct SessionManager {
    active_sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, session: Session) {
        self.active_sessions.write().await.insert(session.id, session);
    }

    pub async fn unregister(&self, session_id: Uuid) {
        self.active_sessions.write().await.remove(&session_id);
    }

    pub async fn get(&self, session_id: Uuid) -> Option<Session> {
        self.active_sessions.read().await.get(&session_id).cloned()
    }

    pub async fn get_active_sessions(&self) -> Vec<Session> {
        self.active_sessions.read().await.values().cloned().collect()
    }

    pub async fn cleanup_expired(&self, max_inactivity: chrono::Duration) {
        let mut sessions = self.active_sessions.write().await;
        let now = Utc::now();
        sessions.retain(|id, session| {
            let elapsed = now - session.last_activity_timestamp;
            if elapsed > max_inactivity {
                info!(target: "session", session_id = %id, "Session expired and cleaned up");
                false
            } else {
                true
            }
        });
    }
}
