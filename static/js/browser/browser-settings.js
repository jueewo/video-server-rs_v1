// ── Folder Settings ─────────────────────────────────────────────────────

async function openFolderSettings(folderPath, folderName) {
    event.stopPropagation(); // Prevent navigation to folder

    // Set edit mode
    document.getElementById('folder-settings-mode').value = 'edit';
    document.getElementById('folder-settings-title').innerHTML = '<i data-lucide="settings" class="w-5 h-5"></i> Folder Settings';
    document.getElementById('delete-folder-btn').classList.remove('hidden');
    document.getElementById('save-folder-btn').textContent = 'Save Settings';

    document.getElementById('folder-settings-path').value = folderPath;
    document.getElementById('folder-name').value = folderName;

    // Ensure folder types, AI providers, and Git providers are loaded
    if (folderTypesCache.length === 0) await loadFolderTypes();
    await loadAiProviders();
    await loadGitProviders();

    // Reset AI fields before populating
    document.getElementById('meta-llm-provider').value = '';
    document.getElementById('meta-llm-model').value = '';

    // Reset Git fields before populating
    document.getElementById('meta-git-provider').value = '';
    document.getElementById('meta-git-repo').value = '';
    document.getElementById('meta-git-branch').value = '';
    document.getElementById('git-repo-status').classList.add('hidden');
    document.getElementById('git-create-status').classList.add('hidden');

    // Load existing folder config from workspace.yaml
    let hasTypedChildren = false;
    try {
        const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/folder-config?path=${encodeURIComponent(folderPath)}`);
        if (r.ok) {
            const config = await r.json();
            hasTypedChildren = !!config.has_typed_children;
            populateFolderTypeSelect(config.type || 'default');
            document.getElementById('folder-description').value = config.description || '';

            // Render metadata fields for this type, then populate saved values
            await updateMetadataFields();

            if (config.metadata) {
                Object.entries(config.metadata).forEach(([key, value]) => {
                    const fieldId = `meta-${key.replace(/_/g, '-')}`;
                    const field = document.getElementById(fieldId);
                    if (field) field.value = value;
                });
            }
        }
    } catch (e) {
        console.error('Failed to load folder config:', e);
    }

    document.getElementById('folder-type-picker').classList.add('hidden');
    applyParentTypeLock();
    applyChildTypeLock(hasTypedChildren);
    document.getElementById('folder-settings-modal').showModal();
    lucide.createIcons();
}

// Locks the type selector when the current directory is itself a typed folder.
// Sub-folders of typed folders should not get their own independent type.
function applyParentTypeLock() {
    const hidden = document.getElementById('folder-type');
    const changeBtn = document.getElementById('folder-type-change-btn');
    const picker = document.getElementById('folder-type-picker');
    const notice = document.getElementById('parent-type-notice');

    if (currentDirType) {
        hidden.disabled = true;
        hidden.value = 'default';
        if (changeBtn) changeBtn.classList.add('hidden');
        if (picker) picker.classList.add('hidden');
        updateFolderTypeCard('default');
        if (notice) {
            notice.innerHTML = `<i data-lucide="lock" class="w-3 h-3 inline-block align-middle mr-1"></i>This folder is inside a "<strong>${currentDirType}</strong>" folder — sub-folders inherit that context and cannot have their own type.`;
            lucide.createIcons({ nodes: [notice] });
            notice.classList.remove('hidden');
        }
    } else {
        hidden.disabled = false;
        if (changeBtn) changeBtn.classList.remove('hidden');
        if (notice) notice.classList.add('hidden');
    }
}

// Locks the type selector when the folder has typed children.
function applyChildTypeLock(hasTypedChildren) {
    if (!hasTypedChildren) return;
    // Don't override parent lock (which is stricter)
    if (currentDirType) return;

    const changeBtn = document.getElementById('folder-type-change-btn');
    const picker = document.getElementById('folder-type-picker');
    const notice = document.getElementById('parent-type-notice');

    if (changeBtn) changeBtn.classList.add('hidden');
    if (picker) picker.classList.add('hidden');
    if (notice) {
        notice.innerHTML = `<i data-lucide="lock" class="w-3 h-3 inline-block align-middle mr-1"></i>This folder has typed subfolders — remove their types first to change this folder's type.`;
        lucide.createIcons({ nodes: [notice] });
        notice.classList.remove('hidden');
    }
}

