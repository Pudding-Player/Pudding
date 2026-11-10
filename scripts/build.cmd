@echo off
setlocal
rem Keep this file ASCII-only: cmd.exe reads the system OEM code page.
cd /d "%~dp0.."
set "ROOT=%CD%"
set "TEMP=%ROOT%\target\tmp"
if defined PUDDING_BUILD_TMP set "TEMP=%PUDDING_BUILD_TMP%"
set "TMP=%TEMP%"
set "TMPDIR=%TEMP%"
set "NODE_DISABLE_COMPILE_CACHE=1"
set "npm_config_logs_max=0"
if not exist "%TEMP%" mkdir "%TEMP%"
if errorlevel 1 exit /b 1

set "CARGO=cargo"
where cargo >nul 2>&1
if errorlevel 1 (
  set "CARGO="
  if exist "%CARGO_HOME%\bin\cargo.exe" set "CARGO=%CARGO_HOME%\bin\cargo.exe"
  if not defined CARGO if exist "%USERPROFILE%\.cargo\bin\cargo.exe" set "CARGO=%USERPROFILE%\.cargo\bin\cargo.exe"
  if not defined CARGO (
    echo cargo not found. Install Rust or add its bin directory to PATH.
    exit /b 1
  )
)
for /f "tokens=2 delims== " %%V in ('findstr /b /c:"version = " Cargo.toml') do set "VERSION=%%~V"
if not defined VERSION exit /b 1
node scripts\bridges.mjs ensure
if errorlevel 1 exit /b 1
if not exist node_modules call npm ci
if errorlevel 1 exit /b 1
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\package-windows.ps1" -Action Check
if errorlevel 1 exit /b 1

echo [1/4] Building independent bridges...
node scripts\bridges.mjs build
if errorlevel 1 exit /b 1

echo [2/4] Building interface...
call npm run build
if errorlevel 1 exit /b 1

echo [3/4] Building desktop...
"%CARGO%" build --release
if errorlevel 1 exit /b 1

echo [4/4] Building installer...
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\package-windows.ps1" -Action Stage
if errorlevel 1 goto failed
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\pack-payload.ps1"
if errorlevel 1 goto failed
"%CARGO%" build --release --no-default-features --features setup --bin pudding-setup
if errorlevel 1 goto failed
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\package-windows.ps1" -Action Complete
if errorlevel 1 goto failed
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\package-windows.ps1" -Action Clean
if errorlevel 1 exit /b 1
exit /b 0

:failed
powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\package-windows.ps1" -Action Clean
exit /b 1
