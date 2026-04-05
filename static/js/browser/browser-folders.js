// ── Folder Type Registry ─────────────────────────────────────────────────

let folderTypesCache = [];
// Type of the directory currently being browsed (empty string = root or default).
// If non-empty, sub-folders here are children of a typed folder and should not get their own type.
let currentDirType = '';

function typeIcon(name) { return ''; }

async function loadFolderTypes() {
    try {
        const r = await fetch('/api/folder-types');
        if (r.ok) {
            folderTypesCache = await r.json();
            populateFolderTypeSelect();
        }
    } catch (e) {
        console.error('Failed to load folder types:', e);
    }
}

let aiProvidersLoaded = false;
async function loadAiProviders() {
    if (aiProvidersLoaded) return;
    try {
        const r = await fetch('/api/llm/providers');
        if (r.ok) {
            const providers = await r.json();
            const sel = document.getElementById('meta-llm-provider');
            if (sel) {
                providers.forEach(p => {
                    const opt = document.createElement('option');
                    opt.value = p.name;
                    opt.textContent = p.name + ' (' + p.provider + ')' + (p.is_default ? ' ★' : '');
                    sel.appendChild(opt);
                });
                aiProvidersLoaded = true;
            }
        }
    } catch (e) {
        console.error('Failed to load AI providers:', e);
    }
}

let gitProvidersLoaded = false;
async function loadGitProviders() {
    if (gitProvidersLoaded) return;
    try {
        const r = await fetch('/api/git/providers');
        if (r.ok) {
            const providers = await r.json();
            const sel = document.getElementById('meta-git-provider');
            if (sel) {
                providers.forEach(p => {
                    const opt = document.createElement('option');
                    opt.value = p.name;
                    opt.textContent = p.name + ' (' + p.provider_type + ')' + (p.is_default ? ' ★' : '');
                    sel.appendChild(opt);
                });
                gitProvidersLoaded = true;
            }
        }
    } catch (e) {
        console.error('Failed to load Git providers:', e);
    }
}

async function checkGitRepo() {
    const providerName = document.getElementById('meta-git-provider').value;
    const repo = document.getElementById('meta-git-repo').value.trim();
    const statusEl = document.getElementById('git-repo-status');

    if (!providerName) { statusEl.textContent = 'Select a provider first'; statusEl.className = 'text-xs mt-1 text-error'; return; }
    if (!repo || !repo.includes('/')) { statusEl.textContent = 'Enter owner/repo format'; statusEl.className = 'text-xs mt-1 text-error'; return; }

    statusEl.textContent = 'Checking...';
    statusEl.className = 'text-xs mt-1 text-base-content/50';
    statusEl.classList.remove('hidden');

    try {
        const r = await fetch('/api/git/repos/check', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ provider_name: providerName, repo }),
        });
        const data = await r.json();
        if (data.exists) {
            statusEl.textContent = 'Repository exists (branch: ' + data.default_branch + ')';
            statusEl.className = 'text-xs mt-1 text-success';
            if (!document.getElementById('meta-git-branch').value) {
                document.getElementById('meta-git-branch').value = data.default_branch;
            }
        } else if (data.error) {
            statusEl.textContent = 'Error: ' + data.error;
            statusEl.className = 'text-xs mt-1 text-error';
        } else {
            statusEl.textContent = 'Repository not found — you can create it below';
            statusEl.className = 'text-xs mt-1 text-warning';
        }
    } catch (e) {
        statusEl.textContent = 'Check failed: ' + e.message;
        statusEl.className = 'text-xs mt-1 text-error';
    }
}

async function createGitRepo() {
    const providerName = document.getElementById('meta-git-provider').value;
    const repo = document.getElementById('meta-git-repo').value.trim();
    const statusEl = document.getElementById('git-create-status');

    if (!providerName) { statusEl.textContent = 'Select a provider first'; statusEl.className = 'text-xs self-center text-error'; statusEl.classList.remove('hidden'); return; }
    if (!repo || !repo.includes('/')) { statusEl.textContent = 'Enter owner/repo format'; statusEl.className = 'text-xs self-center text-error'; statusEl.classList.remove('hidden'); return; }

    statusEl.textContent = 'Creating...';
    statusEl.className = 'text-xs self-center text-base-content/50';
    statusEl.classList.remove('hidden');

    try {
        const folderName = document.getElementById('folder-name').value.trim();
        const r = await fetch('/api/git/repos/create', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ provider_name: providerName, repo, description: folderName, private: true }),
        });
        const data = await r.json();
        if (data.success) {
            statusEl.textContent = 'Created! ' + data.html_url;
            statusEl.className = 'text-xs self-center text-success';
            if (!document.getElementById('meta-git-branch').value) {
                document.getElementById('meta-git-branch').value = 'main';
            }
        } else {
            statusEl.textContent = 'Failed: ' + data.error;
            statusEl.className = 'text-xs self-center text-error';
        }
    } catch (e) {
        statusEl.textContent = 'Create failed: ' + e.message;
        statusEl.className = 'text-xs self-center text-error';
    }
}

