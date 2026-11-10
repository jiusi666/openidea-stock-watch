var errors = [];
window.addEventListener('error', function (e) { errors.push(String(e.message)); });

setTimeout(function () {
  var out = [];
  function add(n, ok, x) { out.push((ok ? '' : '✗') + n + (x !== undefined ? '=' + x : '')); }
  document.getElementById('viewTable').click();
  setTimeout(function () {
    document.getElementById('colAll').click();
    setTimeout(function () {
      var wrap = document.querySelector('.tablewrap');
      var bar = document.querySelector('.hscroll');
      var thumb = document.querySelector('.hscroll-thumb');
      var tb = thumb.getBoundingClientRect();
      var x0 = tb.left + tb.width / 2, y0 = tb.top + tb.height / 2;

      // 拖拇指到一半时触发一次刷新，表格会被重建
      thumb.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true, clientX: x0, clientY: y0, button: 0, view: window }));
      document.dispatchEvent(new MouseEvent('mousemove', { bubbles: true, clientX: x0 + 40, clientY: y0, view: window }));
      var mid = wrap.scrollLeft;
      document.getElementById('refreshBtn').click();
      document.dispatchEvent(new MouseEvent('mousemove', { bubbles: true, clientX: x0 + 80, clientY: y0, view: window }));
      document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, clientX: x0 + 80, clientY: y0, view: window }));
      add('拖动中被刷新', mid > 0, Math.round(mid));

      var wrap2 = document.querySelector('.tablewrap');
      var bar2 = document.querySelector('.hscroll');
      add('重建后仍只有一个滚动条', document.querySelectorAll('.hscroll').length === 1, document.querySelectorAll('.hscroll').length);
      add('重建后滚动条正常显示', !bar2.classList.contains('off'), bar2.className);
      add('重建后无残留 active', !document.querySelector('.hscroll-thumb.active'));

      // 切卡片再切回表格
      document.getElementById('viewCard').click();
      document.getElementById('viewTable').click();
      var bar3 = document.querySelector('.hscroll');
      add('切视图后滚动条仍在', bar3 && !bar3.classList.contains('off'), bar3 && bar3.className);

      // 新滚动条仍可拖动
      var thumb3 = document.querySelector('.hscroll-thumb');
      var t3 = thumb3.getBoundingClientRect();
      thumb3.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true, clientX: t3.left + 5, clientY: t3.top + 3, button: 0, view: window }));
      document.dispatchEvent(new MouseEvent('mousemove', { bubbles: true, clientX: t3.left + 80, clientY: t3.top + 3, view: window }));
      document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, clientX: t3.left + 80, clientY: t3.top + 3, view: window }));
      var w3 = document.querySelector('.tablewrap');
      add('新滚动条可拖', w3.scrollLeft > 0, Math.round(w3.scrollLeft));

      add('无JS异常', errors.length === 0, errors.join(';') || 'none');
      var bad = out.filter(function (s) { return s.charAt(0) === '✗'; });
      document.title = (bad.length === 0 ? 'PASS ' : 'FAIL ') + out.join(' | ');
    }, 130);
  }, 70);
}, 40);
