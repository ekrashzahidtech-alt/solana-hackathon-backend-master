Deployment steps for Railway

1) Connect repository
- In Railway, create a new project and choose GitHub; connect the `solana-hackathon-backend-master` repo and select the `main` branch.

2) Build settings
- Railway uses `railway.toml` which already specifies:
  - builder = "dockerfile"
  - dockerfilePath = "Dockerfile"
- No change required; Railway will build the Docker image using the repo's `Dockerfile`.

3) Plugins / Services
- Add a PostgreSQL plugin (Railway → Add Plugin → PostgreSQL). Railway will provide a `DATABASE_URL` environment variable.
- (Optional) Add a Redis plugin and set `REDIS_URL` if you want rate-limiting features.

4) Required Environment Variables
- `DATABASE_URL` (provided by Railway Postgres plugin)
- `JWT_SECRET` — generate a secure random string and set in Railway Environment variables

Optional environment variables (set only if you need those features):
- `REDIS_URL` (if you added Redis) — Railway plugin provides this
- `SOLANA_WALLET_PRIVATE_KEY`, `SOLANA_TOKEN_MINT_ADDRESS`, `SOLANA_PROGRAM_ID` — enable Solana features
- `CLOUDINARY_*`, `HF_*`, `FRONTEND_URL`, etc.

5) Healthcheck and Port
- `railway.toml` already configures `healthcheckPath = "/health"` and `healthcheckTimeout = 120`.
- The app binds to the port in the `PORT` env var (default 3000). Railway sets the port automatically.

6) Deploy
- After adding the Postgres plugin and setting `JWT_SECRET`, deploy the project in Railway.
- Monitor logs for:
  - successful container start
  - "Running database migrations in background" messages
  - "Migrations applied successfully" or migration errors

7) Troubleshooting
- If healthcheck fails:
  - Open Logs → look for database connection errors or migration failures.
  - Confirm `DATABASE_URL` is present and correct.
  - Ensure migrations have been applied — the app logs them.
  - If migrations take long, the background retry will attempt up to 10 times with backoff.

Local build & smoke test

Build and run locally (replace placeholders):

```bash
docker build -t backend-rust .

docker run --rm -p 3000:3000 \
  -e DATABASE_URL="postgres://user:pass@host:5432/dbname" \
  -e JWT_SECRET="change-me" \
  backend-rust

# health
curl http://localhost:3000/health
```
