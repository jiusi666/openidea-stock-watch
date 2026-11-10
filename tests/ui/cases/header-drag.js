setTimeout(function () {
  var out = [];
  function list() {
    return Array.prototype.map.call(document.querySelectorAll('th[data-colkey]'), function (t) {
      return t.getAttribute('data-colkey');
    });
  }
  function fire(el, type, x, y) {
    el.dispatchEvent(new MouseEvent(type, { bubbles: true, cancelable: true, clientX: x, clientY: y, button: 0, view: window }));
  }
  function center(el) {
    var b = el.getBoundingClientRect();
    return { x: b.left + b.width / 2, y: b.top + b.height / 2, box: b };
  }
  document.getElementById('viewTable').click();

  setTimeout(function () {
    // 场景 A：拖拽途中来一次刷新（定时轮询 / 点刷新按钮），表头不能被重建。
    var from = document.querySelector('th[data-colkey="volume"]');
    var to = document.querySelector('th[data-colkey="change"]');
    var fc = center(from), tc = center(to);
    fire(from, 'mousedown', fc.x, fc.y);
    fire(document, 'mousemove', fc.x + 14, fc.y);
    document.getElementById('refreshBtn').click();
    out.push('A_survived=' + !!document.querySelector('th.dragging'));
    fire(document, 'mousemove', tc.box.left + 2, tc.y);
    fire(document, 'mouseup', tc.box.left + 2, tc.y);
    out.push('A_after=' + list().join(','));
    out.push('A_dirty=' + (document.body.classList.contains('col-dragging') || document.querySelectorAll('th.dragging,th.drop-before,th.drop-after').length));

    // 场景 B：拖到左侧列之后（after 方向），并且拖到自身不应产生变化。
    var f2 = document.querySelector('th[data-colkey="prevClose"]');
    var t2 = document.querySelector('th[data-colkey="volume"]');
    var fc2 = center(f2), tc2 = center(t2);
    fire(f2, 'mousedown', fc2.x, fc2.y);
    fire(document, 'mousemove', fc2.x - 14, fc2.y);
    fire(document, 'mousemove', tc2.box.right - 3, tc2.y);
    fire(document, 'mouseup', tc2.box.right - 3, tc2.y);
    out.push('B_after=' + list().join(','));

    // 场景 C：只是点击表头（无位移）不应改动顺序。
    var beforeC = list().join(',');
    var c1 = center(document.querySelector('th[data-colkey="change"]'));
    fire(document.querySelector('th[data-colkey="change"]'), 'mousedown', c1.x, c1.y);
    fire(document, 'mousemove', c1.x + 1, c1.y + 1);
    fire(document, 'mouseup', c1.x + 1, c1.y + 1);
    out.push('C_same=' + (list().join(',') === beforeC));

    var okA = out[1] === 'A_after=volume,change,prevClose' && out[0] === 'A_survived=true' && out[2] === 'A_dirty=0';
    var okB = out[3] === 'B_after=volume,prevClose,change';
    var okC = out[4] === 'C_same=true';
    document.title = ((okA && okB && okC) ? 'PASS ' : 'FAIL ') + out.join(' | ');
  }, 150);
}, 40);
