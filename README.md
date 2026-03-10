# Comms Relay

A self-hosted cross-posting relay for publishing content to Mastodon and Bluesky simultaneously. You write once through the **uplink** CLI; the **relay** server stores the publication and fans it out to each social platform.

For implementation details, see [IMPLEMENTATION.md](IMPLEMENTATION.md).

---

## Components

| Component | Description |
|---|---|
| **relay** | HTTP server (port 8000). Stores publications in Postgres and will handle cross-posting |
| **uplink** | CLI client. Create, list, retrieve, and delete publications |
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
# or: set the required env vars (see Configuration reference) and run:
# cargo run --bin relay
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

List publications, newest first. Results are paginated.

```
uplink list [--page <N>] [--per-page <N>] [--json]
```

| Flag | Default | Description |
|---|---|---|
| `--page <N>` | `1` | Page number to fetch (1-indexed) |
| `--per-page <N>` | `100` | Number of results per page (max 500) |

**Output:**

```
01JNXK2ABCDEFGHIJKLMNOPQRS · 2026/03/07 - 14:32 (UTC)
  My first post

01JNXK1ABCDEFGHIJKLMNOPQRS · 2026/03/06 - 09:10 (UTC)
  An earlier publication

Page 1 of 1 · 2 publications total
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

### `uplink delete`

Delete a publication by its ULID. Requires a token (from config, `RELAY_API_TOKEN`, or `--token`).

```
uplink delete <ID>
```

```sh
uplink delete 01JNXK2ABCDEFGHIJKLMNOPQRS
```

**Output:**

```
Deleted · 01JNXK2ABCDEFGHIJKLMNOPQRS
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

List publications ordered by date, newest first. Results are paginated.

**Auth required:** no

**Query parameters:**

| Parameter | Default | Constraints | Description |
|---|---|---|---|
| `page` | `1` | integer >= 1 | Page number (1-indexed) |
| `per_page` | `100` | integer 1–500 | Results per page |

**Response** `200 OK`:
```json
{
  "publications": [
    {
      "id": "01JNXK2ABCDEFGHIJKLMNOPQRS",
      "content": "Hello from the API!",
      "pub_date": "2026-03-07T16:35:03.970665956+00:00[UTC]",
      "mastodon_id": null,
      "mastodon_url": null,
      "bluesky_id": null,
      "bluesky_url": null
    }
  ],
  "page": 1,
  "per_page": 100,
  "total_results": 1,
  "total_pages": 1
}
```

**Response** `422 Unprocessable Entity`: returned when `page < 1` or `per_page` is outside the range `[1, 500]`.

### `GET /publications/{id}`

Fetch one publication by its ULID.

**Auth required:** no

**Response** `200 OK`: single publication object, or `404 Not Found`.

### `DELETE /publications/{id}`

Delete a publication by its ULID.

**Auth required:** yes

**Response** `204 No Content` if deleted, `404 Not Found` if the ID does not exist.

---

## Configuration reference

### Relay (server)

| Variable | Required | Description |
|---|---|---|
| `DATABASE_URL` | yes | Postgres connection string (`postgres://user:password@host:5432/db`) |
| `RELAY_API_TOKEN` | yes | Bearer token validated on write requests |
| `MASTODON_ACCESS_TOKEN` | yes | OAuth bearer token for Mastodon |
| `MASTODON_INSTANCE_URL` | yes | Mastodon server base URL (e.g. `https://mastodon.social`) |
| `BLUESKY_INSTANCE_URL` | yes | Bluesky PDS base URL (e.g. `https://bsky.social`) |
| `BLUESKY_IDENTIFIER` | yes | Bluesky account handle or DID |
| `BLUESKY_APP_PASSWORD` | yes | Bluesky app password |
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

---

## Deployment

### Docker Compose

The following `docker-compose.yml` example deploys the relay alongside Postgres.

Environment variables are loaded from a `.env` file:

| Variable | Description |
|---|---|
| `POSTGRES_DB` | Postgres database name |
| `POSTGRES_USER` | Postgres username |
| `POSTGRES_PASSWORD` | Postgres password |
| `RELAY_API_TOKEN` | Bearer token for the Relay server. Used by the Uplink client. |
| `MASTODON_INSTANCE_URL` | Mastodon server base URL |
| `MASTODON_ACCESS_TOKEN` | OAuth bearer token for Mastodon |
| `BLUESKY_INSTANCE_URL` | Bluesky PDS base URL |
| `BLUESKY_IDENTIFIER` | Bluesky account handle or DID |
| `BLUESKY_APP_PASSWORD` | Bluesky app password |

```yaml
services:
  db:
    image: postgres:18
    environment:
      POSTGRES_DB: ${POSTGRES_DB}
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - db_data:/var/lib/postgresql
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER} -d relay"]
      interval: 5s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  relay:
    image: ghcr.io/jorgemayoral/comms-relay:latest
    expose:
      - 8000
    environment:
      DATABASE_URL: postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@db:5432/${POSTGRES_DB}
      RELAY_API_TOKEN: ${RELAY_API_TOKEN}
      RUST_LOG: info
      MASTODON_INSTANCE_URL: ${MASTODON_INSTANCE_URL}
      BLUESKY_INSTANCE_URL: ${BLUESKY_INSTANCE_URL}
      MASTODON_ACCESS_TOKEN: ${MASTODON_ACCESS_TOKEN}
      BLUESKY_IDENTIFIER: ${BLUESKY_IDENTIFIER}
      BLUESKY_APP_PASSWORD: ${BLUESKY_APP_PASSWORD}
    depends_on:
      db:
        condition: service_healthy
    restart: unless-stopped

volumes:
  db_data:
```
