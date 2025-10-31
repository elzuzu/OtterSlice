# Security Policy

## Supported Versions

Security updates are provided on a best-effort basis for the `main` branch. Forks and experimental branches may not receive patches.

## Reporting a Vulnerability

- Never commit or share private keys, API secrets, or confidential credentials in this repository.
- If you discover a vulnerability, please email security@otterslice.example with detailed reproduction steps and proposed mitigations.
- Limit vulnerability reports to read-only access; trading and withdrawal endpoints must remain disabled in test harnesses.
- Use the provided paper-trading configuration when investigating issues. Do not target production accounts without prior written consent.

## Operational Guardrails

- Restrict RPC access to trusted IP allowlists.
- Store macOS signing keys in the system Keychain; do not persist them in plaintext.
- Always run `toon` in paper mode before enabling live trading flags.
