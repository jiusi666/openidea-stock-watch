setTimeout(function () {
  var out = [];
  function add(n, ok, x) { out.push((ok ? '' : '✗') + n + (x !== undefined ? '=' + x : '')); }
  document.getElementById('viewTable').click();
  setTimeout(function () {
    document.getElementById('colAll').click();
    setTimeout(function () {
      var wrap = document.querySelector('.tablewrap');
      var gap = wrap.scrollWidth - wrap.clientWidth;
      wrap.scrollLeft = gap * 0.5;
      wrap.dispatchEvent(new Event('scroll'));
      setTimeout(function () {
        var wb = wrap.getBoundingClientRect();
        var c1 = document.querySelector('tbody td:nth-child(1)').getBoundingClientRect();
        var c2 = document.querySelector('tbody td:nth-child(2)').getBoundingClientRect();
        var h1 = document.querySelector('thead th:nth-child(1)').getBoundingClientRect();
        var h2 = document.querySelector('thead th:nth-child(2)').getBoundingClientRect();
        var col1w = wrap.style.getPropertyValue('--col1w');
        add('已滚动', Math.round(wrap.scrollLeft), Math.round(wrap.scrollLeft));
        add('col1w 已量出', !!col1w, col1w);
        add('代码列贴左', Math.abs(c1.left - (wb.left + 1)) <= 2, Math.round(c1.left - wb.left));
        add('名称列紧邻代码列', Math.abs(c2.left - c1.right) <= 2, Math.round(c2.left - c1.right));
        add('表头与数据列对齐', Math.abs(h1.left - c1.left) <= 1 && Math.abs(h2.left - c2.left) <= 1, Math.round(h1.left - c1.left) + '/' + Math.round(h2.left - c2.left));
        add('操作列仍在右侧', Math.abs(document.querySelector('td.actions').getBoundingClientRect().right - (wb.right - 1)) <= 2);
        // 名称列应仍在可视区，且宽度合理
        add('名称列可见', c2.right <= wb.right, Math.round(c2.width));
        var bad = out.filter(function (s) { return s.charAt(0) === '✗'; });
        document.title = (bad.length === 0 ? 'PASS ' : 'FAIL ') + out.join(' | ');
      }, 120);
    }, 130);
  }, 60);
}, 40);
