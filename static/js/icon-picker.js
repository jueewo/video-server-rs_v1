/**
 * Icon Picker Component
 * Reusable searchable Lucide icon picker.
 *
 * Usage:
 *   openIconPicker(currentIconName, callback)
 *   // callback receives the chosen icon name (kebab-case)
 *
 * Attach to an input + optional preview element:
 *   <input id="my-icon" type="text" />
 *   <button type="button" onclick="openIconPicker(
 *       document.getElementById('my-icon').value,
 *       name => { document.getElementById('my-icon').value = name; }
 *   )">Pick</button>
 */

(function () {
    const MODAL_ID = 'icon-picker-modal';
    const GRID_ID  = 'icon-picker-grid';
    const PAGE_SIZE = 168; // multiples of common grid columns

    let _allNames  = null;
    let _callback  = null;

    // Derive all Lucide icon names from the global lucide object at call-time.
    // Icons are exported as Arrays; functions/primitives are the API surface.
    function allIconNames() {
        if (_allNames) return _allNames;
        _allNames = Object.entries(window.lucide)
            .filter(([, v]) => Array.isArray(v))
            .map(([k]) =>
                k
                    // PascalCase → kebab-case
                    .replace(/([A-Z])/g, m => '-' + m.toLowerCase())
                    .replace(/^-/, '')
            )
            .sort();
        return _allNames;
    }

    function buildModal() {
        if (document.getElementById(MODAL_ID)) return;

        const el = document.createElement('dialog');
        el.id = MODAL_ID;
        el.className = 'modal';
        el.innerHTML = `
<div class="modal-box w-11/12 max-w-3xl flex flex-col" style="max-height:80vh">
  <div class="flex items-center justify-between mb-3">
    <h3 class="font-bold text-lg">Pick Icon</h3>
    <form method="dialog">
      <button class="btn btn-sm btn-circle btn-ghost">✕</button>
    </form>
  </div>

  <div class="relative mb-3">
    <i data-lucide="search"
       class="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-base-content/40 pointer-events-none"></i>
    <input id="icon-picker-search"
           type="text"
           placeholder="Search icons…"
           autocomplete="off"
           class="input input-bordered input-sm w-full pl-9"
           oninput="window.__iconPickerFilter(this.value)" />
  </div>

  <p id="icon-picker-hint" class="text-xs text-base-content/40 mb-2"></p>

  <div id="${GRID_ID}"
       class="grid gap-0.5 overflow-y-auto flex-1"
       style="grid-template-columns: repeat(auto-fill, minmax(60px, 1fr))">
    <!-- populated dynamically -->
  </div>
</div>
<form method="dialog" class="modal-backdrop"><button>close</button></form>`;

        document.body.appendChild(el);
    }

    function renderGrid(query) {
        const all      = allIconNames();
        const q        = (query || '').toLowerCase().trim();
        const filtered = q ? all.filter(n => n.includes(q)) : all;
        const shown    = filtered.slice(0, PAGE_SIZE);
        const more     = filtered.length - shown.length;

        const hint = document.getElementById('icon-picker-hint');
        if (hint) {
            hint.textContent = q
                ? `${filtered.length} match${filtered.length !== 1 ? 'es' : ''}${more > 0 ? ` — showing first ${PAGE_SIZE}` : ''}`
                : `${all.length} icons — type to search`;
        }

        const grid = document.getElementById(GRID_ID);
        grid.innerHTML = shown.map(name => `
<button type="button"
        title="${name}"
        onclick="window.__iconPickerSelect('${name}')"
        class="flex flex-col items-center justify-center gap-1.5 p-2 rounded-lg hover:bg-base-200 active:scale-95 transition-all group cursor-pointer"
        style="min-height:56px">
  <i data-lucide="${name}" class="w-5 h-5 text-base-content/60 group-hover:text-primary shrink-0"></i>
  <span style="font-size:9px;line-height:1.1" class="text-base-content/40 truncate w-full text-center">${name}</span>
</button>`).join('');

        lucide.createIcons();
    }

    // Public globals used by inline handlers (avoids closure issues with dialog)
    window.__iconPickerFilter = function (val) { renderGrid(val); };

    window.__iconPickerSelect = function (name) {
        document.getElementById(MODAL_ID).close();
        if (typeof _callback === 'function') _callback(name);
    };

    window.openIconPicker = function (currentIcon, callback) {
        buildModal();
        _callback = callback;

        const search = document.getElementById('icon-picker-search');
        search.value = '';
        renderGrid('');

        // Scroll grid back to top
        const grid = document.getElementById(GRID_ID);
        if (grid) grid.scrollTop = 0;

        lucide.createIcons();
        document.getElementById(MODAL_ID).showModal();

        // Focus search after modal opens
        requestAnimationFrame(() => search.focus());
    };
})();
