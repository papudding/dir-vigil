# Security Policy

## Supported versions

Security fixes are applied to the latest release (currently the `0.2.x` line).

## Reporting a vulnerability

Please **do not** open a public issue. Use GitHub's private vulnerability
reporting / "Report a vulnerability" button on the repository Security tab.
Include steps to reproduce, affected version, and impact. Do not include real
TOTP secrets, `user_secret.enc`, `DIR_VIGIL_SECRET_KEY`, or alert URL tokens.

## Secrets in this project

Treat the following as secrets:

- `DIR_VIGIL_SECRET_KEY` — AES key used to encrypt `user_secret.enc`
- `user_secret.enc` — encrypted TOTP secret
- `ALERT_URL` — Bark / Server Chan URL often embeds a private token
- Runtime logs and archived logs (they may contain operational details)

After 0.2.0, normal startup does **not** print the TOTP secret, otpauth URI, or
alert URL into logs. Non-interactive deployments can retrieve the enrollment
QR code explicitly with `dir-vigil --print-secret-uri`; run that only in a
private terminal, because its output is the 2FA enrollment secret.

## Security model and limitations

- dir-vigil is a single-user service: one TOTP secret per instance, no
  multi-user accounts.
- The random captcha is an anti-automation speed bump, not the primary
  authentication factor. Access is protected by the TOTP 2FA code. A wrong
  2FA attempt consumes the captcha immediately, so one captcha allows one 2FA try.
- The web UI/API is meant to be self-hosted. By default the backend binds to
  `127.0.0.1`; if you expose it beyond your machine, put a TLS reverse proxy in
  front of it and configure access controls.
- The included nginx config adds rate limiting, request-size limits, security
  headers, and avoids logging query strings. Review it for your own deployment.
- This tool deletes data by design when the timer expires. Backups are your
  responsibility.

## Safe testing

Never test deletion behavior against real data. Use a temporary directory
(`mktemp -d`) or a disposable container volume.
