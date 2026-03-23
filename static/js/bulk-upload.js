/**
 * bulk-upload.js
 *
 * Multi-file upload dialog. Uploads each file to POST /api/media/upload
 * with auto-detected media type and filename-based title.
 *
 * Usage:
 *   BulkUpload.open();                          // plain bulk upload
 *   BulkUpload.open({ groupId: 5 });            // pre-assign to group
 *   BulkUpload.open({ vaultId: 'abc' });        // pre-select vault
 *   BulkUpload.open({ groupId: 5, onDone: fn }) // callback when finished
 */
var BulkUpload = (function () {
    'use strict';

    var dialog = null;
    var state = {
        files: [],
        uploading: false,
        current: 0,
        total: 0,
        results: [],
        errors: [],
        groupId: null,
        vaultId: null,
        vaults: [],
        onDone: null,
    };

    // ── Helpers ──────────────────────────────────────────────────────────

    function esc(s) {
        return String(s)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;');
    }

    function detectMediaType(file) {
        var type = file.type.toLowerCase();
        if (type.startsWith('video/')) return 'video';
        if (type.startsWith('image/')) return 'image';
        return 'document';
    }

    function titleFromFilename(filename) {
        // Remove extension, replace separators with spaces, trim
        return filename
            .replace(/\.[^.]+$/, '')
            .replace(/[-_]+/g, ' ')
            .replace(/\s+/g, ' ')
            .trim();
    }

    function formatSize(bytes) {
        if (bytes < 1024) return bytes + ' B';
        if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB';
        return (bytes / 1048576).toFixed(1) + ' MB';
    }

    function typeIcon(mediaType) {
        switch (mediaType) {
            case 'video': return 'film';
            case 'image': return 'image';
            default: return 'file-text';
        }
    }

    // ── Build dialog ─────────────────────────────────────────────────────

    function buildDialog() {
        var d = document.createElement('dialog');
        d.className = 'modal';
        d.setAttribute('data-bulk-upload', '');
        d.innerHTML =
            '<div class="modal-box max-w-2xl w-full p-0 flex flex-col max-h-[85vh]">' +
                // header
                '<div class="flex items-center justify-between px-5 py-3 border-b border-base-300">' +
                    '<h3 class="font-bold text-base flex items-center gap-2">' +
                        '<i data-lucide="upload" class="w-5 h-5"></i> Bulk Upload' +
                    '</h3>' +
                    '<button type="button" class="btn btn-sm btn-ghost btn-square" onclick="BulkUpload.close()">✕</button>' +
                '</div>' +

                // body
                '<div class="px-5 py-4 overflow-y-auto flex-1" id="bu-body">' +

                    // vault selector row
                    '<div class="flex gap-3 mb-4" id="bu-vault-row">' +
                        '<label class="form-control flex-1">' +
                            '<div class="label py-0"><span class="label-text text-xs">Vault</span></div>' +
                            '<select id="bu-vault" class="select select-bordered select-sm w-full"></select>' +
                        '</label>' +
                        '<label class="form-control flex-1" id="bu-group-row">' +
                            '<div class="label py-0"><span class="label-text text-xs">Group (optional)</span></div>' +
                            '<select id="bu-group" class="select select-bordered select-sm w-full">' +
                                '<option value="">None</option>' +
                            '</select>' +
                        '</label>' +
                    '</div>' +

                    // drop zone
                    '<div id="bu-dropzone" class="border-2 border-dashed border-base-300 rounded-xl p-8 text-center cursor-pointer transition-colors hover:border-primary hover:bg-primary/5">' +
                        '<i data-lucide="cloud-upload" class="w-10 h-10 mx-auto text-base-content/30 mb-3"></i>' +
                        '<p class="text-sm text-base-content/50 mb-1">Drag & drop files here</p>' +
                        '<p class="text-xs text-base-content/30">or click to browse</p>' +
                        '<input type="file" id="bu-file-input" multiple class="hidden" />' +
                    '</div>' +

                    // file list
                    '<div id="bu-file-list" class="mt-4 space-y-1 hidden"></div>' +

                    // progress
                    '<div id="bu-progress" class="mt-4 hidden">' +
                        '<div class="flex items-center justify-between text-sm mb-1">' +
                            '<span id="bu-progress-text" class="text-base-content/60">Uploading...</span>' +
                            '<span id="bu-progress-count" class="font-mono text-xs"></span>' +
                        '</div>' +
                        '<progress id="bu-progress-bar" class="progress progress-primary w-full" value="0" max="100"></progress>' +
                    '</div>' +

                    // results
                    '<div id="bu-results" class="mt-4 hidden"></div>' +

                '</div>' +

                // footer
                '<div class="px-5 py-3 border-t border-base-300 flex justify-end gap-2">' +
                    '<button type="button" class="btn btn-ghost btn-sm" onclick="BulkUpload.close()">Cancel</button>' +
                    '<button type="button" id="bu-upload-btn" class="btn btn-primary btn-sm gap-1.5" onclick="BulkUpload.start()" disabled>' +
                        '<i data-lucide="upload" class="w-4 h-4"></i> Upload <span id="bu-upload-count"></span>' +
                    '</button>' +
                '</div>' +
            '</div>' +
            '<form method="dialog" class="modal-backdrop"><button>close</button></form>';

        document.body.appendChild(d);
        dialog = d;

        // Wire up drop zone
        var dropzone = d.querySelector('#bu-dropzone');
        var fileInput = d.querySelector('#bu-file-input');

        dropzone.addEventListener('click', function () { fileInput.click(); });
        fileInput.addEventListener('change', function () {
            addFiles(this.files);
            this.value = '';
        });

        dropzone.addEventListener('dragover', function (e) {
            e.preventDefault();
            this.classList.add('border-primary', 'bg-primary/5');
        });
        dropzone.addEventListener('dragleave', function () {
            this.classList.remove('border-primary', 'bg-primary/5');
        });
        dropzone.addEventListener('drop', function (e) {
            e.preventDefault();
            this.classList.remove('border-primary', 'bg-primary/5');
            addFiles(e.dataTransfer.files);
        });

        // Init lucide icons in dialog
        if (window.lucide) lucide.createIcons({ nodes: [d] });

        return d;
    }

    // ── File management ──────────────────────────────────────────────────

    function addFiles(fileList) {
        for (var i = 0; i < fileList.length; i++) {
            // Avoid duplicates by name+size
            var f = fileList[i];
            var dup = state.files.some(function (existing) {
                return existing.name === f.name && existing.size === f.size;
            });
            if (!dup) state.files.push(f);
        }
        renderFileList();
    }

    function removeFile(index) {
        state.files.splice(index, 1);
        renderFileList();
    }

    function renderFileList() {
        var list = dialog.querySelector('#bu-file-list');
        var btn = dialog.querySelector('#bu-upload-btn');
        var countEl = dialog.querySelector('#bu-upload-count');

        if (state.files.length === 0) {
            list.classList.add('hidden');
            list.innerHTML = '';
            btn.disabled = true;
            countEl.textContent = '';
            return;
        }

        list.classList.remove('hidden');
        btn.disabled = false;
        countEl.textContent = '(' + state.files.length + ')';

        var html = '';
        for (var i = 0; i < state.files.length; i++) {
            var f = state.files[i];
            var mt = detectMediaType(f);
            html +=
                '<div class="flex items-center gap-2 text-sm py-1 px-2 rounded hover:bg-base-200 group" data-idx="' + i + '">' +
                    '<i data-lucide="' + typeIcon(mt) + '" class="w-4 h-4 shrink-0 text-base-content/40"></i>' +
                    '<span class="flex-1 truncate">' + esc(f.name) + '</span>' +
                    '<span class="text-xs text-base-content/30">' + formatSize(f.size) + '</span>' +
                    '<button type="button" class="btn btn-ghost btn-xs btn-square opacity-0 group-hover:opacity-100" onclick="BulkUpload.removeFile(' + i + ')">' +
                        '<i data-lucide="x" class="w-3 h-3"></i>' +
                    '</button>' +
                '</div>';
        }
        list.innerHTML = html;
        if (window.lucide) lucide.createIcons({ nodes: [list] });
    }

    // ── Vault / Group loading ────────────────────────────────────────────

    async function loadVaults() {
        try {
            var res = await fetch('/api/user/vaults');
            var data = await res.json();
            state.vaults = data.vaults || data || [];
        } catch (e) {
            state.vaults = [];
        }

        var select = dialog.querySelector('#bu-vault');
        select.innerHTML = '';
        for (var i = 0; i < state.vaults.length; i++) {
            var v = state.vaults[i];
            var opt = document.createElement('option');
            opt.value = v.vault_id;
            opt.textContent = v.name || v.vault_id;
            if (state.vaultId && v.vault_id === state.vaultId) opt.selected = true;
            else if (!state.vaultId && v.is_default) opt.selected = true;
            select.appendChild(opt);
        }
    }

    async function loadGroups() {
        try {
            var res = await fetch('/api/groups');
            var data = await res.json();
            var groups = data.groups || data || [];
            var select = dialog.querySelector('#bu-group');
            select.innerHTML = '<option value="">None</option>';
            for (var i = 0; i < groups.length; i++) {
                var g = groups[i];
                var opt = document.createElement('option');
                opt.value = g.id;
                opt.textContent = g.name;
                if (state.groupId && g.id == state.groupId) opt.selected = true;
                select.appendChild(opt);
            }
        } catch (e) {
            // Groups not available, hide row
            var row = dialog.querySelector('#bu-group-row');
            if (row) row.classList.add('hidden');
        }
    }

    // ── Upload ───────────────────────────────────────────────────────────

    async function uploadOne(file) {
        var formData = new FormData();
        formData.append('file', file);
        formData.append('title', titleFromFilename(file.name));
        formData.append('media_type', detectMediaType(file));
        formData.append('is_public', '0');
        formData.append('description', '');

        var vaultSelect = dialog.querySelector('#bu-vault');
        if (vaultSelect && vaultSelect.value) {
            formData.append('vault_id', vaultSelect.value);
        }

        var groupSelect = dialog.querySelector('#bu-group');
        if (groupSelect && groupSelect.value) {
            formData.append('group_id', groupSelect.value);
        }

        var res = await fetch('/api/media/upload', {
            method: 'POST',
            body: formData,
        });

        if (!res.ok) {
            var errData = await res.json().catch(function () { return {}; });
            throw new Error(errData.error || 'Upload failed (' + res.status + ')');
        }

        return await res.json();
    }

    // ── Public API ───────────────────────────────────────────────────────

    return {
        open: function (opts) {
            opts = opts || {};
            state.files = [];
            state.uploading = false;
            state.current = 0;
            state.total = 0;
            state.results = [];
            state.errors = [];
            state.groupId = opts.groupId || null;
            state.vaultId = opts.vaultId || null;
            state.onDone = opts.onDone || null;

            if (!dialog) buildDialog();

            // Reset UI
            dialog.querySelector('#bu-file-list').innerHTML = '';
            dialog.querySelector('#bu-file-list').classList.add('hidden');
            dialog.querySelector('#bu-progress').classList.add('hidden');
            dialog.querySelector('#bu-results').classList.add('hidden');
            dialog.querySelector('#bu-dropzone').classList.remove('hidden');
            dialog.querySelector('#bu-upload-btn').disabled = true;
            dialog.querySelector('#bu-upload-btn').classList.remove('hidden');
            dialog.querySelector('#bu-upload-count').textContent = '';

            // Show/hide group row based on context
            var groupRow = dialog.querySelector('#bu-group-row');
            if (opts.groupId) {
                // Pre-selected group — hide the selector
                groupRow.classList.add('hidden');
            } else {
                groupRow.classList.remove('hidden');
            }

            dialog.showModal();
            loadVaults();
            if (!opts.groupId) loadGroups();

            if (window.lucide) lucide.createIcons({ nodes: [dialog] });
        },

        close: function () {
            if (dialog) dialog.close();
            if (state.onDone && state.results.length > 0) {
                state.onDone(state.results, state.errors);
            }
        },

        removeFile: function (index) {
            removeFile(index);
        },

        start: async function () {
            if (state.files.length === 0 || state.uploading) return;
            state.uploading = true;
            state.current = 0;
            state.total = state.files.length;
            state.results = [];
            state.errors = [];

            // Update UI
            var dropzone = dialog.querySelector('#bu-dropzone');
            var uploadBtn = dialog.querySelector('#bu-upload-btn');
            var progressEl = dialog.querySelector('#bu-progress');
            var progressBar = dialog.querySelector('#bu-progress-bar');
            var progressText = dialog.querySelector('#bu-progress-text');
            var progressCount = dialog.querySelector('#bu-progress-count');
            var resultsEl = dialog.querySelector('#bu-results');
            var fileList = dialog.querySelector('#bu-file-list');

            dropzone.classList.add('hidden');
            uploadBtn.disabled = true;
            progressEl.classList.remove('hidden');
            fileList.classList.add('hidden');

            for (var i = 0; i < state.files.length; i++) {
                state.current = i + 1;
                progressText.textContent = 'Uploading: ' + state.files[i].name;
                progressCount.textContent = state.current + ' / ' + state.total;
                progressBar.value = Math.round(((i) / state.total) * 100);

                try {
                    var result = await uploadOne(state.files[i]);
                    state.results.push({ file: state.files[i].name, result: result });
                } catch (err) {
                    state.errors.push({ file: state.files[i].name, error: err.message });
                }
            }

            progressBar.value = 100;

            // Show results
            state.uploading = false;
            uploadBtn.classList.add('hidden');
            progressEl.classList.add('hidden');
            resultsEl.classList.remove('hidden');

            var html = '<div class="space-y-2">';
            html += '<div class="flex items-center gap-2 text-sm font-medium mb-2">';
            html += '<i data-lucide="check-circle" class="w-5 h-5 text-success"></i>';
            html += state.results.length + ' of ' + state.total + ' files uploaded successfully';
            html += '</div>';

            if (state.errors.length > 0) {
                html += '<div class="text-sm text-error mb-2">';
                html += '<i data-lucide="alert-circle" class="w-4 h-4 inline mr-1"></i>';
                html += state.errors.length + ' failed:';
                html += '</div>';
                html += '<div class="text-xs space-y-1 max-h-32 overflow-y-auto">';
                for (var j = 0; j < state.errors.length; j++) {
                    html += '<div class="text-error/80">' + esc(state.errors[j].file) + ': ' + esc(state.errors[j].error) + '</div>';
                }
                html += '</div>';
            }

            html += '<div class="flex justify-end gap-2 mt-4">';
            html += '<button type="button" class="btn btn-ghost btn-sm" onclick="BulkUpload.close()">Close</button>';
            html += '<button type="button" class="btn btn-primary btn-sm" onclick="location.reload()">Done</button>';
            html += '</div>';
            html += '</div>';

            resultsEl.innerHTML = html;
            if (window.lucide) lucide.createIcons({ nodes: [resultsEl] });
        },
    };
})();
