(function() {
  var checkInterval = null;
  var detected = false;

  console.log('[weread-login-monitor] script loaded, starting...');

  function checkLoginStatus() {
    if (detected) return;

    console.log('[weread-login-monitor] checking login status via fetch...');

    fetch('https://weread.qq.com/web/user', {
      credentials: 'include'
    })
    .then(function(resp) {
      console.log('[weread-login-monitor] fetch response status:', resp.status);
      return resp.json();
    })
    .then(function(data) {
      console.log('[weread-login-monitor] user data:', JSON.stringify(data).substring(0, 200));
      if (data && data.name && !data.errCode) {
        detected = true;
        clearInterval(checkInterval);

        var vid = String(data.userVid || '');
        var name = data.name || '';
        var avatar = data.avatar || '';

        var payload = 'WEREAD_LOGIN_OK:' + JSON.stringify({vid: vid, name: name, avatar: avatar});
        console.log('[weread-login-monitor] LOGIN SUCCESS! Setting title to:', payload);
        document.title = payload;
        console.log('[weread-login-monitor] document.title is now:', document.title);
      } else {
        console.log('[weread-login-monitor] not logged in yet, errCode:', data.errCode);
      }
    })
    .catch(function(err) {
      console.log('[weread-login-monitor] fetch error:', err);
    });
  }

  function startMonitoring() {
    console.log('[weread-login-monitor] startMonitoring called');
    checkInterval = setInterval(checkLoginStatus, 3000);
    setTimeout(checkLoginStatus, 1000);
  }

  if (document.readyState === 'complete' || document.readyState === 'interactive') {
    startMonitoring();
  } else {
    document.addEventListener('DOMContentLoaded', startMonitoring);
  }
})();
