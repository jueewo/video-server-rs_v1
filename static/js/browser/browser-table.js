// ── Table Sorting ──────────────────────────────────────────────────────
let _sortCol = null;
let _sortAsc = true;

function sortTable(col) {
    const tbody = document.querySelector('#files-table tbody');
    if (!tbody) return;

    if (_sortCol === col) {
        _sortAsc = !_sortAsc;
    } else {
        _sortCol = col;
        _sortAsc = true;
    }

    // Update sort icons
    document.querySelectorAll('.sort-icon').forEach(icon => {
        const c = icon.dataset.col;
        if (c === col) {
            icon.setAttribute('data-lucide', _sortAsc ? 'arrow-up' : 'arrow-down');
            icon.style.opacity = '1';
        } else {
            icon.setAttribute('data-lucide', 'arrow-up-down');
            icon.style.opacity = '0.3';
        }
    });
    if (window.lucide) lucide.createIcons();

    const rows = Array.from(tbody.querySelectorAll('tr.browser-file'));
    rows.sort((a, b) => {
        let va, vb;
        switch (col) {
            case 'name':
                va = (a.dataset.name || '').toLowerCase();
                vb = (b.dataset.name || '').toLowerCase();
                return _sortAsc ? va.localeCompare(vb) : vb.localeCompare(va);
            case 'type':
                va = (a.dataset.mime || '').toLowerCase();
                vb = (b.dataset.mime || '').toLowerCase();
                return _sortAsc ? va.localeCompare(vb) : vb.localeCompare(va);
            case 'size':
                va = parseInt(a.dataset.size || '0', 10);
                vb = parseInt(b.dataset.size || '0', 10);
                return _sortAsc ? va - vb : vb - va;
            case 'modified':
                va = getModifiedText(a);
                vb = getModifiedText(b);
                return _sortAsc ? va.localeCompare(vb) : vb.localeCompare(va);
            default:
                return 0;
        }
    });
    rows.forEach(row => tbody.appendChild(row));
}

function getModifiedText(row) {
    // Modified is the 5th td (0=checkbox, 1=name, 2=type, 3=size, 4=modified)
    const tds = row.querySelectorAll('td');
    return (tds[4]?.textContent || '').trim();
}

// ── Browser Filter ──────────────────────────────────────────────────────
let _bfFolderType = 'all';
let _bfFileType = '';

function toggleBrowserFilter() {
    document.getElementById('browser-filter-panel').classList.toggle('hidden');
}
document.addEventListener('click', function(e) {
    const c = document.getElementById('browser-filter-container');
    if (c && !c.contains(e.target)) {
        document.getElementById('browser-filter-panel')?.classList.add('hidden');
    }
});

// De-duplicate folder type buttons (template may emit one per folder of that type)
(function dedupeTypeButtons() {
    const seen = new Set();
    document.querySelectorAll('#folder-type-btns button[data-ftype]').forEach(btn => {
        const ft = btn.dataset.ftype;
        if (seen.has(ft)) { btn.remove(); return; }
        seen.add(ft);
    });
})();

function setFolderTypeFilter(type, btn) {
    _bfFolderType = type;
    document.querySelectorAll('#folder-type-btns button').forEach(b => {
        if (b.dataset.ftype === type) {
            b.className = 'btn btn-xs btn-primary';
        } else {
            b.className = 'btn btn-xs btn-ghost border border-base-300';
        }
    });
    applyBrowserFilters();
}

const FILE_TYPE_MAP = {
    image: /^image\//,
    video: /^video\//,
    markdown: /\.(md|mdx|markdown)$/i,
    diagram: /\.(mmd|mermaid|drawio|excalidraw|bpmn|svg)$/i,
    data: /\.(yaml|yml|json|csv|toml)$/i,
};

function setFileTypeChip(type, btn) {
    _bfFileType = type;
    document.querySelectorAll('#file-type-chips .badge').forEach(b => {
        if (b.dataset.filetype === type) {
            b.classList.remove('badge-ghost');
            b.classList.add('badge-primary');
        } else {
            b.classList.remove('badge-primary');
            b.classList.add('badge-ghost');
        }
    });
    applyBrowserFilters();
}

