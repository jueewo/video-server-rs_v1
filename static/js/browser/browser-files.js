// ── File context menu ────────────────────────────────────────────────────

(function () {
    const menu = document.getElementById('file-ctx-menu');
    let ctxPath = null, ctxName = null;

    window.showFileMenu = function (event, btn) {
        event.stopPropagation();
        ctxPath = btn.dataset.path;
        ctxName = btn.dataset.name;
        document.getElementById('ctx-media-li').style.display =
            btn.dataset.media === '1' ? '' : 'none';

        // Position below the button, clamped to viewport
        const r = btn.getBoundingClientRect();
        const menuW = 160;
        let left = r.left;
        if (left + menuW > window.innerWidth - 8) left = window.innerWidth - menuW - 8;
        menu.style.top  = (r.bottom + 4) + 'px';
        menu.style.left = left + 'px';
        menu.classList.remove('hidden');
        lucide.createIcons();
    };

    document.addEventListener('click', () => menu.classList.add('hidden'));
    document.addEventListener('keydown', e => { if (e.key === 'Escape') menu.classList.add('hidden'); });

    document.getElementById('ctx-rename').onclick = () => openRenameModal(ctxPath, ctxName);
    document.getElementById('ctx-move').onclick   = () => openMoveModal(ctxPath, ctxName);
    document.getElementById('ctx-copy').onclick      = () => openCopyModal(ctxPath, ctxName);
    document.getElementById('ctx-duplicate').onclick = () => duplicateFile(ctxPath, ctxName);
    document.getElementById('ctx-media').onclick     = () => openSendToMediaModal(ctxPath, ctxName);
    document.getElementById('ctx-delete').onclick    = () => deleteItem(ctxPath, false);
})();

// ── Rename ──────────────────────────────────────────────────────────────

function openRenameModal(filePath, fileName) {
    document.getElementById('rename-file-path').value = filePath;
    document.getElementById('rename-file-name').value = fileName;
    document.getElementById('rename-file-error').classList.add('hidden');
    document.getElementById('rename-file-modal').showModal();
    setTimeout(() => {
        const input = document.getElementById('rename-file-name');
        const dot = input.value.lastIndexOf('.');
        input.setSelectionRange(0, dot > 0 ? dot : input.value.length);
        input.focus();
    }, 50);
}

