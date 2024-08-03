//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.10

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "follow_relation")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub followee_id: String,
    pub follower_id: String,
    pub followee_host: Option<String>,
    pub follower_host: Option<String>,
    pub followee_inbox: Option<String>,
    pub follower_inbox: Option<String>,
    pub accept_id: Option<String>,
    pub ap_id: Option<String>,
    pub ap_accept_id: Option<String>,
    pub remote: bool,
    pub ap_json: String,
    pub ap_accept_json: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::FollowerId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User2,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::FolloweeId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User1,
}

impl ActiveModelBehavior for ActiveModel {}
