@echo off
REM 小说爬虫工具 - 从头开始抓取
REM 此脚本将清空数据库和封面数据，然后从ID=1开始抓取

REM 切换到 UTF-8 编码以正确显示中文和特殊字符
chcp 65001 >nul

setlocal enabledelayedexpansion

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
echo 小说爬虫工具 - 从头开始抓取
echo ========================================
echo.
echo ⚠️  警告：此操作将删除以下所有数据：
echo    - 数据库中的所有书籍记录
echo    - 所有下载的封面图片
echo.
echo 请确认要继续...
echo.

pause

echo.
echo [1/3] 清空数据库和封面...
python -c "from database.db_manager import DatabaseManager; from config import settings; db = DatabaseManager(settings); result = db.clear_all(confirm=True); exit(0 if result else 1)"

if %ERRORLEVEL% NEQ 0 (
    echo ❌ 清空数据失败！
    pause
    exit /b 1
)

echo ✓ 数据清空完成
echo.
echo [2/3] 开始从ID=1抓取数据...
echo.

python main.py crawl --mode initial --start 1

if %ERRORLEVEL% NEQ 0 (
    echo ❌ 抓取失败！
    pause
    exit /b 1
)

echo.
echo [3/3] 显示统计信息...
echo.

python main.py stats

echo.
echo ========================================
echo ✓ 从头抓取完成！
echo ========================================
echo.

pause
