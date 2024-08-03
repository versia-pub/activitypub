use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use time::OffsetDateTime;
use url::Url;

use crate::{
    entities::{follow_relation, prelude, user},
    utils::generate_follow_accept_id,
    API_DOMAIN, DB,
};

use super::{
    conversion::{fetch_user_from_url, lysand_user_from_db},
    objects::FollowResult,
    superx::request_client,
};

async fn send_follow_accept_to_lysand(model: follow_relation::Model) -> anyhow::Result<()> {
    let request_client = request_client();
    let db = DB.get().unwrap();

    let id_raw = model.accept_id.unwrap();
    let id = uuid::Uuid::parse_str(&id_raw)?;
    let uri = generate_follow_accept_id(API_DOMAIN.as_str(), &id_raw)?;

    let follower_model = prelude::User::find()
        .filter(user::Column::Id.eq(model.follower_id))
        .one(db)
        .await?
        .unwrap();
    let lysand_follower = fetch_user_from_url(Url::parse(&follower_model.url)?).await?;

    let followee_model = prelude::User::find()
        .filter(user::Column::Id.eq(model.followee_id))
        .one(db)
        .await?
        .unwrap();
    let lysand_followee = lysand_user_from_db(followee_model).await?;

    let entity = FollowResult {
        rtype: super::objects::LysandType::FollowAccept,
        id,
        uri,
        created_at: OffsetDateTime::now_utc(),
        author: lysand_followee.uri,
        follower: lysand_follower.uri,
    };

    let request = request_client
        .post(lysand_follower.inbox.as_str())
        .header("Content-Type", "application/json; charset=utf-8")
        .header("Accept", "application/json")
        .header("Date", entity.created_at.clone().to_string())
        .json(&entity);

    let response = request.send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to send follow accept to Lysand"))
    }
}