async function updateMetadataFields() {
    const type = document.getElementById('folder-type').value;
    const container = document.getElementById('metadata-fields');
    const descField = document.getElementById('folder-description');
    const isDefault = type === 'default';
    descField.disabled = isDefault;
    descField.placeholder = isDefault ? 'Not saved for default folders' : 'What is this folder for?';
    if (isDefault) descField.value = '';

    container.innerHTML = '';

    if (isDefault) {
        container.innerHTML = `<div class="alert alert-info text-sm flex gap-2"><i data-lucide="info" class="w-4 h-4 shrink-0 mt-0.5"></i><span>Default folders don't have special metadata. They're just regular file storage.</span></div>`;
        lucide.createIcons({ nodes: [container] });
        return;
    }

    // Media-server folders get a vault picker instead of raw metadata fields
    if (type === 'media-server') {
        container.innerHTML = `
            <div class="gap-1">
                <label class="label pb-0"><span class="font-medium">Vault</span></label>
                <select id="meta-vault-id" class="select select-sm w-full">
                    <option value="">Create new vault automatically</option>
                </select>
                <label class="label pt-0"><span class="text-xs text-base-content/50">Pick an existing vault or leave on "Create new" to auto-provision one.</span></label>
            </div>`;
        try {
            const r = await fetch('/api/user/vaults');
            if (r.ok) {
                const vaults = await r.json();
                const sel = document.getElementById('meta-vault-id');
                if (sel && vaults.length > 0) {
                    vaults.forEach(v => {
                        const opt = document.createElement('option');
                        opt.value = v.vault_id;
                        opt.textContent = v.vault_name || v.vault_id;
                        sel.appendChild(opt);
                    });
                }
            }
        } catch {}
        return;
    }

    const def = folderTypesCache.find(d => d.id === type);
    if (!def || !def.metadata_schema || def.metadata_schema.length === 0) return;

    const rows = def.metadata_schema.map(field => {
        const id = `meta-${field.key.replace(/_/g, '-')}`;
        const optLabel = field.required ? field.label
            : `${field.label} <span class="text-base-content/50">(optional)</span>`;
        const defVal = field.default != null ? String(field.default) : '';

        let input;
        switch (field.type) {
            case 'enum': {
                const opts = (field.values || []).map(v =>
                    `<option value="${v}"${v === defVal ? ' selected' : ''}>${v}</option>`
                ).join('');
                input = `<select id="${id}" class="select select-sm w-full">${opts}</select>`;
                break;
            }
            case 'number':
                input = `<input type="number" id="${id}" value="${defVal}" class="input input-sm w-full" placeholder="${field.label}" />`;
                break;
            case 'boolean': {
                const opts = `<option value="true"${defVal === 'true' ? ' selected' : ''}>Yes</option>`
                           + `<option value="false"${defVal !== 'true' ? ' selected' : ''}>No</option>`;
                input = `<select id="${id}" class="select select-sm w-full">${opts}</select>`;
                break;
            }
            case 'multiline':
                input = `<textarea id="${id}" rows="3" class="textarea textarea-sm w-full" placeholder="${field.label}">${defVal}</textarea>`;
                break;
            default: // string
                input = `<input type="text" id="${id}" value="${defVal}" class="input input-sm w-full" placeholder="${field.label}" />`;
        }
        return `<tr><td class="font-medium w-1/3 align-top pt-3">${optLabel}</td><td>${input}</td></tr>`;
    }).join('');

    container.innerHTML = `<table class="table table-sm"><tbody>${rows}</tbody></table>`;
}

