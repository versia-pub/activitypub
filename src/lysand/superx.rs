use super::objects::SortAlphabetically;

pub async fn deserialize_user(data: String) -> anyhow::Result<super::objects::User> {
    let user: super::objects::User = serde_json::from_str(&data)?;
    Ok(user)
}

pub async fn serialize_user(user: super::objects::User) -> anyhow::Result<String> {
    let data = serde_json::to_string(&SortAlphabetically(&user))?;
    Ok(data)
}

pub async fn deserialize_lysand_type(data: String) -> anyhow::Result<super::objects::LysandType> {
    let lysand_type: super::objects::LysandType = serde_json::from_str(&data)?;
    Ok(lysand_type)
}

pub async fn serialize_lysand_type(
    lysand_type: super::objects::LysandType,
) -> anyhow::Result<String> {
    let data = serde_json::to_string(&lysand_type)?;
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
