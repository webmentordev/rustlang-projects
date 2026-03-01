# Instruction
Simple project to learn Actix Web and Postgres using SQLx crate.   
⚠️ Please run migration and start database before starting the server.  
  
## Start Postgres in Docker
```
docker compose up -d
```

## Install SQLX CLI (Just run once)
```
cargo install sqlx-cli --no-default-features --features postgres
```

## Create migration sql file
```
sqlx migrate add create_users
```

## Paste MySQL table
```
CREATE TABLE users (
    id    UUID PRIMARY KEY,
    name  TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE
);
```

## Run migration
```
sqlx migrate run
```