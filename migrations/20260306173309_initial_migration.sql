CREATE TABLE publications (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    pub_date TIMESTAMPTZ NOT NULL,
    mastodon_id TEXT,
    mastodon_url TEXT,
    bluesky_id TEXT,
    bluesky_url TEXT
);
