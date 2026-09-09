@echo off
setlocal EnableExtensions

REM ============================================================
REM Light Postman - Windows development launcher
REM Repo: https://github.com/Mohamed-Hammada/ligth-postman
REM
REM Prerequisites:
REM   - Node.js + npm
REM   - Rust/Cargo
REM   - Tauri Windows prerequisites
REM
REM Default layout:
REM   %~dp0
REM     src\
REM     src-tauri\
REM     package.json
REM ============================================================

title Light Postman - Dev Launcher
cd /d "%~dp0"

echo.
echo ============================================================
echo   Light Postman - Windows Development Launcher
echo ============================================================
echo.

REM ---------- Check Node / npm ----------
where npm >nul 2>&1
if errorlevel 1 (
    echo [ERROR] npm was not found in PATH.
    echo Install Node.js, restart the terminal, and run this file again.
    pause
    exit /b 1
)

REM ---------- Check Rust / Cargo ----------
where cargo >nul 2>&1
if errorlevel 1 (
    echo [ERROR] cargo was not found in PATH.
    echo Install Rust, restart the terminal, and run this file again.
    pause
    exit /b 1
)

echo [OK] npm found.
echo [OK] cargo found.
echo.

REM ---------- Install frontend dependencies ----------
if not exist "node_modules" (
    echo [INFO] node_modules not found. Running npm ci...
    call npm ci
    if errorlevel 1 (
        echo [ERROR] npm ci failed.
        pause
        exit /b 1
    )
) else (
    echo [OK] node_modules already exists.
)

echo.

REM ---------- Optional Rust dependency check ----------
echo [INFO] Checking Rust project...
pushd "src-tauri"
cargo check
if errorlevel 1 (
    echo [ERROR] cargo check failed.
    popd
    pause
    exit /b 1
)
popd

echo.
echo ============================================================
echo   Starting Tauri development app
echo ============================================================
echo.
echo The Tauri config already defines:
echo   beforeDevCommand = npm run dev
echo   devUrl           = http://localhost:1420
echo.
echo Closing the Tauri window / this console will stop development.
echo.

REM Tauri will start Vite automatically through beforeDevCommand.
call npx tauri dev

echo.
echo ============================================================
echo   Light Postman development process ended.
echo ============================================================
pause
endlocal
