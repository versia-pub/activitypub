use super::entities::prelude::User;
use crate::{entities::user, error::Error, objects::person::DbUser, LOCAL_USER_NAME};
use anyhow::anyhow;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::{env, sync::{Arc, Mutex}};

#[derive(Debug, Clone)]
pub struct Config {}

#[derive(Debug, Clone)]
pub struct State {
    pub database_connection: Arc<DatabaseConnection>,
}

pub type StateHandle = State;

/// Our "database" which contains all known users (local and federated)
#[derive(Debug)]
pub struct Database {
    pub users: Mutex<Vec<DbUser>>,
}

impl State {
    pub async fn local_user(&self) -> Result<user::Model, Error> {
        let user = User::find()
            .filter(user::Column::Username.eq(env::var("LOCAL_USER_NAME").unwrap_or(LOCAL_USER_NAME.to_string())))
            .one(self.database_connection.as_ref())
            .await?
            .unwrap();
        Ok(user.clone())
    }

    pub async fn read_user(&self, name: &str) -> Result<user::Model, Error> {
        let db_user = self.local_user().await?;
        if name == db_user.username {
            Ok(db_user)
        } else {
            Err(anyhow!("Invalid user {name} // {0}", db_user.username).into())
        }
    }
}
