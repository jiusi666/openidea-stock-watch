setTimeout(function () {
  var out = [];
  function list() {
    return Array.prototype.map.call(document.querySelectorAll('th[data-colkey]'), function (t) {
      return t.getAttribute('data-colkey');
    }).join(',');
  }
  function fire(el, type, x, y) {
    el.dispatchEvent(new MouseEvent(type, { bubbles: true, cancelable: true, clientX: x, clientY: y, button: 0, view: window }));
  }
  function key(k) {
    document.dispatchEvent(new KeyboardEvent('keydown', { key: k, bubbles: true, cancelable: true }));
  }
  function center(el) {
    var b = el.getBoundingClientRect();
    return { x: b.left + b.width / 2, y: b.top + b.height / 2, box: b };
  }
  document.getElementById('viewTable').click();

  setTimeout(function () {
    var base = list();
    out.push('base=' + base);

    // D：拖到一半按 Esc，应当放弃并复原。
    var from = document.querySelector('th[data-colkey="volume"]');
    var to = document.querySelector('th[data-colkey="change"]');
    var fc = center(from), tc = center(to);
    fire(from, 'mousedown', fc.x, fc.y);
    fire(document, 'mousemove', fc.x + 16, fc.y);
    fire(document, 'mousemove', tc.box.left + 2, tc.y);
    key('Escape');
    fire(document, 'mouseup', tc.box.left + 2, tc.y);
    out.push('D_same=' + (list() === base));
    out.push('D_dirty=' + (document.body.classList.contains('col-dragging') || document.querySelectorAll('th.dragging,th.drop-before,th.drop-after').length));

    // E：拖到表格外面（没有落在任何列上）松手，顺序不该变。
    var f2 = document.querySelector('th[data-colkey="volume"]');
    var fc2 = center(f2);
    fire(f2, 'mousedown', fc2.x, fc2.y);
    fire(document, 'mousemove', fc2.x + 16, fc2.y);
    fire(document, 'mousemove', 5, 5);
    fire(document, 'mouseup', 5, 5);
    out.push('E_same=' + (list() === base));
    out.push('E_dirty=' + (document.body.classList.contains('col-dragging') || document.querySelectorAll('th.dragging,th.drop-before,th.drop-after').length));

    // F：订单列的「持仓」按钮仍然可以点开弹窗（确认拖动实现没抢走点击）。
    document.querySelector('button[data-act="pos"]').click();
    out.push('F_modal=' + (!document.getElementById('posModal').hidden));
    document.getElementById('posCancel').click();
    out.push('F_closed=' + document.getElementById('posModal').hidden);

    var ok = out[1] === 'D_same=true' && out[2] === 'D_dirty=0' &&
             out[3] === 'E_same=true' && out[4] === 'E_dirty=0' &&
             out[5] === 'F_modal=true' && out[6] === 'F_closed=true';
    document.title = (ok ? 'PASS ' : 'FAIL ') + out.join(' | ');
  }, 150);
}, 40);
