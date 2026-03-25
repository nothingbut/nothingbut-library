# Installation and Setup Guide

## Quick Setup

### 1. Activate Virtual Environment

**Windows:**
```bash
cd tools/novel-crawler
venv\Scripts\activate
```

**macOS/Linux:**
```bash
cd tools/novel-crawler
source venv/bin/activate
```

### 2. Install Dependencies

```bash
pip install -r requirements.txt
```

### 3. Verify Installation

```bash
python tests/test_manual.py all
```

## What Gets Installed

- `requests==2.31.0` - HTTP library
- `beautifulsoup4==4.12.3` - HTML parsing
- `lxml==4.9.3` - XML/HTML parser
- `APScheduler==3.10.4` - Task scheduling
- `python-dotenv==1.0.0` - Environment variables
- `pydantic==2.5.0` - Data validation
- `pytest==7.4.3` - Testing framework
- `pytest-cov==4.1.0` - Test coverage

## Troubleshooting

### Issue: ModuleNotFoundError

**Solution:** Make sure you've activated the virtual environment and installed dependencies.

```bash
# Check if venv is activated
python --version

# Install dependencies
pip install -r requirements.txt
```

### Issue: pip is slow

**Solution:** Use a faster pip mirror.

```bash
# Use Tsinghua mirror (China)
pip install -r requirements.txt -i https://pypi.tuna.tsinghua.edu.cn/simple

# Or use Aliyun mirror
pip install -r requirements.txt -i https://mirrors.aliyun.com/pypi/simple/
```

## Next Steps

After installation:

1. **Run manual tests:**
   ```bash
   python tests/test_manual.py all
   ```

2. **Run unit tests:**
   ```bash
   pytest tests/ -v
   ```

3. **Try a single book crawl:**
   ```bash
   python main.py single --book-id 1
   ```

4. **Check database stats:**
   ```bash
   python main.py stats
   ```
