// ── Upload ──────────────────────────────────────────────────────────────

async function startUpload(fileList, isFolder) {
    const files = Array.from(fileList);
    if (files.length === 0) return;

    const progress = document.getElementById('upload-progress');
    const bar = document.getElementById('upload-bar');
    const status = document.getElementById('upload-status');
    progress.classList.remove('hidden');

    let done = 0;
    const errors = [];

    for (const file of files) {
        // Determine workspace-relative path:
        //   folder upload → preserve webkitRelativePath (includes top folder name)
        //   file upload   → just the filename, placed in current directory
        const relPath = isFolder && file.webkitRelativePath
            ? pathJoin(CURRENT_PATH, file.webkitRelativePath)
            : pathJoin(CURRENT_PATH, file.name);

        status.textContent = `Uploading ${done + 1} / ${files.length}: ${file.name}`;
        bar.value = Math.round((done / files.length) * 100);

        try {
            const fd = new FormData();
            fd.append('path', relPath);
            fd.append('file', file);

            const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/files/upload`, {
                method: 'POST',
                body: fd,
            });

            if (!r.ok) errors.push(`${file.name}: server error ${r.status}`);
        } catch (e) {
            errors.push(`${file.name}: ${e.message}`);
        }

        done++;
        bar.value = Math.round((done / files.length) * 100);
    }

    // Reset the input so the same folder/files can be selected again if needed
    document.getElementById('upload-files-input').value = '';
    document.getElementById('upload-folder-input').value = '';

    progress.classList.add('hidden');

    if (errors.length > 0) {
        showToast(`${done - errors.length}/${files.length} uploaded. Errors: ${errors.join('; ')}`, 'error');
    } else {
        showToast(`${done} file${done !== 1 ? 's' : ''} uploaded successfully!`, 'success');
    }

    window.location.reload();
}

// ── New File ────────────────────────────────────────────────────────────

const NEW_FILE_TYPES = {
    '.md':          { name: 'notes.md',         hint: 'Markdown document with preview and edit support.' },
    '.mmd':         { name: 'diagram.mmd',       hint: 'Mermaid diagram with live split-pane editor.' },
    '.drawio':      { name: 'diagram.drawio',    hint: 'draw.io diagram, opens in the draw.io editor.' },
    '.bpmn':        { name: 'process.bpmn',      hint: 'BPMN process diagram with bpmn.io editor.' },
    '.excalidraw':  { name: 'sketch.excalidraw', hint: 'Freehand canvas drawing. Supports Apple Pencil.' },
};

function newFile() {
    document.getElementById('new-file-type').value = '';
    document.getElementById('new-file-name').value = '';
    document.getElementById('new-file-type-hint').textContent = '';
    document.getElementById('new-file-modal').showModal();
    lucide.createIcons();
}

function onNewFileTypeChange(type) {
    const nameInput = document.getElementById('new-file-name');
    const hint = document.getElementById('new-file-type-hint');
    if (type && NEW_FILE_TYPES[type]) {
        nameInput.value = NEW_FILE_TYPES[type].name;
        hint.textContent = NEW_FILE_TYPES[type].hint;
    } else {
        hint.textContent = '';
    }
}

async function newFolder() {
    if (folderTypesCache.length === 0) await loadFolderTypes();
    await loadAiProviders();
    await loadGitProviders();

    // Open folder settings modal in create mode
    document.getElementById('folder-settings-mode').value = 'create';
    document.getElementById('folder-settings-title').innerHTML = '<i data-lucide="folder-plus" class="w-5 h-5"></i> Create New Folder';
    document.getElementById('folder-settings-path').value = '';
    document.getElementById('folder-name').value = '';
    document.getElementById('folder-description').value = '';
    document.getElementById('meta-llm-provider').value = '';
    document.getElementById('meta-llm-model').value = '';
    document.getElementById('meta-git-provider').value = '';
    document.getElementById('meta-git-repo').value = '';
    document.getElementById('meta-git-branch').value = '';
    document.getElementById('git-repo-status').classList.add('hidden');
    document.getElementById('git-create-status').classList.add('hidden');
    populateFolderTypeSelect('default');

    // Hide delete button in create mode
    document.getElementById('delete-folder-btn').classList.add('hidden');
    document.getElementById('save-folder-btn').textContent = 'Create Folder';

    applyParentTypeLock();
    updateMetadataFields();
    document.getElementById('folder-settings-modal').showModal();
    lucide.createIcons();
}

function generateBpmnId(prefix) {
    const randomPart = Math.random().toString(36).substring(2, 9);
    return `${prefix}_${randomPart}`;
}

function getBpmnTemplate() {
    const processId = generateBpmnId('Process');
    const definitionsId = generateBpmnId('Definitions');
    const diagramId = generateBpmnId('BPMNDiagram');
    const planeId = generateBpmnId('BPMNPlane');

    return `<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" xmlns:bpmndi="http://www.omg.org/spec/BPMN/20100524/DI" xmlns:dc="http://www.omg.org/spec/DD/20100524/DC" id="${definitionsId}" targetNamespace="http://bpmn.io/schema/bpmn" exporter="bpmn-js (https://demo.bpmn.io)" exporterVersion="18.12.0">
  <bpmn:process id="${processId}" isExecutable="false">
  </bpmn:process>
  <bpmndi:BPMNDiagram id="${diagramId}">
    <bpmndi:BPMNPlane id="${planeId}" bpmnElement="${processId}">
    </bpmndi:BPMNPlane>
  </bpmndi:BPMNDiagram>
</bpmn:definitions>`;
}

function getMmdTemplate() {
    return `flowchart TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Action]
    B -->|No| D[End]
    C --> D`;
}

function getExcalidrawTemplate() {
    return JSON.stringify({
        type: "excalidraw",
        version: 2,
        source: "workspace",
        elements: [],
        appState: { gridSize: null, viewBackgroundColor: "#ffffff" },
        files: {}
    }, null, 2);
}

async function submitNewFile() {
    const name = document.getElementById('new-file-name').value.trim();
    if (!name) return;
    const fullPath = pathJoin(CURRENT_PATH, name);

    let content = null;
    const lname = name.toLowerCase();
    if (lname.endsWith('.bpmn')) {
        content = getBpmnTemplate();
    } else if (lname.endsWith('.mmd') || lname.endsWith('.mermaid')) {
        content = getMmdTemplate();
    } else if (lname.endsWith('.excalidraw')) {
        content = getExcalidrawTemplate();
    }

    try {
        const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/files/new`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
                path: fullPath,
                content: content
            }),
        });
        if (r.ok) {
            document.getElementById('new-file-modal').close();
            window.location.href = `/workspaces/${WORKSPACE_ID}/edit?file=${encodeURIComponent(fullPath)}`;
        } else {
            alert('Failed to create file');
        }
    } catch (e) {
        alert('Network error: ' + e.message);
    }
}
