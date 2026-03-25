@echo off
REM 小说爬虫运行脚本
REM 使用方法: run-scraper.bat <来源站点> <书名> [作者]

REM 切换到 UTF-8 编码以正确显示中文和特殊字符
chcp 65001 >nul

echo ========================================
echo 小说爬虫工具
echo ========================================
echo.

if "%1"=="" (
    echo 用法: run-scraper.bat ^<来源站点^> ^<书名^> [作者]
    echo.
    echo 参数:
    echo   来源站点  起点 或 独阅读
    echo   书名      要搜索的书名
    echo   作者      可选，用于精确匹配
    echo.
    echo 示例:
    echo   run-scraper.bat 起点 "诡秘之主"
    echo   run-scraper.bat 起点 "诡秘之主" "爱潜水的乌贼"
    echo.
    pause
    exit /b 1
)

set SOURCE_SITE=%1
set TITLE=%2
set AUTHOR=%3

echo 来源站点: %SOURCE_SITE%
echo 书名: %TITLE%
if not "%AUTHOR%"=="" (
    echo 作者: %AUTHOR%
)
echo.

cd /d "%~dp0..\src-tauri"

echo 正在构建并运行爬虫 CLI 工具...
echo.

cargo run --bin scraper_cli -- %SOURCE_SITE% "%TITLE%" %AUTHOR%

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo ❌ 运行失败！
    echo.
    pause
    exit /b 1
)

echo.
pause
