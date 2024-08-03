use crate::{
    activities::{
        create_post::CreatePost,
        follow::{self, Follow},
    },
    database::{State, StateHandle},
    entities::{self, user},
    error::Error,
    API_DOMAIN,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    http_signatures::generate_actor_keypair,
    kinds::actor::PersonType,
    protocol::{public_key::PublicKey, verification::verify_domains_match},
    traits::{ActivityHandler, Actor, Object},
};
use actix_web::http::header::Accept;
use chrono::{prelude, DateTime, Utc};
use entities::prelude::User;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tracing::info;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DbUser {
    pub name: String,
    pub ap_id: ObjectId<user::Model>,
    pub inbox: Url,
    // exists for all users (necessary to verify http signatures)
    pub public_key: String,
    // exists only for local users
    pub private_key: Option<String>,
    last_refreshed_at: DateTime<Utc>,
    pub followers: Vec<Url>,
    pub local: bool,
}

/// List of all activities which this actor can receive.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonAcceptedActivities {
    CreateNote(CreatePost),
    Follow(Follow),
    Accept(follow::Accept),
}

impl DbUser {
    pub fn new(hostname: &str, name: &str) -> Result<DbUser, Error> {
        let ap_id = Url::parse(&format!("https://{}/{}", hostname, &name))?.into();
        let inbox = Url::parse(&format!("https://{}/{}/inbox", hostname, &name))?;
        let keypair = generate_actor_keypair()?;
        Ok(DbUser {
            name: name.to_string(),
            ap_id,
            inbox,
            public_key: keypair.public_key,
            private_key: Some(keypair.private_key),
            last_refreshed_at: Utc::now(),
            followers: vec![],
            local: true,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(rename = "type")]
    pub kind: PersonType,
    pub preferred_username: String,
    pub name: String,
    pub summary: Option<String>,
    pub url: Url,
    pub id: ObjectId<user::Model>,
    pub inbox: Url,
    pub public_key: PublicKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manually_approves_followers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followers: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub following: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<EndpointType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outbox: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured_tags: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<Vec<TagType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<IconType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<IconType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<Vec<AttachmentType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub also_known_as: Option<Vec<Url>>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TagType {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<Url>,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<IconType>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointType {
    pub shared_inbox: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IconType {
    #[serde(rename = "type")]
    pub type_: String, //Always "Image"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    pub url: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AttachmentType {
    #[serde(rename = "type")]
    pub type_: String, //Always "PropertyValue"
    pub name: String,
    pub value: String,
}

#[async_trait::async_trait]
impl Object for user::Model {
    type DataType = StateHandle;
    type Kind = Person;
    type Error = Error;

    fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        Some(self.last_refreshed_at)
    }

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        println!("!!!!!!!!Reading user from id!!!!!!!!!!!: {}", object_id);
        let res = entities::prelude::User::find()
            .filter(entities::user::Column::Url.eq(object_id.to_string()))
            .one(data.database_connection.as_ref())
            .await?;
        println!("!!!!!!!!Reading user from id!!!!!!!!!!!: {}", res.clone().is_some());
        Ok(res)
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let serialized = serde_json::from_str(self.ap_json.as_ref().unwrap().as_str())?;
        Ok(serialized)
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)?;
        Ok(())
    }

    async fn from_json(
        json: Self::Kind,
        _data: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        let query = User::find()
            .filter(user::Column::Id.eq(json.id.inner().as_str()))
            .one(_data.database_connection.as_ref())
            .await?;
        if let Some(user) = query {
            return Ok(user);
        }
        let copied_json = json.clone();
        let model = user::ActiveModel {
            id: Set(Uuid::now_v7().to_string()),
            username: Set(json.preferred_username),
            name: Set(json.name),
            inbox: Set(json.inbox.to_string()),
            public_key: Set(json.public_key.public_key_pem),
            local: Set(false),
            summary: Set(json.summary),
            url: Set(json.id.to_string()),
            follower_count: Set(0),
            following_count: Set(0),
            created_at: Set(Utc::now()),
            last_refreshed_at: Set(Utc::now()),
            ap_json: Set(Some(serde_json::to_string(&copied_json).unwrap())),
            ..Default::default()
        };
        let model = model.insert(_data.database_connection.as_ref()).await;
        if let Err(err) = model {
            eprintln!("Error inserting user: {:?}", err);
            Err(err.into())
        } else {
            info!("User inserted: {:?}", model.as_ref().unwrap());
            Ok(model.unwrap())
        }
    }
}

impl Actor for user::Model {
    fn id(&self) -> Url {
        Url::parse(&format!(
            "https://{}/apbridge/user/{}",
            API_DOMAIN.to_string(),
            &self.id
        ))
        .unwrap()
    }

    fn public_key_pem(&self) -> &str {
        &self.public_key
    }

    fn private_key_pem(&self) -> Option<String> {
        self.private_key.clone()
    }

    fn inbox(&self) -> Url {
        Url::parse(&self.inbox).unwrap()
    }

    //TODO: Differenciate shared inbox
    fn shared_inbox(&self) -> Option<Url> {
        None
    }
}
