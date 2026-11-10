// 列表拖动排序 + 浮层提示 + 提示收敛 的自动测试
setTimeout(function () {
  var out = [];
  function ok(cond, label) { out.push((cond ? 'OK ' : 'FAIL ') + label); }
  function fire(el, ty, x, y) {
    el.dispatchEvent(new MouseEvent(ty, {
      bubbles: true, cancelable: true, clientX: x, clientY: y, button: 0, view: window
    }));
  }
  function rows() { return Array.prototype.slice.call(document.querySelectorAll('#list [data-secid]')); }
  function order() { return rows().map(function (r) { return r.getAttribute('data-secid'); }).join(','); }
  function toastShown() {
    var t = document.getElementById('toast');
    return t.classList.contains('show') && getComputedStyle(t).visibility === 'visible';
  }
  function toastFixed() { return getComputedStyle(document.getElementById('toast')).position; }

  function drag(fromEl, toEl, side) {
    var f = fromEl.getBoundingClientRect();
    var t = toEl.getBoundingClientRect();
    var fx = f.left + Math.min(24, f.width / 2), fy = f.top + f.height / 2;
    var ty = side === 'after' ? t.bottom - 2 : t.top + 2;
    fire(fromEl, 'mousedown', fx, fy);
    fire(document, 'mousemove', fx, fy + 10);
    fire(document, 'mousemove', t.left + 20, ty);
    fire(document, 'mouseup', t.left + 20, ty);
  }

  // 先补两只，让列表有 5 行
  ['600036', '000858'].forEach(function (c) { document.getElementById('code').value = c; document.getElementById('addBtn').click(); });

  setTimeout(function () {
    // --- 1. 浮层提示不占布局 ---
    var toastEl = document.getElementById('toast');
    document.getElementById('stealthBtn').click();   // 隐匿开关会弹提示
    var shownNow = toastEl.classList.contains('show');
    var fixed = toastFixed();
    var outOfFlow = toastEl.offsetParent === null;
    var hShown = toastEl.getBoundingClientRect().height;
    // 只比较「提示显示 / 隐藏」这一件事对布局的影响：其余状态完全相同
    var addbarShown = document.querySelector('.addbar').getBoundingClientRect().top;
    toastEl.classList.remove('show');
    var addbarHidden = document.querySelector('.addbar').getBoundingClientRect().top;
    toastEl.classList.add('show');
    out.push('showClass=' + shownNow + ' pos=' + fixed + ' outOfFlow=' + outOfFlow +
      ' h=' + Math.round(hShown) + ' shift=' + Math.round(addbarShown - addbarHidden));
    ok(shownNow, '切换隐匿后有浮层提示');
    ok(fixed === 'fixed' && outOfFlow, '提示是浮层、不在文档流里');
    ok(Math.abs(addbarShown - addbarHidden) < 0.5, '提示显示 / 隐藏不影响布局');
    ok(hShown > 10, '提示气泡可见有高度');
    document.getElementById('stealthBtn').click();

    // --- 2. 卡片视图拖动排序 ---
    var before = order();
    var r = rows();
    drag(r[0], r[3], 'after');
    var after = order();
    out.push('card before=' + before + ' after=' + after);
    ok(before !== after && after.split(',')[3] === before.split(',')[0], '卡片拖动：第一只被移到第 4 位');
    ok(document.querySelectorAll('.rowline').length === 0, '拖动结束清理插入线');

    // --- 3. 表格视图拖动排序 ---
    document.getElementById('viewTable').click();
    setTimeout(function () {
      var b2 = order();
      var t = rows();
      drag(t[4], t[1], 'before');
      var a2 = order();
      out.push('table before=' + b2 + ' after=' + a2);
      ok(b2 !== a2 && a2.split(',')[1] === b2.split(',')[4], '表格拖动：最后一只被移到第 2 位');

      // --- 4. 刷新 / 改设置不再弹提示 ---
      var tt = document.getElementById('toast');
      tt.classList.remove('show');
      document.getElementById('refreshBtn').click();
      var shownAfterRefresh = toastShown();
      document.getElementById('p-sortBy').value = 'percent';
      document.getElementById('p-sortBy').dispatchEvent(new Event('change', { bubbles: true }));
      var shownAfterPref = toastShown();
      out.push('refreshToast=' + shownAfterRefresh + ' prefToast=' + shownAfterPref);
      ok(!shownAfterRefresh, '点刷新不弹提示');
      ok(!shownAfterPref, '改设置不弹提示');

      // --- 5. 按涨跌幅排序时禁止拖动 ---
      var beforeDrag = order();
      var rr = rows();
      drag(rr[0], rr[rr.length - 1], 'after');
      ok(order() === beforeDrag, '按涨跌幅排序时拖动无效');

      document.title = out.join(' | ');
    }, 120);
  }, 160);
}, 40);
