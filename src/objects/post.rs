use crate::{
    activities::create_post::CreatePost,
    database::StateHandle,
    entities::{post, user},
    error::Error,
    lysand::conversion::db_user_from_url,
    objects::person::DbUser,
    utils::generate_object_id,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{object::NoteType, public},
    protocol::{helpers::deserialize_one_or_many, verification::verify_domains_match},
    traits::{Actor, Object},
};
use activitystreams_kinds::link::MentionType;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use tracing::info;
use url::Url;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbPost {
    pub text: String,
    pub ap_id: ObjectId<post::Model>,
    pub creator: ObjectId<user::Model>,
    pub local: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    #[serde(rename = "type")]
    pub(crate) kind: NoteType,
    pub(crate) id: ObjectId<post::Model>,
    pub(crate) attributed_to: ObjectId<user::Model>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) content: String,
    pub(crate) in_reply_to: Option<ObjectId<post::Model>>,
    pub(crate) tag: Vec<Mention>,
    pub(crate) sensitive: Option<bool>,
    pub(crate) cc: Option<Vec<Url>>,
}

impl Note {
    pub fn from_db(post: &post::Model) -> Self {
        serde_json::from_str(&post.ap_json.as_ref().unwrap()).unwrap()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Mention {
    pub href: Url,
    #[serde(rename = "type")]
    pub kind: MentionType,
}

#[async_trait::async_trait]
impl Object for post::Model {
    type DataType = StateHandle;
    type Kind = Note;
    type Error = Error;

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        let post = crate::entities::prelude::Post::find()
            .filter(post::Column::Id.eq(object_id.to_string()))
            .one(data.app_data().database_connection.clone().as_ref())
            .await;
        Ok(post.unwrap())
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let creator = db_user_from_url(Url::parse(self.creator.as_str()).unwrap()).await?;
        let to = match self.visibility.as_str() {
            "public" => vec![
                public(),
                Url::parse(creator.followers.unwrap().as_str()).unwrap(),
            ],
            "followers" => vec![Url::parse(creator.followers.unwrap().as_str()).unwrap()],
            "direct" => vec![], //TODO: implement this
            "unlisted" => vec![
                Url::parse(creator.followers.unwrap().as_str()).unwrap(),
                public(),
            ],
            _ => vec![public()],
        };
        Ok(Note {
            kind: Default::default(),
            id: Url::parse(self.url.as_str()).unwrap().into(),
            attributed_to: Url::parse(self.creator.as_str()).unwrap().into(),
            to: to.clone(),
            content: self.content,
            in_reply_to: None,
            tag: vec![],
            sensitive: Some(self.sensitive),
            cc: Some(to),
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)?;
        Ok(())
    }

    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        println!(
            "Received post with content {} and id {}",
            &json.content, &json.id
        );
        let creator = json.attributed_to.dereference(data).await?;
        let post: post::ActiveModel = post::ActiveModel {
            content: Set(json.content.clone()),
            id: Set(Uuid::now_v7().to_string()),
            creator: Set(creator.id.to_string()),
            created_at: Set(chrono::Utc::now()), //TODO: make this use the real timestamp
            content_type: Set("text/plain".to_string()), // TODO: make this use the real content type
            local: Set(false),
            visibility: Set("public".to_string()), // TODO: make this use the real visibility
            sensitive: Set(json.sensitive.clone().unwrap_or_default()),
            url: Set(json.id.clone().to_string()),
            ap_json: Set(Some(serde_json::to_string(&json).unwrap())),
            ..Default::default()
        };
        let post = post
            .insert(data.app_data().database_connection.clone().as_ref())
            .await;

        if let Err(err) = post {
            eprintln!("Error inserting post: {:?}", err);
            return Err(err.into());
        }
        info!("Post inserted: {:?}", post.as_ref().unwrap());

        Ok(post.unwrap())
    }
}
