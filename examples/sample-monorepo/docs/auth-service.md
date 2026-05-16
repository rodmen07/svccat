# Auth Service

Handles JWT issuance and GitHub / Google OAuth for the platform.

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/auth/token` | Issue a JWT |
| GET  | `/auth/oauth/github` | GitHub OAuth redirect |
| GET  | `/auth/oauth/google` | Google OAuth redirect |
