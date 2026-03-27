// ── Multi-select ─────────────────────────────────────────────────────────

let _selectMode = false;
const _selected = new Set();

function toggleSelectMode() {
    _selectMode = !_selectMode;
    const btn = document.getElementById('select-mode-btn');
    const cols = document.querySelectorAll('.select-col');
    if (_selectMode) {
        btn.classList.add('btn-active');
        cols.forEach(c => c.classList.remove('hidden'));
    } else {
        btn.classList.remove('btn-active');
        cols.forEach(c => c.classList.add('hidden'));
        clearSelection();
    }
}

function onFileCheckbox(cb) {
    if (cb.checked) {
        _selected.add(cb.dataset.path);
    } else {
        _selected.delete(cb.dataset.path);
    }
    _updateBulkBar();
}

function selectAll(checked) {
    document.querySelectorAll('.file-cb').forEach(cb => {
        cb.checked = checked;
        if (checked) _selected.add(cb.dataset.path);
        else _selected.delete(cb.dataset.path);
    });
    _updateBulkBar();
}

function clearSelection() {
    _selected.clear();
    document.querySelectorAll('.file-cb').forEach(cb => cb.checked = false);
    const allCb = document.getElementById('select-all-cb');
    if (allCb) allCb.checked = false;
    _updateBulkBar();
}

function _updateBulkBar() {
    const bar = document.getElementById('bulk-action-bar');
    const n = _selected.size;
    if (n > 0) {
        document.getElementById('bulk-count').textContent = `${n} selected`;
        bar.classList.remove('hidden');
        lucide.createIcons({ nodes: [bar] });
    } else {
        bar.classList.add('hidden');
    }
}

// ── Bulk Move ────────────────────────────────────────────────────────────

async function openBulkMoveModal() {
    if (_selected.size === 0) return;
    _moveSelectedDir = null;
    _setModalOperation('move');
    document.getElementById('move-modal-title').textContent = `Move ${_selected.size} file${_selected.size > 1 ? 's' : ''}`;
    document.getElementById('move-file-path').value = '__bulk__';
    document.getElementById('move-file-name').value = '';
    document.getElementById('move-file-label').textContent = `${_selected.size} selected file${_selected.size > 1 ? 's' : ''}`;
    document.getElementById('move-file-error').classList.add('hidden');

    const list = document.getElementById('move-dir-list');
    list.innerHTML = '<div class="text-xs text-base-content/40 p-2">Loading…</div>';
    document.getElementById('move-file-modal').showModal();

    const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/dirs`);
    if (!r.ok) { list.innerHTML = '<div class="text-xs text-error p-2">Failed to load folders.</div>'; return; }
    const dirs = await r.json();

    list.innerHTML = '';
    dirs.forEach(d => {
        const btn = document.createElement('button');
        btn.className = 'btn btn-xs btn-ghost justify-start text-left w-full font-mono';
        btn.textContent = d.path === '' ? '/ (root)' : '/' + d.path;
        btn.onclick = () => {
            list.querySelectorAll('button').forEach(b => b.classList.remove('btn-active'));
            btn.classList.add('btn-active');
            _moveSelectedDir = d.path;
        };
        list.appendChild(btn);
    });
}

async function openBulkCopyModal() {
    if (_selected.size === 0) return;
    _moveSelectedDir = null;
    document.getElementById('move-modal-title').textContent = `Copy ${_selected.size} file${_selected.size > 1 ? 's' : ''}`;
    document.getElementById('move-file-path').value = '__bulk_copy__';
    document.getElementById('move-file-name').value = '';
    document.getElementById('move-file-label').textContent = `${_selected.size} selected file${_selected.size > 1 ? 's' : ''}`;
    document.getElementById('move-file-error').classList.add('hidden');
    _setModalOperation('copy');

    const list = document.getElementById('move-dir-list');
    list.innerHTML = '<div class="text-xs text-base-content/40 p-2">Loading…</div>';
    document.getElementById('move-file-modal').showModal();

    const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/dirs`);
    if (!r.ok) { list.innerHTML = '<div class="text-xs text-error p-2">Failed to load folders.</div>'; return; }
    const dirs = await r.json();

    list.innerHTML = '';
    dirs.forEach(d => {
        const btn = document.createElement('button');
        btn.className = 'btn btn-xs btn-ghost justify-start text-left w-full font-mono';
        btn.textContent = d.path === '' ? '/ (root)' : '/' + d.path;
        btn.onclick = () => {
            list.querySelectorAll('button').forEach(b => b.classList.remove('btn-active'));
            btn.classList.add('btn-active');
            _moveSelectedDir = d.path;
        };
        list.appendChild(btn);
    });
}

// Patch submitMove to handle bulk mode
const _origSubmitMove = submitMove;
submitMove = async function () {
    const fromPath = document.getElementById('move-file-path').value;
    const isBulkMove = fromPath === '__bulk__';
    const isBulkCopy = fromPath === '__bulk_copy__';
    if (!isBulkMove && !isBulkCopy) { return _origSubmitMove(); }
    if (_moveSelectedDir === null) return;

    const paths = Array.from(_selected);
    let failed = 0;
    const endpoint = isBulkCopy ? 'copy' : 'rename';

    for (const p of paths) {
        const fileName = p.includes('/') ? p.substring(p.lastIndexOf('/') + 1) : p;
        const toPath = _moveSelectedDir === '' ? fileName : _moveSelectedDir + '/' + fileName;
        const targetWs = document.getElementById('move-target-workspace').value;
        const body = { from: p, to: toPath };
        if (targetWs && targetWs !== WORKSPACE_ID) {
            body.target_workspace_id = targetWs;
        }
        const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/files/${endpoint}`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body),
        });
        if (!r.ok) failed++;
    }

    document.getElementById('move-file-modal').close();
    if (failed > 0) {
        const verb = isBulkCopy ? 'copied' : 'moved';
        alert(`${failed} file(s) could not be ${verb} (conflict or error).`);
    }
    location.reload();
};

// ── Bulk Delete ──────────────────────────────────────────────────────────

async function bulkDelete() {
    const n = _selected.size;
    if (n === 0) return;
    if (!confirm(`Delete ${n} file${n > 1 ? 's' : ''}? This cannot be undone.`)) return;

    const paths = Array.from(_selected);
    let failed = 0;

    for (const p of paths) {
        const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/files?path=${encodeURIComponent(p)}`, {
            method: 'DELETE',
        });
        if (r.status !== 204) failed++;
    }

    if (failed > 0) {
        alert(`${failed} file(s) could not be deleted.`);
    }
    location.reload();
}
