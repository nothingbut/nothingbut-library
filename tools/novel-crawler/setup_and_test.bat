@echo off
REM Installation and Test Script for Windows

REM 切换到 UTF-8 编码以正确显示中文和特殊字符
chcp 65001 >nul

echo ========================================
echo Novel Crawler - Setup and Test
echo ========================================
echo.

REM Check if venv exists
if not exist "venv\" (
    echo Creating virtual environment...
    python -m venv venv
    if errorlevel 1 (
        echo ERROR: Failed to create virtual environment
        echo Please make sure Python 3.8+ is installed
        pause
        exit /b 1
    )
    echo Virtual environment created successfully!
    echo.
)

REM Activate venv
echo Activating virtual environment...
call venv\Scripts\activate.bat
if errorlevel 1 (
    echo ERROR: Failed to activate virtual environment
    pause
    exit /b 1
)
echo.

REM Install dependencies
echo Installing dependencies...
pip install -r requirements.txt -i https://pypi.tuna.tsinghua.edu.cn/simple
if errorlevel 1 (
    echo WARNING: Some packages may have failed to install
    echo.
)
echo Dependencies installed!
echo.

REM Run tests
echo ========================================
echo Running Tests
echo ========================================
echo.

echo Running manual tests...
python tests/test_manual.py all
echo.

echo.
echo ========================================
echo Setup Complete!
echo ========================================
echo.
echo You can now use the crawler with:
echo   python main.py single --book-id 1
echo   python main.py crawl --mode initial
echo   python main.py stats
echo.

pause
