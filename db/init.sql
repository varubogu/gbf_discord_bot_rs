CREATE USER gbf_bot_user WITH PASSWORD 'gbf_bot_password';
CREATE USER migration_user WITH PASSWORD 'migration_password';

ALTER USER gbf_bot_user WITH PASSWORD 'gbf_bot_password';
ALTER USER migration_user WITH PASSWORD 'migration_password';

CREATE DATABASE gbf_bot_db WITH OWNER gbf_bot_user;

GRANT CONNECT ON DATABASE gbf_bot_db TO gbf_bot_user;
GRANT CONNECT ON DATABASE gbf_bot_db TO migration_user;

\connect gbf_bot_db

GRANT USAGE ON SCHEMA public TO gbf_bot_user;
GRANT USAGE, CREATE ON SCHEMA public TO migration_user;

GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO gbf_bot_user;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO migration_user;

GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO gbf_bot_user;
GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO migration_user;

ALTER DEFAULT PRIVILEGES FOR USER migration_user IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO gbf_bot_user;
ALTER DEFAULT PRIVILEGES FOR USER migration_user IN SCHEMA public
    GRANT USAGE, SELECT, UPDATE ON SEQUENCES TO gbf_bot_user;
