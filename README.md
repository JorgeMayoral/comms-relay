# Comms Relay

A self-hosted cross-posting relay for publishing content to Mastodon and Bluesky simultaneously. You write once through the **uplink** CLI; the **relay** server stores the publication and (once implemented) fans it out to each social platform.

> **Status:** Storage and the HTTP API are fully functional. Social platform posting (Mastodon, Bluesky) is planned but not yet implemented.

For implementation details, see [IMPLEMENTATION.md](IMPLEMENTATION.md).

---

## Components

| Component | Description |
|---|---|
| **relay** | HTTP server (port 8000). Stores publications in Postgres and will handle cross-posting |
| **uplink** | CLI client. Create, list, and retrieve publications |
| **comms** | Shared Rust library. `Publication` type and HTTP payload types used by both binaries |

---

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- [Docker](https://docs.docker.com/get-docker/) (for the local Postgres instance)
- [mise](https://mise.jdx.dev/) (task runner and env management, optional but recommended)

---

## Quick start

### 1. Start the database

```sh
mise run dev-db
# or: docker compose up db
```

### 2. Start the relay server

```sh
mise run server
# or: RELAY_API_TOKEN=mytoken DATABASE_URL=postgres://user:password@localhost:5432/relay cargo run --bin relay
```

The server binds to `0.0.0.0:8000` and runs database migrations automatically on startup.

### 3. Configure the uplink client

```sh
uplink config --url http://localhost:8000 --token mytoken
```

Settings are saved to `~/.config/uplink/config.toml` and used automatically by subsequent commands.

### 4. Publish something

```sh
uplink publish "Hello from Comms Relay!"
```

---

## Uplink reference

### Global flags

These flags apply to every subcommand and take priority over the saved config and environment variables.

| Flag | Env var | Description |
|---|---|---|
| `--url <URL>` | `RELAY_URL` | Relay server base URL |
| `--token <TOKEN>` | `RELAY_API_TOKEN` | Bearer token for authenticated requests |
| `--json` | - | Output raw JSON instead of formatted text |

### `uplink config`

Save the relay URL and/or token to the local config file. Each flag is optional; omitting one leaves the existing value unchanged.

```
uplink config [--url <URL>] [--token <TOKEN>]
```

```sh
uplink config --url http://relay.example.com --token s3cr3t
uplink config --token newtoken          # update token only
```

### `uplink publish`

Create a new publication. Requires a token (from config, `RELAY_API_TOKEN`, or `--token`).

```
uplink publish <CONTENT>
```

```sh
uplink publish "My first post"
uplink publish --json "My first post"   # raw JSON output
```

**Output:**

```
Published · 2026/03/07 - 14:32 (UTC)
  My first post
  ID: 01JNXK2ABCDEFGHIJKLMNOPQRS
```

### `uplink list`

List all publications, newest first.

```
uplink list [--json]
```

**Output:**

```
01JNXK2ABCDEFGHIJKLMNOPQRS · 2026/03/07 - 14:32 (UTC)
  My first post

01JNXK1ABCDEFGHIJKLMNOPQRS · 2026/03/06 - 09:10 (UTC)
  An earlier publication
```

### `uplink get`

Fetch a single publication by its ULID.

```
uplink get <ID> [--json]
```

```sh
uplink get 01JNXK2ABCDEFGHIJKLMNOPQRS
```

**Output:**

```
01JNXK2ABCDEFGHIJKLMNOPQRS · 2026/03/07 - 14:32 (UTC)
  My first post

Mastodon: (not posted)
Bluesky:  (not posted)
```

---

## Relay HTTP API

All endpoints are served on port 8000. Write endpoints require an `Authorization: Bearer <token>` header matching the server's `RELAY_API_TOKEN`.

### `POST /publications`

Create a new publication.

**Auth required:** yes

**Request body:**
```json
{ "content": "Hello from the API!" }
```

**Response** `201 Created`:
```json
{
  "id": "01JNXK2ABCDEFGHIJKLMNOPQRS",
  "content": "Hello from the API!",
  "pub_date": "2026-03-07T16:35:03.970665956+00:00[UTC]",
  "mastodon_id": null,
  "mastodon_url": null,
  "bluesky_id": null,
  "bluesky_url": null
}
```

### `GET /publications`

List all publications ordered by date, newest first.

**Auth required:** no

**Response** `200 OK`: array of publication objects (same shape as above).

### `GET /publications/{id}`

Fetch one publication by its ULID.

**Auth required:** no

**Response** `200 OK`: single publication object, or `404 Not Found`.

---

## Configuration reference

### Relay (server)

| Variable | Required | Description |
|---|---|---|
| `DATABASE_URL` | yes | Postgres connection string (`postgres://user:password@host:5432/db`) |
| `RELAY_API_TOKEN` | yes | Bearer token validated on write requests |
| `RUST_LOG` | no | Log filter (default: `info`) |

### Uplink (client)

Configuration is read from three sources in priority order (highest first):

1. CLI flag (`--url`, `--token`)
2. Environment variable (`RELAY_URL`, `RELAY_API_TOKEN`)
3. Config file (`~/.config/uplink/config.toml`)

The config file is managed with `uplink config` and looks like:

```toml
url = "http://localhost:8000"
token = "your-token-here"
```
