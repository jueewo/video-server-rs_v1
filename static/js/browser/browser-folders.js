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

function populateFolderTypeSelect(preserveValue) {
    const select = document.getElementById('folder-type');
    const prev = preserveValue ?? select.value;
    select.innerHTML = '<option value="default">Default — Regular file storage</option>';
    for (const def of folderTypesCache) {
        const opt = document.createElement('option');
        opt.value = def.id;
        const desc = def.description ? ` — ${def.description}` : '';
        opt.textContent = `${typeIcon(def.icon)} ${def.name}${desc}`;
        select.appendChild(opt);
    }
    if (prev) select.value = prev;
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
