-- Add down migration script here
ALTER TABLE todos DROP CONSTRAINT user_fk;
ALTER TABLE todos DROP COLUMN user_id;
ALTER TABLE todos DROP COLUMN created_at;