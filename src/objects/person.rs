use crate::{
    activities::{
        create_post::CreatePost,
        follow::{self, Follow},
    },
    database::{State, StateHandle},
    entities::{self, user},
    error::Error,
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
    kind: PersonType,
    preferred_username: String,
    name: String,
    summary: Option<String>,
    url: Url,
    id: ObjectId<user::Model>,
    inbox: Url,
    public_key: PublicKey,
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
        let res = entities::prelude::User::find()
            .filter(entities::user::Column::Id.eq(object_id.as_str()))
            .one(data.database_connection.as_ref())
            .await?;
        Ok(res)
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        Ok(Person {
            preferred_username: self.username.clone(),
            kind: Default::default(),
            id: Url::parse(&self.id).unwrap().into(),
            inbox: Url::parse(&self.inbox).unwrap(),
            public_key: self.public_key(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            url: Url::parse(&self.url).unwrap(),
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
        Url::parse(&self.id).unwrap()
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
