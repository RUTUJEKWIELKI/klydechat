<div align="center">
<h1>
  Klyde Backend
  
  [![Stars](https://img.shields.io/github/stars/klydechat/klydechat?style=flat-square&logoColor=white)](https://github.com/RUTUJEKWIELKI/klydechat/stargazers)
  [![Forks](https://img.shields.io/github/forks/klydechat/klydechat?style=flat-square&logoColor=white)](https://github.com/RUTUJEKWIELKI/klydechat/network/members)
  [![Pull Requests](https://img.shields.io/github/issues-pr/klydechat/klydechat?style=flat-square&logoColor=white)](https://github.com/RUTUJEKWIELKI/klydechat/pulls)
  [![Issues](https://img.shields.io/github/issues/klydechat/klydechat?style=flat-square&logoColor=white)](https://github.com/RUTUJEKWIELKI/klydechat/issues)
  [![Contributors](https://img.shields.io/github/contributors/klydechat/klydechat?style=flat-square&logoColor=white)](https://github.com/RUTUJEKWIELKI/klydechat/graphs/contributors)
  [![License](https://img.shields.io/github/license/klydechat/klydechat?style=flat-square&logoColor=white)](https://github.com/RUTUJEKWIELKI/klydechat/blob/main/LICENSE)
</h1>
The services and libraries that power the Klyde service.<br/>
<br/>

| Crate              | Path                                               | Description                         |                                                                                                                                                                                                                                                                                                           |
| ------------------ | -------------------------------------------------- | ----------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `core/config`      | [crates/core/config](crates/core/config)           | Core: Configuration                 | ![Crates.io Version](https://img.shields.io/crates/v/revolt-config) ![Crates.io Version](https://img.shields.io/crates/msrv/revolt-config) ![Crates.io Version](https://img.shields.io/crates/size/revolt-config) ![Crates.io License](https://img.shields.io/crates/l/revolt-config)                     |
| `core/database`    | [crates/core/database](crates/core/database)       | Core: Database Implementation       | ![Crates.io Version](https://img.shields.io/crates/v/revolt-database) ![Crates.io Version](https://img.shields.io/crates/msrv/revolt-database) ![Crates.io Version](https://img.shields.io/crates/size/revolt-database) ![Crates.io License](https://img.shields.io/crates/l/revolt-database)             |
| `core/files`       | [crates/core/files](crates/core/files)             | Core: S3 and encryption subroutines | ![Crates.io Version](https://img.shields.io/crates/v/revolt-files) ![Crates.io Version](https://img.shields.io/crates/msrv/revolt-files) ![Crates.io Version](https://img.shields.io/crates/size/revolt-files) ![Crates.io License](https://img.shields.io/crates/l/revolt-files)                         |
| `core/models`      | [crates/core/models](crates/core/models)           | Core: API Models                    | ![Crates.io Version](https://img.shields.io/crates/v/revolt-models) ![Crates.io Version](https://img.shields.io/crates/msrv/revolt-models) ![Crates.io Version](https://img.shields.io/crates/size/revolt-models) ![Crates.io License](https://img.shields.io/crates/l/revolt-models)                     |
| `core/permissions` | [crates/core/permissions](crates/core/permissions) | Core: Permission Logic              | ![Crates.io Version](https://img.shields.io/crates/v/revolt-permissions) ![Crates.io Version](https://img.shields.io/crates/msrv/revolt-permissions) ![Crates.io Version](https://img.shields.io/crates/size/revolt-permissions) ![Crates.io License](https://img.shields.io/crates/l/revolt-permissions) |
| `core/presence`    | [crates/core/presence](crates/core/presence)       | Core: User Presence                 | ![Crates.io Version](https://img.shields.io/crates/v/revolt-presence) ![Crates.io Version](https://img.shields.io/crates/msrv/revolt-presence) ![Crates.io Version](https://img.shields.io/crates/size/revolt-presence) ![Crates.io License](https://img.shields.io/crates/l/revolt-presence)             |
| `core/result`      | [crates/core/result](crates/core/result)           | Core: Result and Error types        | ![Crates.io Version](https://img.shields.io/crates/v/revolt-result) ![Crates.io Version](https://img.shields.io/crates/msrv/revolt-result) ![Crates.io Version](https://img.shields.io/crates/size/revolt-result) ![Crates.io License](https://img.shields.io/crates/l/revolt-result)                     |
| `core/coalesced`   | [crates/core/coalesced](crates/core/coalesced)     | Core: Coalescion service            | ![Crates.io Version](https://img.shields.io/crates/v/revolt-coalesced) ![Crates.io Version](https://img.shields.io/crates/msrv/revolt-coalesced) ![Crates.io Version](https://img.shields.io/crates/size/revolt-coalesced) ![Crates.io License](https://img.shields.io/crates/l/revolt-coalesced)         |
| `delta`            | [crates/delta](crates/delta)                       | REST API server                     | ![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue)                                                                                                                                                                                                                                |
| `bonfire`          | [crates/bonfire](crates/bonfire)                   | WebSocket events server             | ![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue)                                                                                                                                                                                                                                |
| `services/january` | [crates/services/january](crates/services/january) | Proxy server                        | ![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue)                                                                                                                                                                                                                                |
| `services/gifbox`  | [crates/services/gifbox](crates/services/gifbox)   | Tenor proxy server                  | ![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue)                                                                                                                                                                                                                                |
| `services/autumn`  | [crates/services/autumn](crates/services/autumn)   | File server                         | ![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue)                                                                                                                                                                                                                                |
| `daemons/crond`    | [crates/daemons/crond](crates/daemons/crond)       | Timed data clean up daemon server   | ![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue)                                                                                                                                                                                                                                |
| `daemons/pushd`    | [crates/daemons/pushd](crates/daemons/pushd)       | Push notification daemon server     | ![License](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue)                                                                                                                                                                                                                                |

</div>
<br/>

## Minimum Supported Rust Version

Rust 1.92.0 or higher.

## Development Guide

Before contributing, make yourself familiar with [our contribution guidelines](https://developers.klyde.chat/developing/contrib/) and the [technical documentation for this project](https://developers.klyde.chat/).

Before getting started, you'll want to install:

- mise
- Docker
- Git
- mold (optional, faster compilation)

> **Flake** is available for Nix users! Run `nix develop` to enter the dev shell.

As a heads-up, the development environment uses the following ports:

| Service                   |      Port      |
| ------------------------- | :------------: |
| PostgreSQL                |      5432      |
| Redis                     |      6379      |
| MinIO                     |     14009      |
| Maildev                   | 14025<br>14080 |
| Revolt Web App            |     14701      |
| RabbitMQ                  | 5672<br>15672  |
| Grafana LGTM (OTLP)       | 13000<br>4318  |
| `crates/delta`            |     14702      |
| `crates/bonfire`          |     14703      |
| `crates/services/autumn`  |     14704      |
| `crates/services/january` |     14705      |
| `crates/services/gifbox`  |     14706      |

Now you can clone and build the project:

```bash
git clone https://github.com/RUTUJEKWIELKI/klydechat klyde-backend
cd klyde-backend
mise install
mise build
```

> [!TIP]
> You can override `BUILDER` in your `.env` file to run cargo with mold if you installed it:
>
> ```bash
> # .env
> BUILDER = "mold --run cargo"
> ```

A default configuration `Revolt.toml` is present in this project that is suited for development.

If you'd like to change anything, create a `Revolt.overrides.toml` file and specify relevant variables.

> [!TIP]
> Use Sentry to catch unexpected service errors:
>
> ```toml
> # Revolt.overrides.toml
> [sentry]
> api = "https://abc@your.sentry/1"
> events = "https://abc@your.sentry/1"
> files = "https://abc@your.sentry/1"
> proxy = "https://abc@your.sentry/1"
> ```

> [!TIP]
> If you have port conflicts on common services, you can try the following:
>
> ```yaml
> # compose.override.yml
> services:
>   redis:
>     ports: !override
>       - "14079:6379"
>
>   database:
>     ports: !override
>       - "14017:5432"
>
>   rabbit:
>     ports: !override
>       - "14072:5672"
>       - "14672:15672"
>
>   otel-lgtm:
>     ports: !override
>       - "13000:3000"
>       - "14318:4318"
> ```
>
> With the corresponding Revolt configuration:
>
> ```toml
> #     Revolt.overrides.toml
> # and Revolt.test-overrides.toml
> [database]
> postgresql = "postgres://postgres:postgres@127.0.0.1:14017/revolt"
> redis = "redis://127.0.0.1:14079/"
>
> [rabbit]
> port = 14072
> ```
>
> And the OTLP endpoint:
>
> ```bash
> #.env
> OTEL_EXPORTER_OTLP_ENDPOINT = "http://localhost:14318"
> ```

Then continue:

```bash
cp livekit.example.yml livekit.yml

mise start
```

You can start a web client by doing the following in another terminal:

```bash
# if you do not have yarn yet and have a modern Node.js:
corepack enable

# clone the web client and run it:
git clone --recursive https://github.com/RUTUJEKWIELKI/klydefor-web klyde-web
cd klyde-web
# refer to klyde-web/README.md for startup, creating an account and loging in
```

When signing up, go to http://localhost:14080 to find confirmation/password reset emails.

To stop all services, hit (CTRL + c) in the terminal you ran `mise start` and run `mise docker:stop`


## Environment Variables

This list is non-exhaustive right now.

| Variable                              | Default                 | Purpose                                                               |
| ------------------------------------- | ----------------------- | --------------------------------------------------------------------- |
| `KLYDE_DATABASE__POSTGRESQL`          |                         | PostgreSQL connection URL, overrides `Revolt.toml`                    |
| `KLYDE_DATABASE__REDIS`               |                         | Redis connection URL, overrides `Revolt.toml`                         |
| `KLYDE_RABBIT__HOST` / `KLYDE_RABBIT__PORT` |                   | RabbitMQ connection, overrides `Revolt.toml`                          |
| `SERVICE_PATH_PREFIX`                 |                         | Base path prefix for API routes, e.g. `/api` on Vercel                |
| `RUST_LOG`                            | `info`                  | Log filter, e.g. `info,revolt_autumn=debug`                           |
| `OTEL_EXPORTER_OTLP_LOGS_ENDPOINT`    |                         | OTLP/HTTP endpoint for logs                                           |
| `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`  |                         | OTLP/HTTP endpoint for traces                                         |
| `OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` |                         | OTLP/HTTP endpoint for metrics                                        |
| `OTEL_EXPORTER_OTLP_ENDPOINT`         | `http://localhost:4318` | Base URL for a single collector; appends `/v1/{logs,traces,metrics}`  |
| `OTEL_RESOURCE_ATTRIBUTES`            |                         | Extra resource attributes, e.g. `deployment.environment.name=staging` |
| `OTEL_TRACES_SAMPLER`                 | `parentbased_always_on` | Trace sampler, e.g. `parentbased_traceidratio`                       |
| `OTEL_TRACES_SAMPLER_ARG`             | `1.0`                   | Sampling ratio, e.g. `0.1`                                            |

## Deploying on Vercel

`vercel.json` declares every Rust service (delta, bonfire, autumn, january, gifbox, voice-ingress), the docs site, and the `pushd`/`crond` daemons. Vercel runs none of the backing infrastructure, so you must provision PostgreSQL, Redis, RabbitMQ and S3-compatible storage separately and point the deployment at them through environment variables (see [Environment Variables](#environment-variables) below for the `KLYDE_` override syntax).

Required environment variables in your Vercel project:

| Variable | Example | Notes |
| --------------------------------------- | ------------------------------------------------- | ------------------------------------------------------------------ |
| `KLYDE_DATABASE__POSTGRESQL` | `postgres://user:pass@db.example.com:5432/revolt?sslmode=require` | Managed PostgreSQL (Neon, Supabase, RDS...); migrations run automatically on start |
| `KLYDE_DATABASE__REDIS` | `redis://default:pass@redis.example.com:6379` | Redis or KeyDB |
| `KLYDE_RABBIT__HOST` / `KLYDE_RABBIT__PORT` | `rabbit.example.com` / `5672` | RabbitMQ AMQP endpoint |
| `KLYDE_RABBIT__USERNAME` / `KLYDE_RABBIT__PASSWORD` | | RabbitMQ credentials |
| `KLYDE_FILES__S3__ENDPOINT` | `https://s3.us-west-004.backblazeb2.com` | S3-compatible storage (Backblaze B2, Cloudflare R2, AWS S3...) |
| `KLYDE_FILES__S3__REGION` | `us-west-004` | |
| `KLYDE_FILES__S3__ACCESS_KEY_ID` / `KLYDE_FILES__S3__SECRET_ACCESS_KEY` | | |
| `KLYDE_FILES__S3__DEFAULT_BUCKET` | `revolt-uploads` | Bucket must exist before the first upload |
| `KLYDE_FILES__ENCRYPTION_KEY` | | Generate with `openssl rand -base64 32`; files become unreadable if lost |
| `KLYDE_HOSTS__API` | `https://your-domain.com/api` | |
| `KLYDE_HOSTS__EVENTS` | `wss://your-domain.com/ws` | |
| `KLYDE_HOSTS__AUTUMN` | `https://your-domain.com/autumn` | |
| `KLYDE_HOSTS__JANUARY` | `https://your-domain.com/january` | |
| `KLYDE_HOSTS__APP` | `https://your-domain.com` | Web client URL |
| `SERVICE_PATH_PREFIX` | `/api` | Must be set; delta mounts all routes under this prefix |

Optional:

- `KLYDE_API__SMTP__HOST` / `KLYDE_API__SMTP__USERNAME` / `KLYDE_API__SMTP__PASSWORD` / `KLYDE_API__SMTP__PORT` — email verification; without SMTP, sign-up works but email confirmation is unavailable
- `KLYDE_PUSHD__VAPID__PUBLIC_KEY` / `KLYDE_PUSHD__VAPID__PRIVATE_KEY` — web push notifications (replace the development keys baked into the default config)
- `KLYDE_API__SECURITY__CAPTCHA__HCAPTCHA_KEY` / `KLYDE_API__SECURITY__CAPTCHA__HCAPTCHA_SITEKEY` — hCaptcha on auth routes

Request routing (defined in `vercel.json`):

| Path | Service |
| ------------ | ------------- |
| `/api/(.*)` | delta (REST API) |
| `/ws` | bonfire (WebSocket) |
| `/autumn/(.*)` | autumn (file uploads) |
| `/january/(.*)` | january (link embeds) |
| `/gifbox/(.*)` | gifbox (GIF search) |
| `/ingress/(.*)` | voice-ingress |
| `/docs/(.*)` | docs (Docusaurus) |

Notes:

- Voice calls need a LiveKit deployment (e.g. LiveKit Cloud) configured under `[api.livekit]`; without it, everything except voice works out of the box.
- `JANUARY_URL` (delta → january) is wired automatically via the service binding in `vercel.json`.
- The `pushd` and `crond` daemons each run one instance in `iad1` by default; adjust `instances` in `vercel.json` if you deploy elsewhere.

## Deployment Guide

### Cutting new crate releases

Begin by bumping crate versions:

```bash
just patch # 0.0.X
just minor # 0.X.0
just major # X.0.0
```

Then commit the changes to package files.

Proceed to publish all the new crates:

```bash
just publish
```

### Cutting new binary releases

Tag and push a new release by running:

```bash
just release
```

If you have bumped the crate versions, proceed to [GitHub releases](https://github.com/RUTUJEKWIELKI/klydechat/releases/new) to create a changelog.

## Testing

First, start the required services:

```sh
docker compose up -d
```

Now run tests for whichever database:

```sh
TEST_DB=REFERENCE cargo nextest run
TEST_DB=POSTGRES cargo nextest run
```

## License

The Klyde backend is generally licensed under the [GNU Affero General Public License v3.0](https://github.com/RUTUJEKWIELKI/klydechat/blob/main/LICENSE).

**Individual crates may supply their own licenses!**