async function submitRename() {
    const fromPath = document.getElementById('rename-file-path').value;
    const newName  = document.getElementById('rename-file-name').value.trim();
    const errEl    = document.getElementById('rename-file-error');
    if (!newName) return;

    const dir  = fromPath.includes('/') ? fromPath.substring(0, fromPath.lastIndexOf('/') + 1) : '';
    const toPath = dir + newName;

    const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/files/rename`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ from: fromPath, to: toPath }),
    });
    if (r.ok) {
        document.getElementById('rename-file-modal').close();
        location.reload();
    } else {
        const msg = r.status === 409 ? 'A file with that name already exists.'
                  : r.status === 404 ? 'File not found.'
                  : `Rename failed (${r.status}).`;
        errEl.textContent = msg;
        errEl.classList.remove('hidden');
    }
}

// ── Move / Copy ───────────────────────────────────────────────────────────

let _moveSelectedDir = null;

function _setModalOperation(op) {
    document.getElementById('move-file-operation').value = op;
    const isCopy = op === 'copy';
    document.getElementById('move-modal-icon').setAttribute('data-lucide', isCopy ? 'copy' : 'folder-input');
    document.getElementById('move-modal-submit').textContent = isCopy ? 'Copy' : 'Move';
    if (typeof lucide !== 'undefined') lucide.createIcons();
}

let _moveExcludeFilter = null;

// ── Build collapsible tree from flat dir list ────────────────────────
// activePath: if provided, the branch leading to this path is auto-expanded
function _buildDirTree(dirs, excludeFilter, activePath) {
    const list = document.getElementById('move-dir-list');
    list.innerHTML = '';

    // Build tree structure
    const tree = { children: {}, path: '', label: '/ (root)' };
    dirs.forEach(d => {
        if (excludeFilter && excludeFilter(d.path)) return;
        if (d.path === '') return; // root handled separately
        const parts = d.path.split('/');
        let node = tree;
        parts.forEach((part, i) => {
            if (!node.children[part]) {
                node.children[part] = { children: {}, path: parts.slice(0, i + 1).join('/'), label: part };
            }
            node = node.children[part];
        });
    });

    // Render root button
    const rootBtn = document.createElement('button');
    rootBtn.type = 'button';
    rootBtn.className = 'btn btn-xs btn-ghost justify-start text-left w-full font-mono';
    rootBtn.textContent = '/ (root)';
    rootBtn.onclick = () => {
        list.querySelectorAll('button').forEach(b => b.classList.remove('btn-active'));
        rootBtn.classList.add('btn-active');
        _moveSelectedDir = '';
    };
    list.appendChild(rootBtn);

    // Build set of paths that should be auto-expanded (ancestors of activePath)
    const expandPaths = new Set();
    if (activePath) {
        const parts = activePath.split('/');
        for (let i = 1; i <= parts.length; i++) {
            expandPaths.add(parts.slice(0, i).join('/'));
        }
    }

    // Unified recursive renderer
    function _renderTreeInto(node, depth, container) {
        const keys = Object.keys(node.children).sort();
        keys.forEach(key => {
            const child = node.children[key];
            const hasChildren = Object.keys(child.children).length > 0;
            // Expand if: depth 1 (always show top level) OR this node is on the active path
            // Expand: active branch always, first level only if no active path set
            const onActiveBranch = expandPaths.has(child.path);
            const expanded = hasChildren && (onActiveBranch || (depth === 1 && expandPaths.size === 0));
            const isActive = activePath && child.path === activePath;

            const row = document.createElement('div');
            row.style.paddingLeft = (depth * 16) + 'px';
            row.className = 'flex items-center';

            if (hasChildren) {
                const toggle = document.createElement('button');
                toggle.type = 'button';
                toggle.className = 'btn btn-ghost btn-xs px-1 min-h-0 h-5 text-base-content/60 font-mono text-xs';
                toggle.textContent = expanded ? '−' : '+';
                toggle.onclick = (e) => {
                    e.stopPropagation();
                    const sub = row.nextElementSibling;
                    if (sub && sub.dataset.subtree === child.path) {
                        const hidden = sub.classList.toggle('hidden');
                        toggle.textContent = hidden ? '+' : '−';
                    }
                };
                row.appendChild(toggle);
            } else {
                const spacer = document.createElement('span');
                spacer.className = 'w-5 inline-block';
                row.appendChild(spacer);
            }

            const btn = document.createElement('button');
            btn.type = 'button';
            btn.className = 'btn btn-xs btn-ghost justify-start text-left flex-1 font-mono gap-1' + (isActive ? ' btn-active' : '');
            btn.innerHTML = `<svg class="w-3 h-3 shrink-0 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>${child.label}`;
            btn.onclick = () => {
                document.getElementById('move-dir-list')
                    .querySelectorAll('button')
                    .forEach(b => b.classList.remove('btn-active'));
                btn.classList.add('btn-active');
                _moveSelectedDir = child.path;
            };
            row.appendChild(btn);
            container.appendChild(row);

            if (hasChildren) {
                const subtree = document.createElement('div');
                subtree.dataset.subtree = child.path;
                if (!expanded) subtree.className = 'hidden';
                container.appendChild(subtree);
                _renderTreeInto(child, depth + 1, subtree);
            }
        });
    }

    _renderTreeInto(tree, 1, list);
}

// ── Load dirs for a workspace ───────────────────────────────────────
async function _loadDirsForWorkspace(wsId, excludeFilter, activePath) {
    const list = document.getElementById('move-dir-list');
    list.innerHTML = '<div class="text-xs text-base-content/40 p-2">Loading…</div>';

    const r = await fetch(`/api/workspaces/${wsId}/dirs`);
    if (!r.ok) { list.innerHTML = '<div class="text-xs text-error p-2">Failed to load folders.</div>'; return; }
    const dirs = await r.json();
    _buildDirTree(dirs, excludeFilter, activePath);
}

let _moveActivePath = null;

// ── Workspace selector change ───────────────────────────────────────
async function onMoveWorkspaceChange(wsId) {
    document.getElementById('move-target-workspace').value = wsId;
    _moveSelectedDir = null;
    // When switching to a different workspace, no active path to highlight
    const ap = wsId === WORKSPACE_ID ? _moveActivePath : null;
    await _loadDirsForWorkspace(wsId, wsId === WORKSPACE_ID ? _moveExcludeFilter : null, ap);
}

