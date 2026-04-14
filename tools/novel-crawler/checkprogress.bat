@echo off
REM Check crawl progress - last success ID and gap >= 100 ID pairs
REM Usage: checkprogress.bat [gap threshold]

cd /d "%~dp0"

if not exist "data\youshu.db" (
    echo Database not found: data\youshu.db
    pause
    exit /b 1
)

if "%~1"=="" (
    python check_progress.py
) else (
    python check_progress.py --gap %~1
)

echo.
pause
