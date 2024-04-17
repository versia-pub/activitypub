use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(User::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(User::Username).string().not_null())
                    .col(ColumnDef::new(User::Name).string().not_null())
                    .col(ColumnDef::new(User::Summary).string())
                    .col(ColumnDef::new(User::Url).string().not_null())
                    .col(ColumnDef::new(User::PublicKey).string().not_null())
                    .col(ColumnDef::new(User::PrivateKey).string())
                    .col(ColumnDef::new(User::LastRefreshedAt).timestamp().not_null())
                    .col(ColumnDef::new(User::Local).boolean().not_null())
                    .col(ColumnDef::new(User::FollowerCount).integer().not_null())
                    .col(ColumnDef::new(User::FollowingCount).integer().not_null())
                    .col(ColumnDef::new(User::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(User::UpdatedAt).timestamp())
                    .col(ColumnDef::new(User::Following).string())
                    .col(ColumnDef::new(User::Followers).string())
                    .col(ColumnDef::new(User::Inbox).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Username,
    Name,
    Summary,
    Url,
    PublicKey,
    PrivateKey,
    LastRefreshedAt,
    Local,
    FollowerCount,
    FollowingCount,
    CreatedAt,
    UpdatedAt,
    Following,
    Followers,
    Inbox,
}