async function _openMoveOrCopyModal(op, filePath, fileName, excludeFilter) {
    _moveSelectedDir = null;
    _moveExcludeFilter = excludeFilter;
    // Compute the directory containing the file being moved/copied
    _moveActivePath = filePath.includes('/') ? filePath.substring(0, filePath.lastIndexOf('/')) : '';
    _setModalOperation(op);
    const verb = op === 'copy' ? 'Copy' : 'Move';
    document.getElementById('move-modal-title').textContent = `${verb} File`;
    document.getElementById('move-file-path').value = filePath;
    document.getElementById('move-file-name').value = fileName;
    document.getElementById('move-file-label').textContent = fileName;
    document.getElementById('move-file-error').classList.add('hidden');
    document.getElementById('move-target-workspace').value = WORKSPACE_ID;

    const list = document.getElementById('move-dir-list');
    list.innerHTML = '<div class="text-xs text-base-content/40 p-2">Loading…</div>';

    // Load workspace selector
    const sel = document.getElementById('move-workspace-select');
    sel.innerHTML = '<option value="">Loading…</option>';
    document.getElementById('move-file-modal').showModal();

    // Fetch workspaces and dirs in parallel
    const [wsRes, dirRes] = await Promise.all([
        fetch('/api/user/workspaces'),
        fetch(`/api/workspaces/${WORKSPACE_ID}/dirs`),
    ]);

    // Populate workspace dropdown
    if (wsRes.ok) {
        const workspaces = await wsRes.json();
        sel.innerHTML = '';
        workspaces.forEach(ws => {
            const opt = document.createElement('option');
            opt.value = ws.workspace_id;
            opt.textContent = ws.name;
            if (ws.workspace_id === WORKSPACE_ID) opt.selected = true;
            sel.appendChild(opt);
        });
    } else {
        sel.innerHTML = `<option value="${WORKSPACE_ID}">Current workspace</option>`;
    }

    // Populate directory tree
    if (!dirRes.ok) { list.innerHTML = '<div class="text-xs text-error p-2">Failed to load folders.</div>'; return; }
    const dirs = await dirRes.json();
    _buildDirTree(dirs, excludeFilter, _moveActivePath);
}

function openMoveModal(filePath, fileName) {
    const currentDir = filePath.includes('/') ? filePath.substring(0, filePath.lastIndexOf('/')) : '';
    _openMoveOrCopyModal('move', filePath, fileName, d => d === currentDir);
}

function openCopyModal(filePath, fileName) {
    // Copy can go anywhere — no exclusion needed
    _openMoveOrCopyModal('copy', filePath, fileName, null);
}

async function duplicateFile(filePath, fileName) {
    // Build duplicated name: "file_1.ext", "file_1_1.ext", etc.
    const dot = fileName.lastIndexOf('.');
    const base = dot > 0 ? fileName.substring(0, dot) : fileName;
    const ext  = dot > 0 ? fileName.substring(dot) : '';
    const newName = base + '_1' + ext;
    const dir = filePath.includes('/') ? filePath.substring(0, filePath.lastIndexOf('/')) : '';
    const toPath = dir ? dir + '/' + newName : newName;

    const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/files/copy`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ from: filePath, to: toPath }),
    });
    if (r.ok) {
        location.reload();
    } else if (r.status === 409) {
        alert('A file named "' + newName + '" already exists.');
    } else {
        alert('Duplicate failed (' + r.status + ').');
    }
}

function openMoveFolderModal(folderPath, folderName) {
    const currentParent = folderPath.includes('/') ? folderPath.substring(0, folderPath.lastIndexOf('/')) : '';
    _openMoveOrCopyModal('move', folderPath, folderName, d => {
        return d === currentParent || d === folderPath || d.startsWith(folderPath + '/');
    });
    document.getElementById('move-modal-title').textContent = 'Move Folder';
}

async function submitMove() {
    if (_moveSelectedDir === null) return;
    const fromPath  = document.getElementById('move-file-path').value;
    const fileName  = document.getElementById('move-file-name').value;
    const op        = document.getElementById('move-file-operation').value;
    const targetWs  = document.getElementById('move-target-workspace').value;
    const errEl     = document.getElementById('move-file-error');

    const toPath = _moveSelectedDir === '' ? fileName : _moveSelectedDir + '/' + fileName;
    const endpoint = op === 'copy' ? 'copy' : 'rename';

    const body = { from: fromPath, to: toPath };
    if (targetWs && targetWs !== WORKSPACE_ID) {
        body.target_workspace_id = targetWs;
    }

    const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/files/${endpoint}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    });
    if (r.ok) {
        document.getElementById('move-file-modal').close();
        location.reload();
    } else {
        const verb = op === 'copy' ? 'Copy' : 'Move';
        const msg = r.status === 409 ? 'A file with that name already exists there.'
                  : r.status === 404 ? 'File not found.'
                  : `${verb} failed (${r.status}).`;
        errEl.textContent = msg;
        errEl.classList.remove('hidden');
    }
}

// ── Delete ──────────────────────────────────────────────────────────────

async function deleteItem(path, isDir) {
    const kind = isDir ? 'folder' : 'file';
    if (!confirm(`Delete this ${kind}? This cannot be undone.`)) return;
    try {
        const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/files?path=${encodeURIComponent(path)}`, {
            method: 'DELETE',
        });
        if (r.status === 204) {
            window.location.reload();
        } else {
            alert('Failed to delete ' + kind);
        }
    } catch (e) {
        alert('Network error: ' + e.message);
    }
}
