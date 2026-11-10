setTimeout(function () {
  var out = [];
  function add(name, ok, extra) { out.push((ok ? '' : '✗') + name + (extra !== undefined ? '=' + extra : '')); }
  document.getElementById('viewTable').click();
  setTimeout(function () {
    document.getElementById('colAll').click();
    setTimeout(function () {
      var wrap = document.querySelector('.tablewrap');
      var bar = document.querySelector('.hscroll');
      var thumb = document.querySelector('.hscroll-thumb');
      var gap = wrap.scrollWidth - wrap.clientWidth;
      add('溢出', gap > 2, gap);
      add('滚动条显示', !bar.classList.contains('off'), bar.className);
      add('底部留白生效', wrap.classList.contains('hscroll-on'));
      var tw = thumb.offsetWidth, track = bar.clientWidth;
      add('拇指比例合理', tw > 26 && tw < track, tw + '/' + track);

      // 拖动拇指：起点在拇指中央，向右拖 60px
      var tb = thumb.getBoundingClientRect();
      var startX = tb.left + tb.width / 2, y = tb.top + tb.height / 2;
      thumb.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true, clientX: startX, clientY: y, button: 0, view: window }));
      document.dispatchEvent(new MouseEvent('mousemove', { bubbles: true, clientX: startX + 60, clientY: y, view: window }));
      var movedTo = wrap.scrollLeft;
      document.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, clientX: startX + 60, clientY: y, view: window }));
      add('拖拇指能滚动', movedTo > 0, Math.round(movedTo));
      add('拖动中样式', thumb.classList.contains('active') === false, 'released');

      // 点轨道右端 → 应该跳到最右附近
      var bb = bar.getBoundingClientRect();
      bar.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true, clientX: bb.right - 4, clientY: bb.top + 3, button: 0, view: window }));
      add('点轨道跳转', wrap.scrollLeft > gap * 0.8, Math.round(wrap.scrollLeft));

      // 滚到最右后拇指贴右边缘
      wrap.scrollLeft = gap;
      wrap.dispatchEvent(new Event('scroll'));
      var m = /translateX\((-?[\d.]+)px\)/.exec(thumb.style.transform);
      var x = m ? parseFloat(m[1]) : -1;
      add('拇指贴到右端', Math.abs(x - (track - thumb.offsetWidth)) <= 2, x + '/' + (track - thumb.offsetWidth));

      // 还原到左端，右侧内阴影提示存在
      wrap.scrollLeft = 0;
      wrap.dispatchEvent(new Event('scroll'));
      add('左侧无阴影右侧有', !wrap.classList.contains('more-left') && wrap.classList.contains('more-right'));

      // 列少时不显示滚动条
      document.getElementById('colNone').click();
      setTimeout(function () {
        var w2 = document.querySelector('.tablewrap');
        var b2 = document.querySelector('.hscroll');
        add('无溢出时隐藏', b2.classList.contains('off'), b2.className);
        add('无溢出时无留白', !w2.classList.contains('hscroll-on'));
        var bad = out.filter(function (s) { return s.charAt(0) === '✗'; });
        document.title = (bad.length === 0 ? 'PASS ' : 'FAIL ') + out.join(' | ');
      }, 120);
    }, 120);
  }, 60);
}, 40);
