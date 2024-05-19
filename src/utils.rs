use url::{ParseError, Url};

pub fn generate_object_id(domain: &str, uuid: &str) -> Result<Url, ParseError> {
    let id: String = uuid::Uuid::new_v4().to_string();
    Url::parse(&format!("https://{}/apbridge/object/{}", domain, id))
}

pub fn generate_user_id(domain: &str, uuid: &str) -> Result<Url, ParseError> {
    let id: String = uuid::Uuid::new_v4().to_string();
    Url::parse(&format!("https://{}/apbridge/user/{}", domain, id))
}

pub fn generate_random_object_id(domain: &str) -> Result<Url, ParseError> {
    let id: String = uuid::Uuid::new_v4().to_string();
    Url::parse(&format!("https://{}/apbridge/object/{}", domain, id))
}

/// Generate a follow accept id
pub fn generate_follow_accept_id(domain: &str, db_id: i32) -> Result<Url, ParseError> {
    Url::parse(&format!("https://{}/apbridge/activity/follow/{}", domain, db_id))
}
