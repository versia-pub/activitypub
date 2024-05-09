use url::Url;

pub enum LysandType {
    User
}

pub struct User {
    public_key: String,
    id: String,
    uri: Url,
    created_at: String,
    display_name: Option<String>
}