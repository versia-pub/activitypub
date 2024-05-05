use crate::{
    activities::create_post::CreatePost,
    database::StateHandle,
    entities::{post, user},
    error::Error,
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
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use tracing::info;
use url::Url;

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
    pub(crate) sensitive: bool,
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
        _object_id: Url,
        _data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        Ok(None)
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        todo!()
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
            id: Set(json.id.to_string()),
            creator: Set(creator.id.to_string()),
            created_at: Set(chrono::Utc::now()), //TODO: make this use the real timestamp
            content_type: Set("text/plain".to_string()), // TODO: make this use the real content type
            local: Set(false),
            visibility: Set("public".to_string()), // TODO: make this use the real visibility
            sensitive: Set(json.sensitive.clone()),
            url: Set(json.id.clone().to_string()),
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

        let post = post.unwrap();

        let mention = Mention {
            href: Url::parse(&creator.id)?,
            kind: Default::default(),
        };
        let id: ObjectId<post::Model> = generate_object_id(data.domain())?.into();
        let note = Note {
            kind: Default::default(),
            id,
            sensitive: false,
            attributed_to: Url::parse(&data.local_user().await?.id).unwrap().into(),
            to: vec![public()],
            content: format!("Hello {}", creator.name),
            in_reply_to: Some(json.id.clone()),
            tag: vec![mention],
        };
        CreatePost::send(note, creator.shared_inbox_or_inbox(), data).await?;

        Ok(post)
    }
}
