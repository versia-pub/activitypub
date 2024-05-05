use activitypub_federation::{
    activity_sending::SendActivityTask,
    config::Data,
    fetch::object_id::ObjectId,
    protocol::context::WithContext,
    traits::{ActivityHandler, Actor, Object},
};
use activitystreams_kinds::activity::{AcceptType, FollowType};
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    database::StateHandle,
    entities::{follow_relation, post, user},
    error,
    utils::{generate_follow_accept_id, generate_object_id},
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

impl Follow {
    pub async fn send(
        local_user: ObjectId<user::Model>,
        followee: ObjectId<user::Model>,
        inbox: Url,
        data: &Data<StateHandle>,
    ) -> Result<(), error::Error> {
        print!("Sending follow request to {}", &followee);
        let create = Follow {
            actor: local_user.clone(),
            object: followee.clone(),
            kind: FollowType::Follow,
            id: generate_object_id(data.domain())?,
        };
        let create_with_context = WithContext::new_default(create);
        let sends = SendActivityTask::prepare(
            &create_with_context,
            &data.local_user().await?,
            vec![inbox],
            data,
        )
        .await?;
        for send in sends {
            send.sign_and_send(data).await?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Accept {
    actor: ObjectId<user::Model>,
    object: Follow,
    #[serde(rename = "type")]
    kind: AcceptType,
    id: Url,
}

impl Accept {
    pub async fn send(
        follow_relation: follow_relation::Model,
        follow_req: Follow,
        inbox: Url,
        data: &Data<StateHandle>,
    ) -> Result<(), error::Error> {
        print!("Sending accept to {}", &follow_relation.follower_id);
        let create = Accept {
            actor: follow_req.object.clone(),
            object: follow_req,
            kind: AcceptType::Accept,
            id: generate_follow_accept_id(data.domain(), follow_relation.id)?,
        };
        let create_with_context = WithContext::new_default(create);
        let sends = SendActivityTask::prepare(
            &create_with_context,
            &data.local_user().await?,
            vec![inbox],
            data,
        )
        .await?;
        for send in sends {
            send.sign_and_send(data).await?;
        }
        Ok(())
    }
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
        accept_follow(self, data).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Accept {
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
        let user = self.actor.dereference(data).await?;
        let follower = self.object.actor.dereference(data).await?;
        save_follow(user, follower).await?;
        Ok(())
    }
}

async fn accept_follow(
    follow_req: Follow,
    data: &Data<StateHandle>,
) -> Result<(), crate::error::Error> {
    let local_user = follow_req.actor.dereference(data).await?;
    let follower = follow_req.object.dereference(data).await?;
    let follow_relation = save_follow(local_user, follower.clone()).await?;
    Accept::send(follow_relation, follow_req, follower.inbox().clone(), data).await?;
    Ok(())
}

async fn save_follow(
    local_user: user::Model,
    follower: user::Model,
) -> Result<follow_relation::Model, crate::error::Error> {
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
    let model = follow_relation.insert(DB.get().unwrap()).await?;
    Ok(model)
}
