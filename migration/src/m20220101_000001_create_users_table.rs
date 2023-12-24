use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager.get_connection().execute_unprepared("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";").await?;

        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("uuid_generate_v4()")),
                    )
                    .col(
                        ColumnDef::new(User::DiscordSnowflake)
                            .big_integer()
                            .not_null()
                            .unique_key()
                    )
                    .col(
                        ColumnDef::new(User::MinecraftUuid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned()
            ).await?;

        manager
            .create_index(Index::create()
                .table(User::Table)
                .name("user_by_discord_snowflake")
                .col(User::DiscordSnowflake)
                .to_owned()
            ).await?;

        manager
            .create_index(Index::create()
                .table(User::Table)
                .name("user_by_minecraft_uuid")
                .col(User::MinecraftUuid)
                .to_owned()
            ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;

        manager
            .drop_index(Index::drop()
                .table(User::Table)
                .name("user_by_discord_snowflake")
                .to_owned()
            ).await?;

        manager
            .drop_index(Index::drop()
                .table(User::Table)
                .name("user_by_minecraft_uuid")
                .to_owned()
            ).await?;

        manager.get_connection().execute_unprepared("DROP EXTENSION IF EXISTS \"uuid-ossp\";").await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    #[sea_orm(iden = "discord_snowflake")]
    DiscordSnowflake,
    #[sea_orm(iden = "minecraft_uuid")]
    MinecraftUuid,
}
