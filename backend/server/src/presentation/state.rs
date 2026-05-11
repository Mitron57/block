use std::sync::Arc;

use crate::application::{AuthService, BoardService};
use crate::presentation::rooms::RoomRegistry;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthService>,
    pub boards: Arc<BoardService>,
    pub rooms: Arc<RoomRegistry>,
}
