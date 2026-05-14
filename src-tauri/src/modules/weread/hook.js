(function() {
  'use strict';

  window.__wereadExtracted = {
    content: '',
    images: [],
    complete: false,
    pageCount: 0
  };

  let currentY = 0;
  let lastY = -1;
  let currentLine = '';
  let currentFontSize = 16;
  let currentColor = '';
  let isHeading = false;
  let headingLevel = 0;

  const originalGetContext = HTMLCanvasElement.prototype.getContext;
  HTMLCanvasElement.prototype.getContext = function(type, attrs) {
    const ctx = originalGetContext.call(this, type, attrs);
    if (type !== '2d' || this.__hooked) return ctx;
    this.__hooked = true;

    return new Proxy(ctx, {
      get: function(target, prop) {
        if (prop === 'fillText') {
          return function(text, x, y) {
            handleFillText(text, x, y);
            return target.fillText.call(target, text, x, y);
          };
        }
        if (prop === 'clearRect') {
          return function() {
            return target.clearRect.apply(target, arguments);
          };
        }
        if (prop === 'restore') {
          return function() {
            target.restore.call(target);
          };
        }
        const val = target[prop];
        if (typeof val === 'function') {
          return val.bind(target);
        }
        return val;
      },
      set: function(target, prop, value) {
        if (prop === 'font') {
          const sizeMatch = value.match(/(\d+(?:\.\d+)?)px/);
          if (sizeMatch) {
            currentFontSize = parseFloat(sizeMatch[1]);
            if (currentFontSize >= 27) {
              isHeading = true;
              headingLevel = 2;
            } else if (currentFontSize >= 23) {
              isHeading = true;
              headingLevel = 3;
            } else if (currentFontSize >= 20) {
              isHeading = true;
              headingLevel = 4;
            } else {
              isHeading = false;
              headingLevel = 0;
            }
          }
        }
        if (prop === 'fillStyle') {
          currentColor = value;
        }
        target[prop] = value;
        return true;
      }
    });
  };

  function handleFillText(text, x, y) {
    if (!text || text.trim() === '') return;

    var yDiff = Math.abs(y - lastY);

    if (lastY >= 0 && yDiff > 10) {
      flushLine();
    }

    currentLine += text;
    lastY = y;
    currentY = y;
  }

  function flushLine() {
    if (currentLine.trim() === '') {
      currentLine = '';
      return;
    }

    var line = currentLine.trim();

    if (isHeading && headingLevel > 0) {
      var prefix = '';
      for (var i = 0; i < headingLevel; i++) prefix += '#';
      line = prefix + ' ' + line;
    }

    window.__wereadExtracted.content += line + '\n\n';
    currentLine = '';
  }

  function collectDomElements() {
    var container = document.querySelector('.readerChapterContent');
    if (!container) return;

    var imgs = container.querySelectorAll('img');
    for (var i = 0; i < imgs.length; i++) {
      var img = imgs[i];
      if (img.src && window.__wereadExtracted.images.indexOf(img.src) === -1) {
        window.__wereadExtracted.images.push(img.src);
        window.__wereadExtracted.content += '\n\n![image](' + img.src + ')\n\n';
      }
    }

    var pres = container.querySelectorAll('pre');
    for (var j = 0; j < pres.length; j++) {
      var code = pres[j].textContent || '';
      if (code.trim()) {
        window.__wereadExtracted.content += '\n\n```\n' + code + '\n```\n\n';
      }
    }
  }

  function markComplete() {
    flushLine();
    collectDomElements();
    window.__wereadExtracted.complete = true;
    window.__wereadExtracted.pageCount++;
  }

  function hasNextPage() {
    var btn = document.querySelector('.readerFooter_button');
    return btn && btn.textContent && btn.textContent.indexOf('下一页') >= 0;
  }

  function clickNextPage() {
    var btn = document.querySelector('.readerFooter_button');
    if (btn) {
      btn.click();
      window.__wereadExtracted.complete = false;
      lastY = -1;
      currentLine = '';
      return true;
    }
    return false;
  }

  window.__wereadMarkComplete = markComplete;
  window.__wereadHasNextPage = hasNextPage;
  window.__wereadClickNextPage = clickNextPage;

  window.__wereadReset = function() {
    window.__wereadExtracted = {
      content: '',
      images: [],
      complete: false,
      pageCount: 0
    };
    lastY = -1;
    currentLine = '';
    currentFontSize = 16;
    isHeading = false;
    headingLevel = 0;
  };

  setTimeout(markComplete, 5000);
})();
