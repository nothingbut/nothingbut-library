#!/bin/bash
# 小说爬虫工具 - 从头开始抓取
# 此脚本将清空数据库和封面数据，然后从ID=1开始抓取

set -e  # Exit on error

echo "========================================"
echo "小说爬虫工具 - 从头开始抓取"
echo "========================================"
echo ""
echo "⚠️  警告：此操作将删除以下所有数据："
echo "   - 数据库中的所有书籍记录"
echo "   - 所有下载的封面图片"
echo ""
echo "请确认要继续..."
echo ""

read -p "按 Enter 继续，或 Ctrl+C 取消..."

echo ""
echo "[1/3] 清空数据库和封面..."
echo ""

python -c "from database.db_manager import DatabaseManager; from config import settings; db = DatabaseManager(settings); result = db.clear_all(confirm=True); exit(0 if result else 1)"

if [ $? -ne 0 ]; then
    echo "❌ 清空数据失败！"
    exit 1
fi

echo "✓ 数据清空完成"
echo ""
echo "[2/3] 开始从ID=1抓取数据..."
echo ""

python main.py crawl --mode initial --start 1

if [ $? -ne 0 ]; then
    echo "❌ 抓取失败！"
    exit 1
fi

echo ""
echo "[3/3] 显示统计信息..."
echo ""

python main.py stats

echo ""
echo "========================================"
echo "✓ 从头抓取完成！"
echo "========================================"
echo ""
