use sea_orm_migration::prelude::*;

use crate::m20240417_230111_user_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(FollowRelation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FollowRelation::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(FollowRelation::FolloweeId).string().not_null())
                    .col(ColumnDef::new(FollowRelation::FollowerId).string().not_null())
                    .col(ColumnDef::new(FollowRelation::FolloweeHost).string())
                    .col(ColumnDef::new(FollowRelation::FollowerHost).string())
                    .col(ColumnDef::new(FollowRelation::FolloweeInbox).string())
                    .col(ColumnDef::new(FollowRelation::FollowerInbox).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_follow_relation_followee_id")
                            .from(FollowRelation::Table, FollowRelation::FolloweeId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_follow_relation_follower_id")
                            .from(FollowRelation::Table, FollowRelation::FollowerId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FollowRelation::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum FollowRelation {
    Table,
    Id,
    FolloweeId,
    FollowerId,
    FolloweeHost,
    FollowerHost,
    FolloweeInbox,
    FollowerInbox
}
