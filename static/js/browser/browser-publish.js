// ── Site Generator ──────────────────────────────────────────────────────

async function generateSite() {
    const btn = document.getElementById('generate-site-btn');
    const originalHtml = btn.innerHTML;
    btn.disabled = true;
    btn.innerHTML = '<span class="loading loading-spinner loading-xs"></span> Publishing...';

    try {
        const r = await fetch(`/api/workspaces/${WORKSPACE_ID}/site/generate`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ folder_path: CURRENT_PATH, build: false, push: true }),
        });

        if (!r.ok) {
            const msg = await r.text();
            showToast('Generation failed: ' + (msg || r.status), 'error');
            return;
        }

        const data = await r.json();
        showToast(data.message || 'Site generated!', 'success');
    } catch (e) {
        showToast('Generation failed: ' + e.message, 'error');
    } finally {
        btn.disabled = false;
        btn.innerHTML = originalHtml;
        if (typeof lucide !== 'undefined') lucide.createIcons();
    }
}

// Stores the existing app_id if one was found for the current folder
let _existingAppId = null;

function _pubTypeLabel() {
    const labels = { course: 'Course', presentation: 'Presentation' };
    return labels[CURRENT_TYPE_ID] || 'App';
}

async function openPublishAsAppModal() {
    const folderName = CURRENT_PATH ? CURRENT_PATH.split('/').pop() : WORKSPACE_ID;
    const typeLabel = _pubTypeLabel();
    document.getElementById('app-title').value = folderName;
    document.getElementById('app-description').value = '';
    document.getElementById('app-access').value = 'public';
    document.getElementById('app-result').classList.add('hidden');
    document.getElementById('app-submit-btn').disabled = false;
    document.getElementById('app-create-new').checked = false;

    // Check for existing publication
    _existingAppId = null;
    const banner = document.getElementById('app-existing-banner');
    banner.classList.add('hidden');
    document.getElementById('app-new-fields').classList.remove('hidden');
    document.getElementById('publish-app-modal-title').textContent = `Publish ${typeLabel}`;
    document.getElementById('app-submit-btn').textContent = 'Publish';

    try {
        const res = await fetch(`/api/publications/find?workspace_id=${encodeURIComponent(WORKSPACE_ID)}&folder_path=${encodeURIComponent(CURRENT_PATH)}`);
        if (res.ok) {
            const existing = await res.json();
            _existingAppId = existing.slug;
            const existingUrl = `/pub/${existing.slug}`;
            const shareUrl = `${window.location.origin}${existingUrl}`;
            document.getElementById('app-existing-link').href = existingUrl;
            document.getElementById('app-existing-link').textContent = shareUrl;
            banner.classList.remove('hidden');
            // Default to update mode
            document.getElementById('app-new-fields').classList.add('hidden');
            document.getElementById('publish-app-modal-title').textContent = `Update Published ${typeLabel}`;
            document.getElementById('app-submit-btn').textContent = 'Update';
        }
    } catch (_) {}

    document.getElementById('publish-app-modal').showModal();
}

function togglePublishMode() {
    const createNew = document.getElementById('app-create-new').checked;
    const typeLabel = _pubTypeLabel();
    document.getElementById('app-new-fields').classList.toggle('hidden', !createNew);
    if (createNew) {
        document.getElementById('publish-app-modal-title').textContent = `Publish ${typeLabel}`;
        document.getElementById('app-submit-btn').textContent = 'Publish';
    } else {
        document.getElementById('publish-app-modal-title').textContent = `Update Published ${typeLabel}`;
        document.getElementById('app-submit-btn').textContent = 'Update';
    }
}

async function submitPublishApp() {
    const resultEl = document.getElementById('app-result');
    const submitBtn = document.getElementById('app-submit-btn');
    submitBtn.disabled = true;
    resultEl.classList.add('hidden');

    const isUpdate = _existingAppId && !document.getElementById('app-create-new').checked;

    try {
        let r, data;
        if (isUpdate) {
            r = await fetch(`/api/publications/${_existingAppId}/republish`, { method: 'POST' });
        } else {
            const title = document.getElementById('app-title').value.trim();
            const description = document.getElementById('app-description').value.trim();
            const access = document.getElementById('app-access').value;
            if (!title) { submitBtn.disabled = false; alert('Please enter a title.'); return; }
            // Map folder type to publication type
            const pubTypeMap = { 'course': 'course', 'presentation': 'presentation' };
            const pubType = pubTypeMap[CURRENT_TYPE_ID] || 'app';

            r = await fetch('/api/publications', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    pub_type: pubType,
                    workspace_id: WORKSPACE_ID,
                    folder_path: CURRENT_PATH,
                    title,
                    description,
                    access,
                }),
            });
        }

        if (r.ok) {
            data = await r.json();
            const shareUrl = data.access_code
                ? `${window.location.origin}${data.url}?code=${data.access_code}`
                : `${window.location.origin}${data.url}`;
            const verb = isUpdate ? 'Updated!' : 'Published!';
            resultEl.className = 'alert alert-success mb-3';
            let bundleHtml = '';
            if (data.bundles && data.bundles.length > 0) {
                const items = data.bundles.map(b =>
                    `<li class="flex items-center gap-1.5">
                        <span class="badge badge-xs ${b.access === 'bundled' ? 'badge-accent' : 'badge-success'}">${b.access}</span>
                        <a href="/pub/${b.slug}" target="_blank" class="link text-xs">${b.title}</a>
                     </li>`
                ).join('');
                bundleHtml = `<div class="mt-2 text-sm">
                    <p class="font-medium text-base-content/70">Bundled content (${data.bundles.length}):</p>
                    <ul class="mt-1 space-y-1">${items}</ul>
                </div>`;
            }
            resultEl.innerHTML = `<div class="w-full">
                <p class="font-semibold">${verb}</p>
                <p class="text-sm mt-1 break-all">
                    <a href="${data.url}" target="_blank" class="link">${shareUrl}</a>
                </p>
                ${bundleHtml}
                <div class="flex gap-2 mt-2">
                    <button onclick="navigator.clipboard.writeText('${shareUrl}').then(() => showToast('URL copied!', 'success'))"
                            class="btn btn-xs btn-ghost">Copy link</button>
                    <a href="/my-publications" class="btn btn-xs btn-ghost">My Publications</a>
                </div>
            </div>`;
            resultEl.classList.remove('hidden');
            showToast(verb, 'success');
        } else {
            const text = await r.text();
            resultEl.className = 'alert alert-error mb-3';
            resultEl.textContent = `Error ${r.status}: ${text}`;
            resultEl.classList.remove('hidden');
            submitBtn.disabled = false;
        }
    } catch (e) {
        resultEl.className = 'alert alert-error mb-3';
        resultEl.textContent = 'Network error: ' + e.message;
        resultEl.classList.remove('hidden');
        submitBtn.disabled = false;
    }
}
