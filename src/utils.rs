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
    generate_object_id(domain, &id)
}

/// Generate a follow accept id
pub fn generate_follow_accept_id(domain: &str, db_id: &str) -> Result<Url, ParseError> {
    Url::parse(&format!("https://{}/apbridge/follow/{}", domain, db_id))
}

// TODO for later aprl: needs to be base64url!!!
pub fn generate_create_id(
    domain: &str,
    create_db_id: &str,
    basesixfour_url: &str,
) -> Result<Url, ParseError> {
    Url::parse(&format!(
        "https://{}/apbridge/create/{}/{}",
        domain, create_db_id, basesixfour_url
    ))
}

pub fn generate_random_create_id(domain: &str, basesixfour_url: &str) -> Result<Url, ParseError> {
    let id: String = uuid::Uuid::new_v4().to_string();
    generate_create_id(domain, &id, basesixfour_url)
}

pub fn base_url_encode(url: &Url) -> String {
    base64_url::encode(&url.to_string())
}

pub fn base_url_decode(encoded: &str) -> String {
    String::from_utf8(base64_url::decode(encoded).unwrap()).unwrap()
}
