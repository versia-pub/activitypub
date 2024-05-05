use activitypub_federation::{config::Data, fetch::object_id::ObjectId, traits::ActivityHandler};
use activitystreams_kinds::activity::FollowType;
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    database::StateHandle,
    entities::{follow_relation, prelude::FollowRelation, user},
    DB,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct Follow {
    actor: ObjectId<user::Model>,
    object: ObjectId<user::Model>,
    #[serde(rename = "type")]
    kind: FollowType,
    id: Url,
}

#[async_trait::async_trait]
impl ActivityHandler for Follow {
    type DataType = StateHandle;
    type Error = crate::error::Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let local_user = self.object.dereference(data).await?;
        let follower = self.actor.dereference(data).await?;
        save_follow(local_user, follower).await?;
        Ok(())
    }
}

async fn save_follow(
    local_user: user::Model,
    follower: user::Model,
) -> Result<(), crate::error::Error> {
    let url = Url::parse(&follower.url)?;
    let follow_relation = follow_relation::ActiveModel {
        followee_id: Set(local_user.id.clone()),
        follower_id: Set(follower.id.clone()),
        followee_host: Set(None),
        follower_host: Set(Some(url.host_str().unwrap().to_string())),
        followee_inbox: Set(Some(local_user.inbox.clone())),
        follower_inbox: Set(Some(follower.inbox.clone())),
        ..Default::default()
    };
    follow_relation.insert(DB.get().unwrap()).await?;
    Ok(())
}
