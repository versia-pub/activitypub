use super::objects::SortAlphabetically;

pub async fn deserialize_user(data: String) -> anyhow::Result<super::objects::User> {
    let user: super::objects::User = serde_json::from_str(&data)?;
    Ok(user)
}

pub async fn serialize_user(user: super::objects::User) -> anyhow::Result<String> {
    let data = serde_json::to_string(&SortAlphabetically(&user))?;
    Ok(data)
}

pub async fn deserialize_versia_type(data: String) -> anyhow::Result<String> {
    let versia_type: String = serde_json::from_str(&data)?;
    Ok(versia_type)
}

pub async fn serialize_versia_type(
    versia_type: String,
) -> anyhow::Result<String> {
    let data = serde_json::to_string(&versia_type)?;
    Ok(data)
}

pub async fn deserialize_note(data: String) -> anyhow::Result<super::objects::Note> {
    let post: super::objects::Note = serde_json::from_str(&data)?;
    Ok(post)
}

pub async fn serialize_note(post: super::objects::Note) -> anyhow::Result<String> {
    let data = serde_json::to_string(&SortAlphabetically(&post))?;
    Ok(data)
}

pub async fn deserialize_outbox(data: String) -> anyhow::Result<super::objects::Outbox> {
    let outbox: super::objects::Outbox = serde_json::from_str(&data)?;
    Ok(outbox)
}

pub async fn serialize_outbox(outbox: super::objects::Outbox) -> anyhow::Result<String> {
    let data = serde_json::to_string(&SortAlphabetically(&outbox))?;
    Ok(data)
}

#[inline]
pub fn request_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
        ))
        .build()
        .unwrap()
}
