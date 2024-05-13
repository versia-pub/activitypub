use crate::lysand::objects::SortAlphabetically;

use super::superx::request_client;

pub async fn main() -> anyhow::Result<()> {
    let client = request_client();

    println!("Requesting user");
    let response = client
        .get("https://social.lysand.org/users/018ec082-0ae1-761c-b2c5-22275a611771")
        .send()
        .await?;
    println!("Response: {:?}", response);
    let user_json = response.text().await?;
    println!("User JSON: {:?}", user_json);
    let user = super::superx::deserialize_user(user_json).await?;

    println!("\n\n\nUser: ");
    print!("{:#?}", user);

    println!("\n\n\nas JSON:");
    let user_json = serde_json::to_string_pretty(&SortAlphabetically(&user))?;
    println!("{}", user_json);

    Ok(())
}
