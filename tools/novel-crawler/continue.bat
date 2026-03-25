@echo off
REM 小说爬虫工具 - 继续上次进度
REM 此脚本将从上次停止的位置继续抓取

REM 切换到 UTF-8 编码以正确显示中文和特殊字符
chcp 65001 >nul

REM 切换到脚本所在目录
cd /d "%~dp0"

REM 检查并激活虚拟环境
if not exist "venv\Scripts\activate.bat" (
    echo ❌ 虚拟环境不存在！
    echo 请先运行 setup_and_test.bat 安装依赖
    echo.
    pause
    exit /b 1
)

call venv\Scripts\activate.bat

echo ========================================
echo 小说爬虫工具 - 继续上次进度
echo ========================================
echo.

echo [1/2] 检查当前状态...
echo.

python main.py stats

echo.
echo [2/2] 继续抓取数据（增量模式）...
echo.

python main.py crawl --mode incremental

if %ERRORLEVEL% NEQ 0 (
    echo ❌ 抓取失败！
    pause
    exit /b 1
)

echo.
echo ========================================
echo ✓ 增量抓取完成！
echo ========================================
echo.

pause
