use sea_orm_migration::prelude::*;

use crate::{m20220101_000001_post_table::Post, m20240417_230111_user_table::User};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Post::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Post::Title).string())
                    .col(ColumnDef::new(Post::Content).string().not_null())
                    .col(ColumnDef::new(Post::Local).boolean().not_null())
                    .col(ColumnDef::new(Post::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Post::UpdatedAt).timestamp())
                    .col(ColumnDef::new(Post::ReblogId).string())
                    .col(ColumnDef::new(Post::ContentType).string().not_null())
                    .col(ColumnDef::new(Post::Visibility).string().not_null())
                    .col(ColumnDef::new(Post::ReplyId).string())
                    .col(ColumnDef::new(Post::QuotingId).string())
                    .col(ColumnDef::new(Post::Sensitive).boolean().not_null())
                    .col(ColumnDef::new(Post::SpoilerText).string())
                    .col(ColumnDef::new(Post::Creator).string().not_null())
                    .col(ColumnDef::new(Post::Url).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_creator_user_id")
                            .from(Post::Table, Post::Creator)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_reblog_id")
                            .from(Post::Table, Post::ReblogId)
                            .to(Post::Table, Post::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_reply_id")
                            .from(Post::Table, Post::ReplyId)
                            .to(Post::Table, Post::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_quoting_id")
                            .from(Post::Table, Post::QuotingId)
                            .to(Post::Table, Post::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_post_creator_user_id")
                    .table(Post::Table)
                    .to_owned(),
            )
            .await
    }
}
