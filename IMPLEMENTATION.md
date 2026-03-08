# Implementation

This document describes how Comms Relay is built internally. For usage instructions, see [README.md](README.md).

---

## Architecture

One Rust crate (`comms-relay`) with three elements, two binaries and one library:

```
comms-relay/
├── src/lib/          → comms (lib)   - shared types
├── src/bin/relay/    → relay (bin)   - HTTP server
└── src/bin/uplink/   → uplink (bin)  - CLI client
```

The library (`comms`) exists solely to share the `Publication` domain type and the HTTP payload wrappers between the two binaries without duplication. Both binaries depend on it; neither binary depends on the other.

---

## Data model

The central type is `Publication` (`src/lib/publication.rs`):

| Field | Type | Notes |
|---|---|---|
| `id` | `Ulid` | Sortable, URL-safe, collision-resistant identifier |
| `content` | `String` | The text body of the publication |
| `pub_date` | `jiff::Zoned` | UTC timestamp of creation |
| `mastodon_id` | `Option<String>` | Set after successful Mastodon post |
| `mastodon_url` | `Option<String>` | Set after successful Mastodon post |
| `bluesky_id` | `Option<String>` | Set after successful Bluesky post |
| `bluesky_url` | `Option<String>` | Set after successful Bluesky post |

All fields are private; access goes through getter methods. The social platform fields are populated after a successful post to each platform.

**Database schema** (`migrations/20260306173309_initial_migration.sql`):

```sql
CREATE TABLE publications (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    pub_date TIMESTAMPTZ NOT NULL,
    mastodon_id TEXT,
    mastodon_url TEXT,
    bluesky_id TEXT,
    bluesky_url TEXT
);
```

The ULID is stored as `TEXT`. Timestamps are stored as `TIMESTAMPTZ`.

---

## Relay internals

### Router and AppState

`src/bin/relay/app.rs` defines the Axum router with four routes:

| Method | Path | Auth | Handler |
|---|---|---|---|
| `GET` | `/publications` | no | `list_publications` |
| `POST` | `/publications` | yes | `post_publication` |
| `GET` | `/publications/{id}` | no | `get_publication` |
| `DELETE` | `/publications/{id}` | yes | `delete_publication` |

`AppState` holds a `PgStorage` instance, the `api_token` string, and the five social-platform credential fields (`mastodon_access_token`, `mastodon_instance_url`, `bluesky_instance_url`, `bluesky_identifier`, `bluesky_app_password`). It is wrapped in `Arc` and shared across all handlers.

### Authentication

`src/bin/relay/auth.rs` implements `BearerAuth`, a custom `FromRequestParts` extractor. It parses the `Authorization: Bearer <token>` header and compares it to `AppState.api_token` in constant time. Returns `401 Unauthorized` if the header is absent, malformed, or invalid. Both `POST /publications` and `DELETE /publications/{id}` require it.

### Social posting

`src/bin/relay/mastodon.rs` provides `MastodonClient`. Its `post()` method sends the publication content to `POST {instance_url}/api/v1/statuses` as multipart form data, with an `Idempotency-Key` header (a fresh ULID per request) and bearer auth. Returns `MastodonStatus { id, url, uri, created_at }` from the `comms` lib.

`src/bin/relay/bluesky.rs` provides `BlueskyClient`. Its `post()` method is two-step: it authenticates via `createSession` (receiving a DID and access JWT), then creates a post record via `createRecord`. The public URL is constructed from the AT URI returned by the API (`https://bsky.app/profile/{handle}/post/{rkey}`), by extracting the rkey from the last segment of the URI. Returns `BlueskyStatus { uri, url }` from the `comms` lib.

Both clients are instantiated in `AppState` from the environment variables loaded at startup.

### Storage and PgZoned

`src/bin/relay/storage.rs` wraps a `sqlx::PgPool`. All queries use `sqlx::query!()` macros, which are checked against the live database schema at compile time. Public methods: `create`, `insert_publication`, `get_publication`, `list_publications`, and `delete_publication`. `delete_publication` returns `Result<bool>` (`true` if a row was deleted, `false` if the ID was not found).

