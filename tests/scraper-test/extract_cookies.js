// 起点Cookies提取脚本
// 在起点网站的Console中运行此代码

(function() {
  console.log('=== 起点Cookies提取工具 ===\n');

  // 获取所有cookies
  const cookies = document.cookie.split(';')
    .map(c => c.trim())
    .filter(c => c.length > 0);

  // 检查是否有cookies
  if (cookies.length === 0) {
    console.error('❌ 未找到任何cookies！');
    console.log('请确保：');
    console.log('1. 已经在起点网站上（https://www.qidian.com）');
    console.log('2. 已完成验证（如果有滑动验证码）');
    return;
  }

  console.log(`✅ 找到 ${cookies.length} 个cookies\n`);

  // 输出格式1：配置文件格式（每行一个）
  console.log('--- 复制下面的内容到 qidian_cookies.txt ---\n');
  cookies.forEach(cookie => {
    console.log(cookie);
  });
  console.log('\n--- 复制结束 ---\n');

  // 输出格式2：环境变量格式
  const cookieStr = cookies.join('; ');
  console.log('环境变量格式（用于测试）：');
  console.log(`export QIDIAN_COOKIES="${cookieStr}"`);

  console.log('\n✅ 提取完成！');
  console.log('\n📋 下一步：');
  console.log('1. 复制上面 "--- 复制下面的内容 ---" 之间的内容');
  console.log('2. 保存到 tests/scraper-test/qidian_cookies.txt 文件');
  console.log('3. 运行: cargo run --bin test_with_cookies');
})();
