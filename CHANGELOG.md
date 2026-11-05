# Changelog

## [0.2.0] - 2026-11-05

### Added

- Responsive web UI for both mobile and desktop, with a compact layout that
  avoids page scrollbars on common viewports.
- `--host` (default `127.0.0.1`) and `--cors-origin` options.
- `--print-secret-uri` for first-time TOTP enrollment on non-interactive hosts
  such as Docker.
- Application logs are written under `/app/logs` (mounted from the host);
  startup archiving in `backend_startup.sh` keeps
  `logs/dir-vigil-YYYYmmdd-HHMMSS.log` and appends new logs.
- nginx: rate limiting, request-size limit, security headers, gzip, static
  asset caching, safe access-log format, and `server_tokens off`.
- GitHub Actions CI (format, clippy, tests, frontend build, Docker build) and
  a release workflow generating GitHub release notes (no images or binary artifacts).
- SECURITY.md, CONTRIBUTING.md, CHANGELOG.md, Dependabot configuration.
- Package metadata (`description`, `license`, `repository`, Node engine).

### Changed (breaking)

- `GET /keepalive?...` is now `POST /keepalive` with a JSON body
  (`captcha`, `tow_fa_code`).
- TOTP/alert URL values are no longer written to logs. On a non-TTY stdout the
  QR/otpauth URI is not printed; use `--print-secret-uri` when enrolling.
- The AES key for `user_secret.enc` must be supplied via
  `DIR_VIGIL_SECRET_KEY` (32 raw bytes or 64 hex characters). Upgrading from
  `0.1.0` requires the legacy key or deleting `user_secret.enc` and re-enrolling.
  The secret file is now created with mode `600`.
- The backend binds to `127.0.0.1` by default; use `--host 0.0.0.0` only behind
  a trusted proxy.
- The Dockerfile is now a multi-stage build that compiles the backend and
  frontend from source; prebuilt artifacts are no longer required or released.
- Frontend `.env` is no longer tracked; use `frontend/h5/.env.example`.

### Fixed

- All backend unit tests pass (14 tests); removed the previously failing
  placeholder tests and stale TOTP length assertion.
- Malformed 2FA input no longer panics; the captcha is invalidated after a
  single failed 2FA attempt (previously up to 5 retries), and status JSON is
  generated via `serde_json` (no unescaped comment).
- README corrections (typos, 6-hour warning default, missing environment vars)
  and updated Docker/bare-metal instructions.

## [0.1.0] - 2025-05-21

- Initial public release: TOTP-gated keep-alive, timed directory cleanup,
  Bark / Server Chan alerts, H5 frontend, Docker image.
