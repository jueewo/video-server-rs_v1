/**
 * media-picker.js
 *
 * Self-contained media picker dialog. Fetches from GET /api/media.
 * Creates a single <dialog> element lazily and reuses it.
 *
 * Usage:
 *   MediaPicker.open(function(url, item) { doSomethingWith(url); });
 *
 * Optional type hint (pre-selects the type filter):
 *   MediaPicker.open(callback, 'image');
 */
var MediaPicker = (function() {
    'use strict';

    var dialog   = null;
    var callback = null;
    var loading  = false;
    var searchTimer = null;

    // ── URL helper ────────────────────────────────────────────────────────────

    function urlForItem(item) {
        if (item.media_type === 'image') return '/media/' + item.slug + '/image.webp';
        return '/media/' + item.slug + '/thumbnail';
    }

    // ── Escape helper ─────────────────────────────────────────────────────────

    function esc(s) {
        return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    }

    // ── Build dialog (once) ───────────────────────────────────────────────────

    function buildDialog() {
        var d = document.createElement('dialog');
        d.className = 'modal';
        d.innerHTML =
            '<div class="modal-box max-w-4xl w-full p-0 flex flex-col max-h-[90vh]">' +

                // header
                '<div class="flex items-center justify-between px-4 py-3 border-b border-base-300 flex-shrink-0">' +
                    '<h3 class="font-bold text-base">Browse Media</h3>' +
                    '<button type="button" class="btn btn-sm btn-ghost btn-square"' +
                        ' onclick="document.querySelector(\'[data-mp]\').close()">✕</button>' +
                '</div>' +

                // toolbar
                '<div class="flex gap-2 px-4 py-2 border-b border-base-300 flex-shrink-0">' +
                    '<input id="mp-search" type="text" placeholder="Search…"' +
                        ' class="input input-bordered input-sm flex-1"' +
                        ' oninput="MediaPicker._onSearch(this.value)">' +
                    '<select id="mp-type" class="select select-bordered select-sm w-32"' +
                        ' onchange="MediaPicker._load(0)">' +
                        '<option value="image">Images</option>' +
                        '<option value="video">Videos</option>' +
                        '<option value="">All media</option>' +
                    '</select>' +
                '</div>' +

                // grid
                '<div class="flex-1 overflow-y-auto p-3">' +
                    '<div id="mp-grid" class="grid grid-cols-4 sm:grid-cols-6 gap-2"></div>' +
                    '<p id="mp-status" class="text-xs text-base-content/40 text-center mt-3 hidden"></p>' +
                '</div>' +

                // footer / pagination
                '<div id="mp-footer" class="flex items-center justify-between px-4 py-2 border-t border-base-300 flex-shrink-0 text-xs text-base-content/50">' +
                    '<span id="mp-count"></span>' +
                    '<div id="mp-pager" class="flex gap-2 items-center"></div>' +
                '</div>' +

            '</div>' +
            '<form method="dialog" class="modal-backdrop"><button>close</button></form>';

        d.setAttribute('data-mp', '');
        document.body.appendChild(d);
        return d;
    }

    // ── Search with debounce ──────────────────────────────────────────────────

    function _onSearch(val) {
        clearTimeout(searchTimer);
        searchTimer = setTimeout(function() { _load(0); }, 300);
    }

    // ── Load page ─────────────────────────────────────────────────────────────

    function _load(page) {
        if (loading) return;
        loading = true;

        var q    = (document.getElementById('mp-search') || {}).value || '';
        var type = (document.getElementById('mp-type')   || {}).value || '';

        var params = new URLSearchParams({
            page_size:  48,
            page:       page,
            sort_by:    'created_at',
            sort_order: 'desc',
        });
        if (q.trim())  params.set('q', q.trim());
        if (type)      params.set('type_filter', type);

        var grid   = document.getElementById('mp-grid');
        var status = document.getElementById('mp-status');
        var count  = document.getElementById('mp-count');
        var pager  = document.getElementById('mp-pager');

        grid.innerHTML = '';
        status.textContent = 'Loading…';
        status.classList.remove('hidden');
        count.textContent  = '';
        pager.innerHTML    = '';

        fetch('/api/media?' + params)
            .then(function(r) { return r.json(); })
            .then(function(data) {
                loading = false;
                status.classList.add('hidden');

                var items = data.items || [];
                if (items.length === 0) {
                    status.textContent = 'No results.';
                    status.classList.remove('hidden');
                    return;
                }

                items.forEach(function(item) {
                    grid.appendChild(buildCard(item));
                });

                count.textContent = items.length + ' of ' + (data.total || items.length);
                buildPager(pager, page, data.total_pages || 1);
            })
            .catch(function() {
                loading = false;
                status.textContent = 'Failed to load media.';
                status.classList.remove('hidden');
            });
    }

    // ── Card ──────────────────────────────────────────────────────────────────

    function buildCard(item) {
        var url = urlForItem(item);
        var btn = document.createElement('button');
        btn.type = 'button';
        btn.title = item.title || item.slug;
        btn.className =
            'group relative aspect-square rounded overflow-hidden ' +
            'border-2 border-transparent hover:border-primary focus:border-primary ' +
            'bg-base-200 transition-colors outline-none';

        var img = document.createElement('img');
        img.src = url;
        img.alt = '';
        img.className = 'w-full h-full object-cover';
        img.onerror = function() { this.style.opacity = '0.2'; };

        var caption = document.createElement('div');
        caption.className =
            'absolute inset-x-0 bottom-0 bg-base-300/85 text-xs px-1 py-0.5 truncate ' +
            'opacity-0 group-hover:opacity-100 transition-opacity';
        caption.textContent = item.title || item.slug;

        btn.appendChild(img);
        btn.appendChild(caption);

        btn.addEventListener('click', function() {
            document.querySelector('[data-mp]').close();
            if (callback) callback(url, item);
        });

        return btn;
    }

    // ── Pagination ────────────────────────────────────────────────────────────

    function buildPager(pager, page, totalPages) {
        if (totalPages <= 1) return;

        if (page > 0) {
            var prev = document.createElement('button');
            prev.type = 'button';
            prev.className = 'btn btn-xs btn-ghost';
            prev.textContent = '← Prev';
            prev.addEventListener('click', function() { _load(page - 1); });
            pager.appendChild(prev);
        }

        var info = document.createElement('span');
        info.textContent = (page + 1) + ' / ' + totalPages;
        pager.appendChild(info);

        if (page < totalPages - 1) {
            var next = document.createElement('button');
            next.type = 'button';
            next.className = 'btn btn-xs btn-ghost';
            next.textContent = 'Next →';
            next.addEventListener('click', function() { _load(page + 1); });
            pager.appendChild(next);
        }
    }

    // ── Public API ────────────────────────────────────────────────────────────

    function open(cb, typeHint) {
        callback = cb;

        if (!dialog) {
            dialog = buildDialog();
        }

        // Reset state
        var searchEl = document.getElementById('mp-search');
        var typeEl   = document.getElementById('mp-type');
        if (searchEl) searchEl.value = '';
        if (typeEl)   typeEl.value   = typeHint || 'image';
        loading = false;
        clearTimeout(searchTimer);

        dialog.showModal();
        _load(0);
    }

    return { open: open, _load: _load, _onSearch: _onSearch };
})();
