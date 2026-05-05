pub mod callbacks;
mod formatting;
pub mod handlers;
mod session_io;

// Re-export public API so external callers (event_dispatch.rs) need no changes
pub use callbacks::{CallbackContext, handle_callback};
pub use handlers::*;
