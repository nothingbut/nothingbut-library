@echo off
REM Repair gaps in crawled data - scan [start, end] for gaps >= threshold and re-crawl
REM Usage: repairAndContinue.bat start end [gap]

cd /d "%~dp0"

if "%~1"=="" (
    echo Usage: repairAndContinue.bat start end [gap]
    echo   start  - Start book ID
    echo   end    - End book ID
    echo   gap    - Minimum gap threshold ^(default: 100^)
    echo.
    echo Example:
    echo   repairAndContinue.bat 1 10000 100
    pause
    exit /b 1
)

if "%~2"=="" (
    echo Error: end ID is required
    pause
    exit /b 1
)

if "%~3"=="" (
    python repair_and_continue.py --start %~1 --end %~2
) else (
    python repair_and_continue.py --start %~1 --end %~2 --gap %~3
)

echo.
pause
