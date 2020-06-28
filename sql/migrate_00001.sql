CREATE TABLE IF NOT EXISTS tosho (
    last_pub_date TIMESTAMP WITHOUT TIME ZONE
);

INSERT INTO tosho
SELECT (datetime('now', '-7 days'))
WHERE NOT EXISTS
(SELECT * FROM tosho);

CREATE TABLE IF NOT EXISTS shows (
    show_id BIGSERIAL PRIMARY KEY,
    "group" VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    quality VARCHAR
);

CREATE INDEX IF NOT EXISTS idx_shows_group_name_quality
ON shows (LOWER("group"), LOWER(name), LOWER(quality));

CREATE TABLE IF NOT EXISTS episodes (
    show_id BIGINT NOT NULL REFERENCES shows (show_id),
    episode INTEGER NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,

    link VARCHAR NOT NULL,
    grabbed BOOLEAN NOT NULL DEFAULT FALSE,
    grabbed_on TIMESTAMP WITHOUT TIME ZONE,

    PRIMARY KEY (show_id, episode)
);
