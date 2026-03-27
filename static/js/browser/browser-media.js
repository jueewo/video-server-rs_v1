// ── Send to Media Library ────────────────────────────────────────────────

async function openSendToMediaModal(filePath, fileName) {
    document.getElementById('stm-file-path').value = filePath;
    document.getElementById('stm-filename').textContent = fileName;
    document.getElementById('stm-result').classList.add('hidden');

    const listEl = document.getElementById('stm-folder-list');
    listEl.innerHTML = '<span class="loading loading-spinner loading-sm"></span>';

    document.getElementById('send-to-media-modal').showModal();

    try {
        const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/media-folders`);
        if (!r.ok) throw new Error('Failed to load media folders');
        const folders = await r.json();

        if (folders.length === 0) {
            listEl.innerHTML = `<div class="alert alert-warning text-sm">
                <i data-lucide="triangle-alert" class="w-4 h-4"></i>
                <span>No media-server folders found. Create a folder and set its type to <strong>Media Server</strong> first.</span>
            </div>`;
            lucide.createIcons();
            return;
        }

        if (folders.length === 1) {
            // Auto-send without picker
            listEl.innerHTML = `<p class="text-sm text-base-content/70">Sending to <strong>${escHtml(folders[0].folder_name)}</strong>…</p>`;
            await doSendToMedia(folders[0].vault_id, folders[0].folder_name);
            return;
        }

        // Show folder picker
        listEl.innerHTML = '<p class="text-sm text-base-content/60 mb-1">Choose a media folder:</p>' +
            folders.map(f => `
                <button onclick="doSendToMedia('${escHtml(f.vault_id)}', '${escHtml(f.folder_name)}')"
                        class="btn btn-outline btn-sm justify-start gap-2 w-full">
                    <i data-lucide="film" class="w-4 h-4 flex-shrink-0"></i>
                    ${escHtml(f.folder_name)}
                </button>`).join('');
        lucide.createIcons();
    } catch (e) {
        listEl.innerHTML = `<div class="alert alert-error text-sm">${escHtml(e.message)}</div>`;
    }
}

async function doSendToMedia(vaultId, folderName) {
    const filePath = document.getElementById('stm-file-path').value;
    const resultEl = document.getElementById('stm-result');
    const listEl = document.getElementById('stm-folder-list');

    listEl.innerHTML = `<p class="text-sm text-base-content/60">Sending to <strong>${escHtml(folderName)}</strong>…</p>`;
    resultEl.classList.add('hidden');

    try {
        const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/files/publish`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ file_path: filePath, vault_id: vaultId }),
        });
        if (r.ok) {
            const data = await r.json();
            const url = data.media_url;
            listEl.innerHTML = '';
            resultEl.className = 'alert alert-success mb-3';
            resultEl.innerHTML = `<div class="w-full">
                <p class="font-semibold">Sent to media library!</p>
                <p class="text-sm mt-1">
                    <a href="${url}" target="_blank" class="link link-primary">${url}</a>
                    <button onclick="navigator.clipboard.writeText(window.location.origin + '${url}').then(() => showToast('URL copied!', 'success'))"
                            class="btn btn-xs btn-ghost ml-1" title="Copy URL"><i data-lucide="copy" class="w-3 h-3"></i></button>
                </p>
            </div>`;
            resultEl.classList.remove('hidden');
            lucide.createIcons();
            showToast('Sent to media library!', 'success');
        } else {
            const text = await r.text();
            listEl.innerHTML = '';
            resultEl.className = 'alert alert-error mb-3';
            resultEl.textContent = 'Error: ' + text;
            resultEl.classList.remove('hidden');
        }
    } catch (e) {
        listEl.innerHTML = '';
        resultEl.className = 'alert alert-error mb-3';
        resultEl.textContent = 'Network error: ' + e.message;
        resultEl.classList.remove('hidden');
    }
}

function escHtml(s) {
    return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;');
}

// ── CSV Preview ──────────────────────────────────────────────────────────

