-- Initiales Schema. Vier Domain-Tabellen plus Migrations-Tracking
-- (schema_migrations wird vom Code separat angelegt).
--
-- Convention:
--   source     = Quellsystem ('ha', 'ccu', 'z2m', 'ha-addon', 'manual', ...)
--   source_id  = Stabile ID im Quellsystem (entity_id, CCU-Adresse, IEEE, slug)
--   (source, source_id) ist immer eindeutig.

CREATE TABLE devices (
    id              INTEGER PRIMARY KEY,
    source          TEXT NOT NULL,
    source_id       TEXT NOT NULL,
    name            TEXT NOT NULL,
    manufacturer    TEXT,
    model           TEXT,
    kind            TEXT,
    room            TEXT,
    last_seen       TIMESTAMP,
    first_seen      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    active          INTEGER NOT NULL DEFAULT 1,
    UNIQUE (source, source_id)
);

CREATE TABLE firmware_snapshot (
    id              INTEGER PRIMARY KEY,
    device_id       INTEGER NOT NULL,
    firmware        TEXT NOT NULL,
    observed_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (device_id) REFERENCES devices(id) ON DELETE CASCADE
);

CREATE INDEX firmware_snapshot_device_idx
    ON firmware_snapshot(device_id, observed_at);

CREATE TABLE software (
    id              INTEGER PRIMARY KEY,
    source          TEXT NOT NULL,
    source_id       TEXT NOT NULL,
    name            TEXT NOT NULL,
    version         TEXT NOT NULL,
    last_seen       TIMESTAMP,
    first_seen      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (source, source_id)
);

CREATE TABLE manual_meta (
    device_key      TEXT PRIMARY KEY,
    room            TEXT,
    purchased_at    DATE,
    purchase_price  REAL,
    warranty_until  DATE,
    manual_url      TEXT,
    notes           TEXT,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