`jiff::Zoned` has no native sqlx codec, so `PgZoned` is a newtype that implements `sqlx::Encode` and `sqlx::Decode` by converting to/from a microsecond-precision Unix timestamp, the representation Postgres uses internally for `TIMESTAMPTZ`.

### Migrations

`sqlx::migrate!()` is called in `PgStorage::create()`, running any pending migrations from the `migrations/` directory before the server begins accepting connections. No manual migration step is needed.

### Telemetry

`src/bin/relay/telemetry.rs` initialises a `tracing_subscriber` filtered by `RUST_LOG` (default: `debug`). A `tower-http` `TraceLayer` adds an INFO-level span to every request with the method, URI, matched route, and response status. 5xx responses are logged at ERROR level, 4xx at WARN.

---

## Uplink internals

### Module layout

```
src/bin/uplink/
├── main.rs        - tokio entrypoint, parses Cli and calls run()
├── cli.rs         - Cli struct (clap Parser), global flags, Command enum, run()
├── cli/
│   ├── config.rs  - uplink config subcommand
│   ├── publish.rs - uplink publish subcommand
│   ├── list.rs    - uplink list subcommand
│   ├── get.rs     - uplink get subcommand
│   └── delete.rs  - uplink delete subcommand
├── actions.rs     - stateless async HTTP functions (reqwest)
├── config.rs      - AppConfig: load/save ~/.config/uplink/config.toml
└── display.rs     - colored terminal output functions
```

### Configuration resolution

`Cli::run()` in `cli.rs` resolves the effective URL and token before dispatching to a subcommand:

```
CLI flag (--url / --token)
  ↓ fallback
Environment variable (RELAY_URL / RELAY_API_TOKEN)     ← handled by clap's env= attribute
  ↓ fallback
Config file (~/.config/uplink/config.toml)
  ↓ fallback (URL only)
Built-in default: http://localhost:8000
```

The token has no built-in default. If `publish` is invoked without a token from any source, the command fails with a clear message pointing to `uplink config --token`.

The config file path is resolved via the `directories` crate (`ProjectDirs::from("dev", "yorch", "uplink")`), which follows XDG on Linux and platform conventions on macOS and Windows.

### Display module

`display.rs` provides three functions called by the subcommands:

| Function | Used by |
|---|---|
| `print_publications` | `list` |
| `print_publication` | `get` |
| `print_publish_success` | `publish` |
| `print_delete_success` | `delete` |

Colors are applied with `owo-colors` using `if_supports_color(Stream::Stdout, ...)`, which automatically suppresses ANSI codes when stdout is not a TTY or when `NO_COLOR` is set.

**Chaining limitation:** `owo-colors` style methods (`.bold()`, `.green()`, etc.) each return a type that borrows from `self`. Chaining two of them inside an `if_supports_color` closure creates a temporary that the compiler rejects (`E0515`). The workaround is `Style::new().green().bold()`, a value type that combines multiple attributes, passed to a single `.style()` call:

```rust
let success_style = Style::new().green().bold();
header.if_supports_color(Stream::Stdout, |t| t.style(success_style))
```

`Style` is `Copy`, so the closure can capture it by value without lifetime issues.

### `--json` flag

A global `--json: bool` flag on `Cli` is threaded through every `exec()` call. When set, subcommands skip the display functions and fall back to `serde_json::to_string_pretty`, useful for scripting or piping output to `jq`.

---

## Design decisions

**ULID over UUID:** ULIDs are lexicographically sortable by creation time, which makes range queries and log correlation easier. They are also URL-safe without encoding.

**jiff over chrono/time:** jiff provides an ergonomic, timezone-aware API with `Zoned` as a first-class type. The `%Z` strftime specifier renders the timezone abbreviation directly, avoiding manual UTC conversion in display code.

**anyhow for error handling:** Both binaries use `anyhow::Result` with `.context()` on every `?`. This produces an error chain that shows the full call path when a command fails, without requiring custom error types.

**Single crate, two binaries:** Keeping relay and uplink in one crate simplifies dependency management and ensures the shared `comms` library is always compiled with the same version of every dependency.
