use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Post {
    Table,
    Id,
    Url,
    Creator,
    Title,
    Content,
    Local,
    CreatedAt,
    UpdatedAt,
    ReblogId,
    ContentType,
    Visibility,
    ReplyId,
    QuotingId,
    Sensitive,
    SpoilerText,
}
