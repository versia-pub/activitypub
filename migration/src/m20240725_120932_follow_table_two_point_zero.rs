use sea_orm_migration::prelude::*;

use crate::m20240417_230111_user_table::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .drop_table(Table::drop().table(FollowRelation::Table).to_owned())
            .await;
        manager
            .create_table(
                Table::create()
                    .table(FollowRelation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FollowRelation::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(FollowRelation::FolloweeId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(FollowRelation::FollowerId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(FollowRelation::FolloweeHost).string())
                    .col(ColumnDef::new(FollowRelation::FollowerHost).string())
                    .col(ColumnDef::new(FollowRelation::FolloweeInbox).string())
                    .col(ColumnDef::new(FollowRelation::FollowerInbox).string())
                    .col(ColumnDef::new(FollowRelation::AcceptId).string())
                    .col(ColumnDef::new(FollowRelation::ApId).string())
                    .col(ColumnDef::new(FollowRelation::ApAcceptId).string())
                    .col(ColumnDef::new(FollowRelation::Remote).boolean().not_null())
                    .col(ColumnDef::new(FollowRelation::ApJson).string().not_null())
                    .col(ColumnDef::new(FollowRelation::ApAcceptJson).string())
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
    AcceptId,
    Remote, // true if initial Follow came from Remote
    ApId,
    ApAcceptId,
    FolloweeId,
    FollowerId,
    FolloweeHost,
    FollowerHost,
    FolloweeInbox,
    FollowerInbox,
    ApJson,
    ApAcceptJson,
}
