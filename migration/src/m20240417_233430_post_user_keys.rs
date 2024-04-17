use sea_orm_migration::prelude::*;

use crate::{m20220101_000001_post_table::Post, m20240417_230111_user_table::User};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Post::Table)
                    .add_foreign_key(
                        ForeignKey::create()
                            .name("fk_post_creator_user_id")
                            .from(Post::Table, Post::Creator)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .get_foreign_key(),
                    )
                    .add_foreign_key(
                        ForeignKey::create()
                            .name("fk_post_reblog_id")
                            .from(Post::Table, Post::ReblogId)
                            .to(Post::Table, Post::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .get_foreign_key(),
                    )
                    .add_foreign_key(
                        ForeignKey::create()
                            .name("fk_post_reply_id")
                            .from(Post::Table, Post::ReplyId)
                            .to(Post::Table, Post::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .get_foreign_key(),
                    )
                    .add_foreign_key(
                        ForeignKey::create()
                            .name("fk_post_quoting_id")
                            .from(Post::Table, Post::QuotingId)
                            .to(Post::Table, Post::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .get_foreign_key(),
                    )
                    .to_owned(),
            )
            .await
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