const _defaultType = { id: 'default', name: 'Default', icon: 'folder', description: 'Regular file storage', color: null };

function _allFolderTypes() {
    return [_defaultType, ...folderTypesCache];
}

function _findType(typeId) {
    return _allFolderTypes().find(t => t.id === typeId) || _defaultType;
}

function populateFolderTypeSelect(preserveValue) {
    const hidden = document.getElementById('folder-type');
    const grid = document.getElementById('folder-type-grid');
    const prev = preserveValue ?? hidden.value;

    grid.innerHTML = '';
    for (const def of _allFolderTypes()) {
        const tile = document.createElement('button');
        tile.type = 'button';
        tile.dataset.typeId = def.id;
        const isSelected = def.id === prev;
        const iconColor = def.color || '#6b7280';
        tile.className = 'folder-type-tile flex flex-col items-center gap-1.5 p-3 rounded-lg border-2 cursor-pointer transition-all text-center hover:shadow-md'
            + (isSelected ? ' border-primary bg-primary/10 shadow-sm' : ' border-base-300 bg-base-100 hover:border-base-content/30');
        tile.innerHTML = `<span class="inline-flex items-center justify-center w-9 h-9 rounded-lg" style="background:${iconColor}20;"><i data-lucide="${def.icon || 'folder'}" class="w-5 h-5" style="color:${iconColor}"></i></span>`
            + `<span class="text-xs font-semibold leading-tight">${def.name}</span>`
            + `<span class="text-[10px] leading-tight text-base-content/50 line-clamp-2">${def.description || ''}</span>`;
        tile.onclick = () => selectFolderType(def.id);
        grid.appendChild(tile);
    }

    hidden.value = prev;
    updateFolderTypeCard(prev);
    if (typeof lucide !== 'undefined') lucide.createIcons();
}

function selectFolderType(typeId) {
    const hidden = document.getElementById('folder-type');
    hidden.value = typeId;
    // Update tile highlights
    document.querySelectorAll('.folder-type-tile').forEach(t => {
        const sel = t.dataset.typeId === typeId;
        t.className = 'folder-type-tile flex flex-col items-center gap-1.5 p-3 rounded-lg border-2 cursor-pointer transition-all text-center hover:shadow-md'
            + (sel ? ' border-primary bg-primary/10 shadow-sm' : ' border-base-300 bg-base-100 hover:border-base-content/30');
    });
    // Update the compact card and collapse the picker
    updateFolderTypeCard(typeId);
    document.getElementById('folder-type-picker').classList.add('hidden');
    const changeBtn = document.getElementById('folder-type-change-btn');
    if (changeBtn) changeBtn.textContent = 'Change';
    updateMetadataFields();
}

function updateFolderTypeCard(typeId) {
    const def = _findType(typeId);
    const iconColor = def.color || '#6b7280';
    const iconEl = document.getElementById('folder-type-icon');
    iconEl.style.background = `${iconColor}20`;
    iconEl.innerHTML = `<i data-lucide="${def.icon || 'folder'}" class="w-5 h-5" style="color:${iconColor}"></i>`;
    document.getElementById('folder-type-name').textContent = def.name;
    document.getElementById('folder-type-desc').textContent = def.description || '';
    if (typeof lucide !== 'undefined') lucide.createIcons();
}

function toggleFolderTypePicker() {
    const picker = document.getElementById('folder-type-picker');
    const btn = document.getElementById('folder-type-change-btn');
    const isOpen = !picker.classList.contains('hidden');
    picker.classList.toggle('hidden');
    if (btn) btn.textContent = isOpen ? 'Change' : 'Cancel';
    if (!isOpen && typeof lucide !== 'undefined') lucide.createIcons();
}

function pathJoin(...parts) {
    return parts.filter(p => p).join('/');
}

// Preload folder types and current directory type on page load
loadFolderTypes();
if (CURRENT_PATH) {
    fetch(`/api/workspaces/${WORKSPACE_ID}/folder-config?path=${encodeURIComponent(CURRENT_PATH)}`)
        .then(r => r.ok ? r.json() : null)
        .then(config => {
            if (config && config.type && config.type !== 'default') {
                currentDirType = config.type;
            }
        })
        .catch(() => {});
}
