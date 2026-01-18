// Library exports for testing and reuse
pub mod api;
pub mod models;
pub mod utils;

// Re-export commonly used types
pub use api::TailscaleClient;
pub use models::{
    Contact, ContactsResponse, Device, DevicesResponse, UpdateTagsRequest, User, UsersResponse,
};
