#!/bin/bash
# 小说爬虫工具 - 刷新脚本
# 此脚本从 ID 1 开始遍历，跳过已存在的书籍，只抓取缺失的 ID

set -e  # Exit on error

echo "========================================"
echo "小说爬虫工具 - 刷新脚本"
echo "========================================"
echo ""

# 解析参数
START_ID=1
END_ID=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --start)
            START_ID="$2"
            shift 2
            ;;
        --end)
            END_ID="$2"
            shift 2
            ;;
        -h|--help)
            echo "用法: ./refresh.sh [--start ID] [--end ID]"
            echo ""
            echo "参数:"
            echo "  --start ID  起始书籍ID（默认：1）"
            echo "  --end ID    结束书籍ID（可选，不指定则无限制）"
            echo ""
            echo "示例:"
            echo "  ./refresh.sh              # 从ID 1开始，无限制"
            echo "  ./refresh.sh --start 100  # 从ID 100开始"
            echo "  ./refresh.sh --start 1 --end 1000  # ID 1-1000"
            echo ""
            exit 0
            ;;
        *)
            echo "未知参数: $1"
            echo "使用 --help 查看帮助"
            exit 1
            ;;
    esac
done

echo "[1/2] 检查当前状态..."
echo ""

python main.py stats

echo ""
echo "[2/2] 开始刷新抓取（跳过已存在的ID）..."
echo "参数："
echo "  起始 ID: $START_ID"
if [ -n "$END_ID" ]; then
    echo "  结束 ID: $END_ID"
else
    echo "  结束 ID: 无限制"
fi
echo ""

# 构建命令
CMD="python main.py refresh --start $START_ID"
if [ -n "$END_ID" ]; then
    CMD="$CMD --end $END_ID"
fi

$CMD

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ 刷新失败！"
    exit 1
fi

echo ""
echo "========================================"
echo "✓ 刷新完成！"
echo "========================================"
echo ""
