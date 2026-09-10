// Tauri overlay: drag bar, lock/unlock
(function () {
    const isTauri = typeof window.__TAURI__ !== 'undefined';
    if (!isTauri) return;

    let isLocked = false;

    // --- Drag bar ---
    const bar = document.createElement('div');
    bar.id = 'overlay-drag-bar';
    bar.style.cssText = 'position:fixed;top:0;left:0;width:100%;height:30px;background:rgba(220,30,30,0.85);color:#fff;display:flex;justify-content:center;align-items:center;font:bold 12px/1 sans-serif;z-index:999999;cursor:grab;user-select:none;';
    bar.textContent = 'UNLOCKED \u2014 drag to move  |  Ctrl+Alt+L to lock';
    document.body.appendChild(bar);

    // --- Dashed border ---
    const border = document.createElement('div');
    border.id = 'overlay-border';
    border.style.cssText = 'position:fixed;top:30px;left:0;width:100%;height:calc(100% - 30px);border:2px dashed rgba(220,30,30,0.7);box-sizing:border-box;z-index:999998;pointer-events:none;';
    document.body.appendChild(border);

    document.body.style.paddingTop = '10px';

    function setUnlocked() {
        bar.style.display = 'flex';
        border.style.display = 'block';
        border.style.visibility = 'visible';
    }

    function setLocked() {
        bar.style.display = 'none';
        border.style.display = 'none';
        border.style.visibility = 'hidden';
    }

    // Drag via Tauri startDragging
    bar.addEventListener('mousedown', () => {
        window.__TAURI__.window.getCurrentWindow().startDragging().catch(() => { });
    });

    // Listen to Tauri events for lock toggle (from tray or Ctrl+Alt+L)
    window.__TAURI__.event.listen('toggle-lock', () => {
        isLocked = !isLocked;
        window.__TAURI__.core.invoke('set_locked', { locked: isLocked }).catch(() => { });
        if (isLocked) setLocked(); else setUnlocked();
    });

    // Ctrl+Alt+I: temp interact while locked (5 seconds)
    window.__TAURI__.event.listen('toggle-interact', () => {
        if (!isLocked) return;
        window.__TAURI__.core.invoke('set_locked', { locked: false }).catch(() => { });
        setTimeout(() => {
            window.__TAURI__.core.invoke('set_locked', { locked: true }).catch(() => { });
        }, 5000);
    });
})();
