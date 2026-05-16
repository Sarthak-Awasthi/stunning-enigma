-- Games supported by the manager (v1: Fallout 4 only)
CREATE TABLE IF NOT EXISTS games (
    id           INTEGER PRIMARY KEY,
    name         TEXT NOT NULL,
    canonical_id TEXT NOT NULL UNIQUE   -- e.g. "fallout4"
);

-- A specific installation of a game on disk
CREATE TABLE IF NOT EXISTS instances (
    id          INTEGER PRIMARY KEY,
    game_id     INTEGER NOT NULL REFERENCES games(id),
    label       TEXT NOT NULL,
    source_type TEXT NOT NULL,          -- "steam" | "gog" | "manual"
    install_path TEXT NOT NULL UNIQUE,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Profiles: isolated mod/plugin/config environments
CREATE TABLE IF NOT EXISTS profiles (
    id                INTEGER PRIMARY KEY,
    instance_id       INTEGER NOT NULL REFERENCES instances(id),
    name              TEXT NOT NULL,
    pinned_runner_id  INTEGER,           -- NULL = no runner pinned yet
    auto_deploy       INTEGER NOT NULL DEFAULT 0,  -- 0=false, 1=true
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(instance_id, name)
);

-- Global mod store: one row per installed mod archive
CREATE TABLE IF NOT EXISTS mods (
    id           INTEGER PRIMARY KEY,
    name         TEXT NOT NULL,
    version      TEXT,
    source_hash  TEXT NOT NULL UNIQUE,  -- SHA-256 of original archive
    install_path TEXT NOT NULL,         -- path inside managed mods dir
    installed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Per-profile mod state
CREATE TABLE IF NOT EXISTS profile_mods (
    profile_id  INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    mod_id      INTEGER NOT NULL REFERENCES mods(id)     ON DELETE CASCADE,
    enabled     INTEGER NOT NULL DEFAULT 1,
    priority    INTEGER NOT NULL,       -- lower number = lower priority
    PRIMARY KEY (profile_id, mod_id),
    UNIQUE(profile_id, priority)
);

-- Plugin metadata cache
CREATE TABLE IF NOT EXISTS plugins (
    id       INTEGER PRIMARY KEY,
    mod_id   INTEGER REFERENCES mods(id) ON DELETE SET NULL,
    filename TEXT NOT NULL UNIQUE,
    kind     TEXT NOT NULL              -- "esm" | "esp" | "esl"
);

-- Per-profile plugin state
CREATE TABLE IF NOT EXISTS profile_plugins (
    profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    plugin_id  INTEGER NOT NULL REFERENCES plugins(id)  ON DELETE CASCADE,
    enabled    INTEGER NOT NULL DEFAULT 1,
    load_index INTEGER NOT NULL,        -- position in load order
    PRIMARY KEY (profile_id, plugin_id),
    UNIQUE(profile_id, load_index)
);

-- Deploy manifests: immutable record of each deployment
CREATE TABLE IF NOT EXISTS deploy_manifests (
    id          INTEGER PRIMARY KEY,
    profile_id  INTEGER NOT NULL REFERENCES profiles(id),
    deployed_at TEXT NOT NULL DEFAULT (datetime('now')),
    symlink_plan TEXT NOT NULL,         -- JSON blob of {source -> target} map
    status      TEXT NOT NULL DEFAULT 'active'  -- "active" | "rolled_back"
);

-- File index: every file owned by every mod
CREATE TABLE IF NOT EXISTS file_index (
    id         INTEGER PRIMARY KEY,
    mod_id     INTEGER NOT NULL REFERENCES mods(id) ON DELETE CASCADE,
    rel_path   TEXT NOT NULL,           -- e.g. "Textures/foo.dds"
    is_ba2     INTEGER NOT NULL DEFAULT 0,
    UNIQUE(mod_id, rel_path)
);

-- Runner catalog: Proton and Wine runners
CREATE TABLE IF NOT EXISTS runner_catalog (
    id           INTEGER PRIMARY KEY,
    kind         TEXT NOT NULL,         -- "proton" | "wine"
    version      TEXT NOT NULL,
    source_url   TEXT,
    install_path TEXT NOT NULL,
    verified     INTEGER NOT NULL DEFAULT 0,
    installed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- F4SE state per instance
CREATE TABLE IF NOT EXISTS f4se_state (
    id           INTEGER PRIMARY KEY,
    instance_id  INTEGER NOT NULL REFERENCES instances(id) ON DELETE CASCADE UNIQUE,
    detected_version TEXT,
    compatible   INTEGER,               -- NULL=unchecked, 1=ok, 0=mismatch
    last_checked TEXT
);

-- Global settings key-value store
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Seed the one supported game
INSERT OR IGNORE INTO games (name, canonical_id)
VALUES ('Fallout 4', 'fallout4');