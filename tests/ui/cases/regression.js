setTimeout(function () {
  var out = [];
  function add(name, ok, extra) { out.push((ok ? '' : '✗') + name + (extra ? '=' + extra : '')); }
  function list() {
    return Array.prototype.map.call(document.querySelectorAll('th[data-colkey]'), function (t) {
      return t.getAttribute('data-colkey');
    }).join(',');
  }
  function fire(el, type, x, y) {
    el.dispatchEvent(new MouseEvent(type, { bubbles: true, cancelable: true, clientX: x, clientY: y, button: 0, view: window }));
  }
  function center(el) {
    var b = el.getBoundingClientRect();
    return { x: b.left + b.width / 2, y: b.top + b.height / 2, box: b };
  }
  function drag(fromKey, toKey, side) {
    var from = document.querySelector('th[data-colkey="' + fromKey + '"]');
    var to = document.querySelector('th[data-colkey="' + toKey + '"]');
    var fc = center(from), tc = center(to);
    var tx = side === 'before' ? tc.box.left + 2 : tc.box.right - 2;
    fire(from, 'mousedown', fc.x, fc.y);
    fire(document, 'mousemove', fc.x + 14, fc.y);
    fire(document, 'mousemove', tx, tc.y);
    fire(document, 'mouseup', tx, tc.y);
  }

  // 1. 基础渲染
  document.getElementById('viewTable').click();
  setTimeout(function () {
    add('表头可拖列数', document.querySelectorAll('th[data-colkey]').length === 3, document.querySelectorAll('th[data-colkey]').length);
    add('行数', document.querySelectorAll('tbody tr[data-secid]').length, document.querySelectorAll('tbody tr[data-secid]').length);

    // 2. 拖拽 → 顺序变化
    drag('volume', 'change', 'before');
    add('拖拽后顺序', list(), list());

    // 3. 切卡片再切回表格，顺序应保持
    document.getElementById('viewCard').click();
    document.getElementById('viewTable').click();
    add('切换视图后保持', list() === 'volume,change,prevClose', list());

    // 4. 恢复列顺序
    document.querySelectorAll('details.group')[1].open = true;
    document.getElementById('colOrderReset').click();
    add('恢复默认顺序', list() === 'change,prevClose,volume', list());

    // 5. 隐匿模式
    document.getElementById('stealthBtn').click();
    add('隐匿开启', document.body.getAttribute('data-stealth') === 'on', document.body.getAttribute('data-stealth'));
    add('隐匿下无表头拖拽', document.querySelectorAll('th[data-colkey]').length === 0);
    document.getElementById('stealthBtn').click();
    add('隐匿关闭', document.body.getAttribute('data-stealth') === 'off');

    // 6. 持仓弹窗
    document.querySelector('button[data-act="pos"]').click();
    add('持仓弹窗打开', !document.getElementById('posModal').hidden);
    document.getElementById('posCost').value = '1000';
    document.getElementById('posQty').value = '100';
    fire(document.getElementById('posQty'), 'input', 0, 0);
    add('弹窗预览有内容', document.getElementById('posPreview').textContent.length > 0);
    document.getElementById('posSave').click();
    setTimeout(function () {
      add('保存后关闭', document.getElementById('posModal').hidden);

      // 7. 删除二次确认
      var del = document.querySelector('button[data-act="remove"]');
      del.click();
      add('删除需二次确认', del.classList.contains('armed'), del.textContent);
      del.click();
      setTimeout(function () {
        add('确认后行数减少', document.querySelectorAll('tbody tr[data-secid]').length === 2, document.querySelectorAll('tbody tr[data-secid]').length);
        var bad = out.filter(function (s) { return s.charAt(0) === '✗'; });
        document.title = (bad.length === 0 ? 'PASS ' : 'FAIL ') + out.join(' | ');
      }, 150);
    }, 150);
  }, 150);
}, 40);
