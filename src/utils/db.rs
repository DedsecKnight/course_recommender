use mongodb::sync::Client;
use std::env;

pub fn create_client() -> Client {
    Client::with_uri_str(env::var("MONGODB_URI").expect("URI to combined API is required"))
        .expect("Cannot connect to MongoDB Client")
}