async function saveFolderSettings() {
    const mode = document.getElementById('folder-settings-mode').value;
    const path = document.getElementById('folder-settings-path').value;
    const newName = document.getElementById('folder-name').value.trim();
    const description = document.getElementById('folder-description').value.trim();
    const folderType = document.getElementById('folder-type').value;

    if (!newName) {
        showToast('Folder name cannot be empty', 'error');
        return;
    }

    // Collect metadata from type-specific fields + AI settings
    const metadata = {};
    const containers = [
        document.getElementById('metadata-fields'),
        document.getElementById('ai-settings-section'),
        document.getElementById('git-settings-section'),
    ];

    containers.forEach(container => {
        if (!container) return;
        container.querySelectorAll('input, select, textarea').forEach(el => {
            if (el.id.startsWith('meta-')) {
                const key = el.id.replace('meta-', '').replace(/-/g, '_');
                if (el.value) { // Only include non-empty values
                    metadata[key] = el.value;
                }
            }
        });
    });

    try {
        if (mode === 'create') {
            // Create mode: first create the folder via mkdir API
            const mkdirRes = await fetch(`/api/workspaces/${WORKSPACE_ID}/mkdir`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ path: CURRENT_PATH ? `${CURRENT_PATH}/${newName}` : newName }),
            });

            if (!mkdirRes.ok) {
                if (mkdirRes.status === 409) {
                    showToast('A folder with that name already exists', 'error');
                } else {
                    const error = await mkdirRes.text();
                    showToast('Failed to create folder: ' + error, 'error');
                }
                return;
            }

            // Now set folder metadata (type, description, metadata)
            const fullPath = CURRENT_PATH ? `${CURRENT_PATH}/${newName}` : newName;
            const metadataRes = await fetch(`/api/workspaces/${WORKSPACE_ID}/folder-metadata`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    path: fullPath,
                    new_name: newName,
                    description: description || null,
                    folder_type: folderType,
                    metadata: metadata
                }),
            });

            if (metadataRes.ok) {
                document.getElementById('folder-settings-modal').close();
                showToast('Folder created successfully!', 'success');
                setTimeout(() => window.location.reload(), 500);
            } else {
                const error = await metadataRes.text();
                showToast('Folder created but failed to save settings: ' + error, 'error');
                setTimeout(() => window.location.reload(), 500);
            }
        } else {
            // Edit mode: update existing folder settings
            const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/folder-metadata`, {
                method: 'PATCH',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    path: path,
                    new_name: newName,
                    description: description || null,
                    folder_type: folderType,
                    metadata: metadata
                }),
            });

            if (r.ok) {
                document.getElementById('folder-settings-modal').close();
                showToast('Folder settings saved!', 'success');
                setTimeout(() => window.location.reload(), 500);
            } else if (r.status === 409) {
                showToast('A folder with that name already exists', 'error');
            } else {
                const error = await r.text();
                showToast('Failed to save: ' + error, 'error');
            }
        }
    } catch (e) {
        showToast('Network error: ' + e.message, 'error');
    }
}

function confirmDeleteFolder() {
    const folderName = document.getElementById('folder-name').value;
    if (confirm(`Delete folder "${folderName}" and all its contents? This cannot be undone.`)) {
        deleteFolder();
    }
}

async function deleteFolder() {
    const path = document.getElementById('folder-settings-path').value;

    try {
        const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/files?path=${encodeURIComponent(path)}`, {
            method: 'DELETE',
        });

        if (r.status === 204) {
            document.getElementById('folder-settings-modal').close();
            showToast('Folder deleted', 'success');
            setTimeout(() => window.location.reload(), 500);
        } else {
            const error = await r.text();
            showToast('Failed to delete: ' + error, 'error');
        }
    } catch (e) {
        showToast('Network error: ' + e.message, 'error');
    }
}
