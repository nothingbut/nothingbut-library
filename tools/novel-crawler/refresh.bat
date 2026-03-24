@echo off
REM 小说爬虫工具 - 刷新脚本
REM 此脚本从 ID 1 开始遍历，跳过已存在的书籍，只抓取缺失的 ID

setlocal enabledelayedexpansion

echo ========================================
echo 小说爬虫工具 - 刷新脚本
echo ========================================
echo.

REM 解析参数
set START_ID=1
set END_ID=
set HELP=

:parse_args
if "%~1"=="" goto end_parse
if "%~1"=="--start" (
    set START_ID=%~2
    shift
    shift
    goto parse_args
)
if "%~1"=="--end" (
    set END_ID=%~2
    shift
    shift
    goto parse_args
)
if "%~1"=="-h" set HELP=1
if "%~1"=="--help" set HELP=1
if defined HELP (
    echo 用法: refresh.bat [--start ID] [--end ID]
    echo.
    echo 参数:
    echo   --start ID  起始书籍ID（默认：1）
    echo   --end ID    结束书籍ID（可选，不指定则无限制）
    echo.
    echo 示例:
    echo   refresh.bat              # 从ID 1开始，无限制
    echo   refresh.bat --start 100  # 从ID 100开始
    echo   refresh.bat --start 1 --end 1000  # ID 1-1000
    echo.
    exit /b 0
)
echo 未知参数: %~1
echo 使用 --help 查看帮助
exit /b 1

:end_parse

echo [1/2] 检查当前状态...
echo.

python main.py stats

echo.
echo [2/2] 开始刷新抓取（跳过已存在的ID）...
echo 参数：
echo   起始 ID: %START_ID%
if defined END_ID (
    echo   结束 ID: %END_ID%
) else (
    echo   结束 ID: 无限制
)
echo.

REM 构建命令
set CMD=python main.py refresh --start %START_ID%
if defined END_ID (
    set CMD=!CMD! --end %END_ID%
)

%CMD%

if %errorlevel% neq 0 (
    echo.
    echo ❌ 刷新失败！
    exit /b 1
)

echo.
echo ========================================
echo ✓ 刷新完成！
echo ========================================
echo.
