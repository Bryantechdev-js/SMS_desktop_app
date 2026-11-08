# Windows development setup

## 1. Verify before installing

Open **PowerShell**, not Command Prompt. Confirm the prompt begins with `PS`. Run:

```powershell
$PSVersionTable.PSVersion
Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version, BuildNumber, OSArchitecture
Get-CimInstance Win32_ComputerSystem | Select-Object Manufacturer, Model, @{N="RAM_GB";E={[math]::Round($_.TotalPhysicalMemory/1GB,2)}}
Get-CimInstance Win32_Processor | Select-Object Name, NumberOfCores, NumberOfLogicalProcessors
rustc --version; cargo --version; rustup --version
node --version; pnpm --version; git --version
```

## 2. Required tools

Install only missing components:

- Rust stable via rustup: https://www.rust-lang.org/tools/install
- Visual Studio 2022 Build Tools: select **Desktop development with C++**, MSVC, and Windows 10/11 SDK.
- Node.js LTS: https://nodejs.org/
- pnpm: `npm install --global pnpm`
- WebView2 Runtime: https://developer.microsoft.com/microsoft-edge/webview2/
- Docker Desktop: https://www.docker.com/products/docker-desktop/ (optional but recommended for local DB)

Follow Tauri's official Windows prerequisites: https://v2.tauri.app/start/prerequisites/ . Restart terminal after installing tools.

## 3. Check versions

```powershell
rustup show
rustc --version
cargo --version
node --version
pnpm --version
```


## 4. Extract and configure the development workspace

Open PowerShell in the folder where you downloaded the source archive. Adjust the archive path if you saved it elsewhere:

```powershell
$archive = "$HOME\Downloads\complexe-sms-rebuild-research-pass.zip"
$destination = "$HOME\Desktop\complexe-sms-work"
Expand-Archive -LiteralPath $archive -DestinationPath $destination -Force
Set-Location $destination
Copy-Item .env.example .env
```

Generate a one-time bootstrap secret and put it in the root `.env` file. Keep this secret private; it is not a staff password:

```powershell
$bytes = New-Object byte[] 32
[System.Security.Cryptography.RandomNumberGenerator]::Fill($bytes)
$bootstrapSecret = [Convert]::ToBase64String($bytes)
$bootstrapSecret
notepad .env
```

Set `BOOTSTRAP_SECRET` in `.env` to the generated value. The development database password in `docker-compose.yml` is deliberately local-only and must be replaced before any shared or production deployment. Do not expose port 5432 beyond localhost.

## 5. Start PostgreSQL and run the API

Start Docker Desktop first, then:

```powershell
docker compose up -d db
docker compose ps
cargo test --workspace --all-targets
cargo run -p sms-api
```

Keep the API terminal open. It applies pending SQLx migrations on startup. The API listens on `127.0.0.1:8080` by default. This build intentionally refuses to bind to a LAN address until the production network, TLS, identity lifecycle, monitoring and deployment controls are complete.

## 6. Install frontend dependencies and run the desktop app

Open a second PowerShell window:

```powershell
Set-Location "$HOME\Desktop\complexe-sms-work"
pnpm install
pnpm --filter @complexe-sms/desktop typecheck
pnpm --filter @complexe-sms/desktop tauri dev
```

On first launch, choose **First-time setup**, enter the one-time bootstrap secret from `.env`, create the owner username and a unique password of at least 12 characters. After setup, use the username and password to sign in. The first owner account receives the owner role from the least-privilege role catalogue.

The bearer session is held in application memory and expires after 12 hours. Closing/reloading the app requires signing in again. Use the sign-out control before leaving a shared workstation.

## 7. Current functional boundary

At this milestone, verified source functionality includes health/readiness, one-time owner bootstrap, Argon2 password hashing, login/logout, session expiry/revocation, failed-login lockout, role/permission lookup, product catalogue read/create, inventory balance read, and receipt-only stock movement. Only the receipt movement endpoint is exposed; sales, production, waste, returns and stock adjustments are blocked until their dedicated atomic/approval workflows exist. This is not yet an end-user bakery release.

## 8. Build a local unsigned Windows test bundle

After the Rust and frontend checks pass:

```powershell
cargo build --release -p sms-api
pnpm --filter @complexe-sms/desktop tauri build
```

Tauri bundle outputs are under `apps\desktop\src-tauri\target\release\bundle`. The exact installer format depends on the configured Tauri bundle targets and Windows tooling. This is an unsigned local test build, not a signed production installer. Test installation and clean uninstall on a disposable Windows machine before any client deployment.

## 9. Stop conditions before LAN or production use

Do not expose the API to the LAN or use real cash, payments or production stock until the remaining authorization, user lifecycle/MFA, rate limiting, complete transaction workflows, database integration tests, TLS/network deployment, backup/restore drills, monitoring, signed update, installer and acceptance-test gates have passed. Never publish `.env`, bootstrap secrets, database credentials or signing keys.
