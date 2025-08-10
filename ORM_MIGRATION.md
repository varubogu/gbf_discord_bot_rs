# ORM Migration Guide

## Overview

This document provides guidance for migrating the current raw SQL queries to ORM-style operations in the GBF Discord Bot project. The current implementation uses sqlx with raw SQL queries, but as per the requirements, we should use ORM-style operations for database CRUD operations where possible.

## Implementation Status

The migration to SeaORM has been completed. The following changes have been made:

1. **Entity Models**: Entity models have been created for all database tables in the `src/models/entities/` directory:
   - `quest.rs`
   - `quest_alias.rs`
   - `battle_recruitment.rs`
   - `environment.rs`
   - `message_text.rs`

2. **Database Operations**: All database operations have been refactored to use SeaORM:
   - `src/models/quest.rs`
   - `src/models/environment.rs`
   - `src/models/battle_recruitment.rs`
   - `src/models/message_text.rs`

3. **Database Connection**: The `Database` struct now uses SeaORM's `DatabaseConnection` instead of sqlx's `PgPool`.

4. **Service Layer Integration**: The `src/services/database/database.rs` file has been refactored to use the `Database` struct from `src/models/database.rs` instead of implementing the database operations directly with raw SQL queries. This ensures that all database operations throughout the application use SeaORM.

## SeaORM Implementation Details

### Entity Models

Entity models are defined using SeaORM's derive macros:

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "quests")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub target_id: i32,
    pub quest_name: String,
    pub default_battle_type: i32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
```

### Relationships

Relationships between entities are defined using the `Relation` enum and `Related` trait:

```rust
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::quest_alias::Entity")]
    QuestAlias,
}

impl Related<super::quest_alias::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::QuestAlias.def()
    }
}
```

### SELECT Operations

Example of SELECT operations with SeaORM:

```
// Find all quests
let models = QuestEntity::find()
    .all(&self.conn)
    .await?;
    
Ok(models.into_iter().map(|model| model.into()).collect())

// Find by condition
let quest = QuestEntity::find()
    .filter(quest::Column::TargetId.eq(target_id))
    .one(&self.conn)
    .await?;
    
Ok(quest.map(|q| q.into()))
```

### INSERT Operations

Example of INSERT operations with SeaORM:

```
// Create a new record
let battle_recruitment = battle_recruitment::ActiveModel {
    guild_id: Set(guild_id),
    channel_id: Set(channel_id),
    message_id: Set(message_id),
    target_id: Set(target_id),
    battle_type_id: Set(battle_type_id),
    expiry_date: Set(expiry_date),
    ..Default::default()
};

let result = battle_recruitment.insert(&self.conn).await?;
```

### UPDATE Operations

Example of UPDATE operations with SeaORM:

```
// Update an existing record
let mut active_model: battle_recruitment::ActiveModel = recruitment.into();
active_model.recruit_end_message_id = Set(Some(message_id));
active_model.update(&self.conn).await?;
```

### Transaction Support

SeaORM provides transaction support:

```
// Start a transaction
let txn = self.conn.begin().await?;

// Perform operations within the transaction
let result = if let Some(existing) = existing {
    // Update existing
    let mut active_model: environment::ActiveModel = existing.into_active_model();
    active_model.value = Set(value.to_string());
    active_model.update(&txn).await?
} else {
    // Create new
    let active_model = environment::ActiveModel {
        key: Set(key.to_string()),
        value: Set(value.to_string()),
        ..Default::default()
    };
    active_model.insert(&txn).await?
};

// Commit the transaction
txn.commit().await?;
```

## Operations Kept as SQL

Some operations might still be more efficient or clearer when written in SQL. These include:

- Complex queries with multiple joins and conditions
- TRUNCATE operations
- Batch operations
- Performance-critical queries

## Testing

A test script has been created at `src/bin/test_orm.rs` to verify the refactored operations. This script tests all the database operations that were refactored to use SeaORM.

## Conclusion

The migration to SeaORM has been completed successfully. The codebase is now more maintainable and less error-prone. It will also be easier to change the database schema in the future.