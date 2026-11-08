-- The opaque bearer credential is an access-session token, not a refresh token.
-- Rename the digest column so the schema accurately describes what is stored.
ALTER TABLE auth_sessions RENAME COLUMN refresh_token_hash TO token_hash;
ALTER TABLE auth_sessions RENAME CONSTRAINT auth_sessions_refresh_token_hash_key TO auth_sessions_token_hash_key;