async function openCsvPreview(filePath) {
    const modal   = document.getElementById('csv-preview-modal');
    const content = document.getElementById('csv-preview-content');
    const meta    = document.getElementById('csv-preview-meta');
    const title   = document.getElementById('csv-modal-title');
    const dlLink  = document.getElementById('csv-download-link');

    const fileName = filePath.split('/').pop();
    title.innerHTML = `<i data-lucide="table-2" class="w-5 h-5"></i> ${escHtml(fileName)}`;
    content.innerHTML = '<div class="flex justify-center py-10"><span class="loading loading-spinner loading-md"></span></div>';
    meta.textContent = '';

    document.getElementById('csv-search').value = '';

    const serveUrl = `/api/workspaces/${WORKSPACE_ID}/files/serve?path=${encodeURIComponent(filePath)}`;
    dlLink.href = serveUrl;
    dlLink.download = fileName;

    modal.showModal();
    lucide.createIcons();

    try {
        const r = await fetch(serveUrl);
        if (!r.ok) throw new Error(`Failed to load file (${r.status})`);
        const text = await r.text();

        const sep = detectCsvSep(text);
        const { headers, rows } = parseCsv(text, sep);

        if (headers.length === 0 && rows.length === 0) {
            content.innerHTML = '<div class="text-center text-base-content/50 py-8">Empty file</div>';
            return;
        }

        const MAX_ROWS = 500;
        const previewRows = rows.slice(0, MAX_ROWS);

        const th = headers.map(h =>
            `<th class="whitespace-nowrap bg-base-200">${escHtml(h)}</th>`
        ).join('');
        const tbody = previewRows.map(row =>
            `<tr>${row.map(cell =>
                `<td class="text-sm max-w-xs overflow-hidden text-ellipsis whitespace-nowrap" title="${escHtml(cell)}">${escHtml(cell)}</td>`
            ).join('')}</tr>`
        ).join('');

        content.innerHTML = `
            <table class="table table-xs table-zebra">
                <thead class="sticky top-0 z-10"><tr>${th}</tr></thead>
                <tbody>${tbody}</tbody>
            </table>`;

        const sepLabel = sep === '\t' ? 'tab' : `'${sep}'`;
        const baseText = `${headers.length} column${headers.length !== 1 ? 's' : ''} · ${rows.length} row${rows.length !== 1 ? 's' : ''}${rows.length > MAX_ROWS ? ` (showing first ${MAX_ROWS})` : ''} · separator: ${sepLabel}`;
        meta.dataset.base = baseText;
        meta.textContent = baseText;
    } catch (e) {
        content.innerHTML = `<div class="alert alert-error text-sm m-4">${escHtml(e.message)}</div>`;
    }
}

/** Detect delimiter by counting occurrences in the first line. */
function detectCsvSep(text) {
    const firstLine = text.split(/\r?\n/)[0] || '';
    const counts = {
        ',': (firstLine.match(/,/g) || []).length,
        ';': (firstLine.match(/;/g) || []).length,
        '\t': (firstLine.match(/\t/g) || []).length,
    };
    return Object.entries(counts).sort((a, b) => b[1] - a[1])[0][0];
}

function parseCsv(text, sep) {
    const lines = text.split(/\r?\n/).filter(l => l.trim() !== '');
    if (lines.length === 0) return { headers: [], rows: [] };
    const headers = parseCsvRow(lines[0], sep);
    const rows = lines.slice(1).map(l => parseCsvRow(l, sep));
    return { headers, rows };
}

function filterCsvRows(query) {
    const tbody = document.querySelector('#csv-preview-content tbody');
    if (!tbody) return;
    const term = query.trim().toLowerCase();
    let visible = 0;
    for (const row of tbody.rows) {
        const match = !term || row.textContent.toLowerCase().includes(term);
        row.style.display = match ? '' : 'none';
        if (match) visible++;
    }
    const meta = document.getElementById('csv-preview-meta');
    const base = meta.dataset.base || '';
    meta.textContent = term ? `${visible} row${visible !== 1 ? 's' : ''} match · ${base}` : base;
}

function parseCsvRow(line, sep) {
    const cells = [];
    let current = '';
    let inQuotes = false;
    for (let i = 0; i < line.length; i++) {
        const ch = line[i];
        if (ch === '"') {
            if (inQuotes && line[i + 1] === '"') { current += '"'; i++; }
            else inQuotes = !inQuotes;
        } else if (ch === sep && !inQuotes) {
            cells.push(current);
            current = '';
        } else {
            current += ch;
        }
    }
    cells.push(current);
    return cells;
}
