use crate::{
    database::StateHandle,
    entities::{self, post, user},
    error::Error,
    objects::{
        person::DbUser,
        post::{DbPost, Note},
    },
    utils::{base_url_encode, generate_create_id, generate_random_object_id}, versia::{conversion::versia_post_from_db, objects::SortAlphabetically, superx::request_client}, DB,
};
use activitypub_federation::{
    activity_sending::SendActivityTask,
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::CreateType,
    protocol::{context::WithContext, helpers::deserialize_one_or_many},
    traits::{ActivityHandler, Object},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreatePost {
    pub(crate) actor: ObjectId<user::Model>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: Note,
    #[serde(rename = "type")]
    pub(crate) kind: CreateType,
    pub(crate) id: Url,
}

impl CreatePost {
    pub async fn send(
        note: Note,
        db_entry: post::Model,
        inbox: Url,
        data: &Data<StateHandle>,
    ) -> Result<(), Error> {
        print!("Sending reply to {}", &note.attributed_to);
        let encoded_url = base_url_encode(&note.id.clone().into());
        let create = CreatePost {
            actor: note.attributed_to.clone(),
            to: note.to.clone(),
            object: note,
            kind: CreateType::Create,
            id: generate_create_id(data.domain(), &db_entry.id, &encoded_url)?,
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
impl ActivityHandler for CreatePost {
    type DataType = StateHandle;
    type Error = crate::error::Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        post::Model::verify(&self.object, &self.id, data).await?;
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let note = post::Model::from_json(self.object, data).await?;
        federate_inbox(note).await?;
        Ok(())
    }
}

async fn federate_inbox(note: crate::entities::post::Model) -> anyhow::Result<()> {
    let versia_post = versia_post_from_db(note.clone()).await?;
    let json = serde_json::to_string_pretty(&SortAlphabetically(&versia_post))?;

    let mut array;
    if versia_post.mentions.is_some() {
        array = versia_post.mentions.clone().unwrap();
    } else {
        array = Vec::new();
    }

    let db = DB.get().unwrap();

    let list_model = entities::prelude::FollowRelation::find()
            .filter(entities::follow_relation::Column::FolloweeId.eq(note.creator.to_string()))
            .all(db)
            .await?;

    let mut list_url = Vec::new();

    for model in list_model {
        let url = Url::parse(&model.follower_inbox.unwrap())?;
        list_url.push(url);
    }

    array.append(&mut list_url);

    let req_client = request_client();
    for inbox in array {
        let push = req_client.post(inbox)
            .json(&json)
            .send();
        tokio::spawn(push);
    }

    Ok(())
}