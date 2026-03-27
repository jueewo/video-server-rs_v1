// ── Share Folder ─────────────────────────────────────────────────────────

function generateShareCode() {
    const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
    let code = '';
    for (let i = 0; i < 10; i++) code += chars[Math.floor(Math.random() * chars.length)];
    document.getElementById('share-code-input').value = code;
}

function openShareModal(folderPath, folderName) {
    event.stopPropagation();
    document.getElementById('share-folder-path').value = folderPath;
    document.getElementById('share-folder-name').textContent = folderName;
    document.getElementById('share-code-input').value = '';
    document.getElementById('share-description-input').value = '';
    document.getElementById('share-expiry-input').value = '';
    shareResetToCreate();
    document.getElementById('share-folder-modal').showModal();
    shareLoadExistingCodes();
    lucide.createIcons();
}

async function submitShareCode() {
    const folderPath = document.getElementById('share-folder-path').value;
    const code = document.getElementById('share-code-input').value.trim() || null;
    const description = document.getElementById('share-description-input').value.trim() || null;
    const expiresAt = document.getElementById('share-expiry-input').value || null;
    const result = document.getElementById('share-result');

    document.getElementById('share-submit-btn').disabled = true;

    const body = {
        code,
        description,
        expires_at: expiresAt,
        folders: [{ workspace_id: WORKSPACE_ID, folder_path: folderPath }],
    };

    const r = await fetch('/api/workspace-access-codes', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
    });

    result.classList.remove('hidden');
    if (r.ok) {
        const data = await r.json();
        result.innerHTML = `
            <div class="alert alert-success py-2 text-sm">
                <div class="flex-1">
                    <p class="font-semibold mb-1">Code created!</p>
                    <div class="flex items-center gap-2">
                        <code class="font-mono bg-base-100 px-2 py-0.5 rounded text-base-content">${data.code}</code>
                        <button onclick="navigator.clipboard.writeText('${data.code}').then(()=>showToast('Copied!','success'))"
                                class="btn btn-xs btn-ghost gap-1">
                            <i data-lucide="copy" class="w-3 h-3"></i> Copy
                        </button>
                    </div>
                    <p class="mt-1 text-xs opacity-70">Share this code with satellite apps or other users. <a href="/workspace-access-codes" class="link">Manage codes →</a></p>
                </div>
            </div>`;
        lucide.createIcons();
    } else if (r.status === 409) {
        const codeVal = document.getElementById('share-code-input').value.trim();
        result.innerHTML = `<div class="alert alert-warning py-2 text-sm">Code <code class="font-mono">${codeVal}</code> already exists. Use the "Add to existing" tab to add this folder to it.</div>`;
        result.classList.remove('hidden');
        document.getElementById('share-submit-btn').disabled = false;
    } else {
        result.innerHTML = '<div class="alert alert-error py-2 text-sm">Failed to create code.</div>';
        result.classList.remove('hidden');
        document.getElementById('share-submit-btn').disabled = false;
    }
}

function shareResetToCreate() {
    document.getElementById('share-result').classList.add('hidden');
    document.getElementById('share-result').innerHTML = '';
    document.getElementById('share-actions-create').classList.remove('hidden');
    document.getElementById('share-actions-add').classList.add('hidden');
    document.getElementById('share-submit-btn').disabled = false;
    document.getElementById('share-add-btn').disabled = false;
    shareSetMode('new');
}

function shareSetMode(mode) {
    const isNew = mode === 'new';
    document.getElementById('share-mode-new').classList.toggle('hidden', !isNew);
    document.getElementById('share-mode-existing').classList.toggle('hidden', isNew);
    document.getElementById('share-actions-create').classList.toggle('hidden', !isNew);
    document.getElementById('share-actions-add').classList.toggle('hidden', isNew);
    document.getElementById('share-tab-new').classList.toggle('tab-active', isNew);
    document.getElementById('share-tab-existing').classList.toggle('tab-active', !isNew);
    document.getElementById('share-result').classList.add('hidden');
    document.getElementById('share-result').innerHTML = '';
}

async function shareLoadExistingCodes() {
    const sel = document.getElementById('share-existing-select');
    try {
        const r = await fetch('/api/workspace-access-codes');
        if (!r.ok) { sel.innerHTML = '<option value="">— none found —</option>'; return; }
        const codes = await r.json();
        const active = codes.filter(c => c.is_active);
        if (active.length === 0) {
            sel.innerHTML = '<option value="">— no active codes —</option>';
            return;
        }
        sel.innerHTML = '<option value="">Select a code…</option>' +
            active.map(c => `<option value="${c.code}" data-description="${c.description || ''}" data-folders="${c.folder_count}">${c.code}${c.description ? ' — ' + c.description : ''}</option>`).join('');
    } catch {
        sel.innerHTML = '<option value="">— failed to load —</option>';
    }
}

function shareExistingSelected() {
    const sel = document.getElementById('share-existing-select');
    const opt = sel.options[sel.selectedIndex];
    const info = document.getElementById('share-existing-info');
    if (!sel.value) { info.classList.add('hidden'); return; }
    const desc = opt.dataset.description;
    const folders = opt.dataset.folders;
    info.textContent = `${folders} folder(s) already linked${desc ? ' · ' + desc : ''}`;
    info.classList.remove('hidden');
}

async function addFolderToSelectedCode() {
    const code = document.getElementById('share-existing-select').value;
    if (!code) return;
    const folderPath = document.getElementById('share-folder-path').value;
    const result = document.getElementById('share-result');
    document.getElementById('share-add-btn').disabled = true;
    const r = await fetch(`/api/workspace-access-codes/${encodeURIComponent(code)}/folders`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ workspace_id: WORKSPACE_ID, folder_path: folderPath }),
    });
    if (r.ok) {
        result.innerHTML = `<div class="alert alert-success py-2 text-sm">Folder added to <code class="font-mono">${code}</code>. <a href="/workspace-access-codes" class="link">Manage codes →</a></div>`;
        result.classList.remove('hidden');
        lucide.createIcons();
        document.getElementById('share-add-btn').disabled = false;
    } else {
        result.innerHTML = '<div class="alert alert-error py-2 text-sm">Failed to add folder to code.</div>';
        result.classList.remove('hidden');
        document.getElementById('share-add-btn').disabled = false;
    }
}
