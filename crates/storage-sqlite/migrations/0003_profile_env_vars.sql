-- Profile-specific environment variables for launch configuration
CREATE TABLE IF NOT EXISTS profile_env_vars (
    profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    key        TEXT NOT NULL,
    value      TEXT NOT NULL,
    PRIMARY KEY (profile_id, key)
);
