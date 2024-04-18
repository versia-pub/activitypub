use crate::{entities::user, error::Error, objects::person::DbUser};
use anyhow::anyhow;
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use super::entities::prelude::User;

#[derive(Debug, Clone)]
pub struct Config {}

#[derive(Debug, Clone)]
pub struct State {
    pub database_connection: Arc<DatabaseConnection>,
    pub config: Arc<Config>,
}

pub type StateHandle = Arc<State>;

/// Our "database" which contains all known users (local and federated)
#[derive(Debug)]
pub struct Database {
    pub users: Mutex<Vec<DbUser>>,
}

impl State {
    pub async fn local_user(&self) -> Result<user::Model, Error> {
        let user = User::find().one(self.database_connection.as_ref()).await?.unwrap();
        Ok(user.clone())
    }

    pub async fn read_user(&self, name: &str) -> Result<user::Model, Error> {
        let db_user = self.local_user().await?;
        if name == db_user.username {
            Ok(db_user)
        } else {
            Err(anyhow!("Invalid user {name}").into())
        }
    }
}
