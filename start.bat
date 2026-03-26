@echo off
REM EPUB Library Startup Script
REM Starts the Tauri development server

echo ========================================
echo   EPUB Library - Development Mode
echo ========================================
echo.

REM Check if bun is installed
where bun >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: bun is not installed
    echo Please install bun: https://bun.sh
    exit /b 1
)

REM Check if node_modules exists
if not exist "node_modules" (
    echo Installing dependencies...
    bun install
    echo.
)

REM Check if Rust is installed
where cargo >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Rust/Cargo is not installed
    echo Please install Rust: https://rustup.rs
    exit /b 1
)

echo Starting EPUB Library...
echo Frontend: http://localhost:1420
echo EPUB Route: http://localhost:1420/epub
echo.
echo Press Ctrl+C to stop
echo.

REM Start Tauri dev
bun run tauri:dev