function clearBrowserFilters() {
    _bfFolderType = 'all';
    _bfFileType = '';
    const search = document.getElementById('browser-search');
    if (search) search.value = '';
    document.querySelectorAll('#folder-type-btns button').forEach(b => {
        b.className = b.dataset.ftype === 'all'
            ? 'btn btn-xs btn-primary'
            : 'btn btn-xs btn-ghost border border-base-300';
    });
    document.querySelectorAll('#file-type-chips .badge').forEach(b => {
        if (b.dataset.filetype === '') {
            b.classList.remove('badge-ghost'); b.classList.add('badge-primary');
        } else {
            b.classList.remove('badge-primary'); b.classList.add('badge-ghost');
        }
    });
    applyBrowserFilters();
}

function applyBrowserFilters() {
    const search = (document.getElementById('browser-search')?.value || '').toLowerCase().trim();
    let visibleFolders = 0, visibleFiles = 0;

    // Filter folders
    document.querySelectorAll('.browser-folder').forEach(el => {
        const name = (el.dataset.name || '').toLowerCase();
        const ftype = el.dataset.ftype || 'untyped';
        const typeMatch = _bfFolderType === 'all' || ftype === _bfFolderType;
        const searchMatch = !search || name.includes(search);
        const show = typeMatch && searchMatch;
        el.style.display = show ? '' : 'none';
        if (show) visibleFolders++;
    });

    // Filter files
    document.querySelectorAll('.browser-file').forEach(el => {
        const name = (el.dataset.name || '').toLowerCase();
        const mime = el.dataset.mime || '';
        let fileTypeMatch = true;
        if (_bfFileType) {
            const regex = FILE_TYPE_MAP[_bfFileType];
            if (regex) {
                fileTypeMatch = _bfFileType === 'image' || _bfFileType === 'video'
                    ? regex.test(mime)
                    : regex.test(name);
            }
        }
        const searchMatch = !search || name.includes(search);
        const show = fileTypeMatch && searchMatch;
        el.style.display = show ? '' : 'none';
        if (show) visibleFiles++;
    });

    // Section visibility
    const foldersSection = document.getElementById('folders-section');
    const filesSection = document.getElementById('files-section');
    if (foldersSection) foldersSection.style.display = visibleFolders > 0 ? '' : 'none';
    if (filesSection) filesSection.style.display = visibleFiles > 0 ? '' : 'none';

    // Filter count
    let filterCount = 0;
    if (_bfFolderType !== 'all') filterCount++;
    if (_bfFileType) filterCount++;
    if (search) filterCount++;

    const badge = document.getElementById('browser-filter-badge');
    if (filterCount > 0) { badge.textContent = filterCount; badge.classList.remove('hidden'); }
    else { badge.classList.add('hidden'); }

    // Summary bar
    const summary = document.getElementById('browser-filter-summary');
    const noResults = document.getElementById('browser-no-results');
    const totalItems = document.querySelectorAll('.browser-folder').length + document.querySelectorAll('.browser-file').length;

    if (filterCount > 0) {
        summary.classList.remove('hidden');
        let badges = '';
        if (_bfFolderType !== 'all') badges += `<span class="badge badge-outline">folder: ${_bfFolderType}</span> `;
        if (_bfFileType) badges += `<span class="badge badge-outline">type: ${_bfFileType}</span> `;
        if (search) badges += `<span class="badge badge-outline">q: ${search}</span> `;
        document.getElementById('browser-summary-badges').innerHTML = badges;
        const total = visibleFolders + visibleFiles;
        document.getElementById('browser-visible-count').textContent = `${total} item${total !== 1 ? 's' : ''}`;
    } else {
        summary.classList.add('hidden');
    }

    if (noResults) {
        noResults.style.display = (visibleFolders + visibleFiles === 0 && totalItems > 0 && filterCount > 0) ? '' : 'none';
    }
}
