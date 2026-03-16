/**
 * json-form-editor.js
 *
 * A generic, dependency-free form editor that renders HTML form fields
 * from a JSON object + a field schema, and keeps both a form view and a
 * raw JSON textarea in sync.
 *
 * Usage
 * ─────
 * 1. Include this script on the page.
 * 2. Create a JsonFormEditor instance, passing config:
 *
 *    var editor = new JsonFormEditor({
 *        // Required: where to render dynamic fields
 *        fieldsContainer: document.getElementById('form-fields'),
 *        // Required: the raw JSON textarea
 *        jsonTextarea: document.getElementById('json-editor'),
 *        // Required: tab button elements
 *        tabForm: document.getElementById('tab-form'),
 *        tabJson: document.getElementById('tab-json'),
 *        // Required: the two view containers
 *        formView: document.getElementById('form-view'),
 *        jsonView: document.getElementById('json-view'),
 *        // Required: map of elementType → field array
 *        schemas: FIELD_SCHEMAS,
 *        // Optional: array of element types that have complex arrays
 *        complexTypes: ['Carousel', 'StatData'],
 *        // Optional: called whenever currentData changes
 *        onChange: function(data) {},
 *    });
 *
 * 3. Open an element for editing:
 *    editor.open(dataObject, elementType);
 *
 * 4. Get the current data (with header fields applied):
 *    var data = editor.getData();
 *
 * Field schema entry
 * ──────────────────
 * { path: 'content.title', type: 'text',       label: 'Title' }
 * { path: 'content.desc',  type: 'text-array', label: 'Description' }
 * { path: 'props.on',      type: 'boolean',    label: 'Active' }
 * { path: 'props.limit',   type: 'number',     label: 'Max items' }
 * { path: 'content.body',  type: 'textarea',   label: 'Body text' }
 *
 * Supported field types: text | textarea | text-array | boolean | number | image
 */

(function(global) {
    'use strict';

    // ── Nested path helpers ───────────────────────────────────────────────────

    function getPath(obj, path) {
        return path.split('.').reduce(function(o, k) {
            return (o !== null && o !== undefined) ? o[k] : undefined;
        }, obj);
    }

    function setPath(obj, path, value) {
        var keys = path.split('.');
        var last = keys.pop();
        var target = keys.reduce(function(o, k) {
            if (o[k] === null || o[k] === undefined || typeof o[k] !== 'object') o[k] = {};
            return o[k];
        }, obj);
        target[last] = value;
    }

    function escHtml(s) {
        return String(s)
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;');
    }

    // ── Field element builders ────────────────────────────────────────────────

    function buildBooleanField(field, value, data) {
        var row = document.createElement('div');
        row.className = 'flex items-center gap-2 h-7';
        var chk = document.createElement('input');
        chk.type = 'checkbox';
        chk.className = 'checkbox checkbox-sm';
        chk.checked = !!value;
        chk.addEventListener('change', function() {
            setPath(data, field.path, this.checked);
        });
        row.appendChild(chk);
        return row;
    }

    function buildNumberField(field, value, data) {
        var inp = document.createElement('input');
        inp.type = 'number';
        inp.className = 'input input-bordered input-sm w-full';
        inp.value = (value !== undefined && value !== null) ? value : '';
        inp.addEventListener('input', function() {
            setPath(data, field.path, this.value === '' ? null : Number(this.value));
        });
        return inp;
    }

    function buildTextArrayField(field, value, data) {
        var arr = Array.isArray(value) ? value.slice() : (value ? [String(value)] : ['']);
        var wrap = document.createElement('div');
        wrap.className = 'space-y-1';

        function syncArray() {
            var inputs = wrap.querySelectorAll('input.jfe-arr-line');
            var vals = [];
            inputs.forEach(function(i) { vals.push(i.value); });
            setPath(data, field.path, vals.length === 1 ? vals[0] : vals);
        }

        function addLine(val) {
            var row = document.createElement('div');
            row.className = 'flex gap-1 items-center';
            var li = document.createElement('input');
            li.type = 'text';
            li.className = 'input input-bordered input-xs flex-1 jfe-arr-line';
            li.value = val;
            li.addEventListener('input', syncArray);
            var rm = document.createElement('button');
            rm.type = 'button';
            rm.className = 'btn btn-xs btn-ghost text-error px-1';
            rm.textContent = '×';
            rm.addEventListener('click', function() { row.remove(); syncArray(); });
            row.appendChild(li);
            row.appendChild(rm);
            wrap.appendChild(row);
        }

        arr.forEach(function(v) { addLine(v); });

        var addBtn = document.createElement('button');
        addBtn.type = 'button';
        addBtn.className = 'btn btn-xs btn-ghost mt-1';
        addBtn.textContent = '+ Add line';
        addBtn.addEventListener('click', function() { addLine(''); syncArray(); });

        var container = document.createElement('div');
        container.appendChild(wrap);
        container.appendChild(addBtn);
        return container;
    }

    function buildTextField(field, value, data) {
        if (field.type === 'textarea') {
            var ta = document.createElement('textarea');
            ta.className = 'textarea textarea-bordered textarea-sm w-full font-mono text-xs';
            ta.rows = 4;
            ta.value = (value !== undefined && value !== null) ? String(value) : '';
            ta.addEventListener('input', function() { setPath(data, field.path, this.value); });
            return ta;
        }
        var inp = document.createElement('input');
        inp.type = 'text';
        inp.className = 'input input-bordered input-sm w-full';
        inp.value = (value !== undefined && value !== null) ? String(value) : '';
        inp.addEventListener('input', function() { setPath(data, field.path, this.value); });
        return inp;
    }

    function buildImageField(field, value, data) {
        var container = document.createElement('div');
        container.className = 'space-y-1';

        // Input + browse button row
        var row = document.createElement('div');
        row.className = 'flex gap-1';

        var inp = document.createElement('input');
        inp.type = 'text';
        inp.className = 'input input-bordered input-sm flex-1 font-mono text-xs';
        inp.placeholder = '/media/slug/image.webp';
        inp.value = (value !== undefined && value !== null) ? String(value) : '';

        var preview = document.createElement('div');
        preview.className = 'mt-1';

        function updatePreview() {
            var src = inp.value.trim();
            if (!src) {
                preview.innerHTML = '';
                return;
            }
            preview.innerHTML =
                '<img src="' + escHtml(src) + '" alt="preview" ' +
                'class="h-16 w-auto rounded border border-base-300 object-contain bg-base-200" ' +
                'onerror="this.parentNode.innerHTML=\'<span class=\\\"text-xs text-base-content/40\\\">preview unavailable</span>\'">';
        }

        inp.addEventListener('input', function() {
            setPath(data, field.path, this.value);
            updatePreview();
        });

        row.appendChild(inp);

        // Browse button — only rendered when MediaPicker is available
        if (typeof MediaPicker !== 'undefined') {
            var browseBtn = document.createElement('button');
            browseBtn.type = 'button';
            browseBtn.className = 'btn btn-sm btn-ghost flex-shrink-0';
            browseBtn.title = 'Browse media vault';
            browseBtn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>';
            browseBtn.addEventListener('click', function() {
                MediaPicker.open(function(url) {
                    inp.value = url;
                    setPath(data, field.path, url);
                    updatePreview();
                }, 'image');
            });
            row.appendChild(browseBtn);
        }

        updatePreview();
        container.appendChild(row);
        container.appendChild(preview);
        return container;
    }

    function buildFieldEl(field, data) {
        var value = getPath(data, field.path);
        var wrap = document.createElement('div');
        wrap.className = 'form-control';

        var lbl = document.createElement('label');
        lbl.className = 'label py-0 pb-1';
        lbl.innerHTML =
            '<span class="label-text text-xs font-medium">' + escHtml(field.label) + '</span>' +
            '<span class="label-text-alt text-xs text-base-content/30 font-mono">' + escHtml(field.path) + '</span>';
        wrap.appendChild(lbl);

        var control;
        switch (field.type) {
            case 'boolean':    control = buildBooleanField(field, value, data);   break;
            case 'number':     control = buildNumberField(field, value, data);    break;
            case 'text-array': control = buildTextArrayField(field, value, data); break;
            case 'image':      control = buildImageField(field, value, data);     break;
            default:           control = buildTextField(field, value, data);      break;
        }
        wrap.appendChild(control);
        return wrap;
    }

    // ── JsonFormEditor class ──────────────────────────────────────────────────

    function JsonFormEditor(config) {
        this.fieldsContainer = config.fieldsContainer;
        this.jsonTextarea    = config.jsonTextarea;
        this.tabForm         = config.tabForm;
        this.tabJson         = config.tabJson;
        this.formView        = config.formView;
        this.jsonView        = config.jsonView;
        this.schemas         = config.schemas || {};
        this.complexTypes    = config.complexTypes || [];
        this.onChange        = config.onChange || null;
        this._tab            = 'form';
        this._data           = null;

        var self = this;
        if (this.tabForm) {
            this.tabForm.addEventListener('click', function() { self.switchTab('form'); });
        }
        if (this.tabJson) {
            this.tabJson.addEventListener('click', function() { self.switchTab('json'); });
        }
    }

    // Flatten legacy content/props nesting into flat structure.
    // YAML-compiled elements use { content: { title, desc, ... }, props: { fullscreen, ... } }
    // but the editor and Astro components prefer flat { title, desc, fullscreen, ... }.
    function flattenElement(obj) {
        if (!obj || typeof obj !== 'object') return obj;
        var flat = {};
        // Copy all top-level keys except content and props
        Object.keys(obj).forEach(function(k) {
            if (k !== 'content' && k !== 'props') flat[k] = obj[k];
        });
        // Merge props first (lower priority), then content (higher priority)
        if (obj.props && typeof obj.props === 'object') {
            Object.keys(obj.props).forEach(function(k) {
                if (flat[k] === undefined || flat[k] === null) flat[k] = obj.props[k];
            });
        }
        if (obj.content && typeof obj.content === 'object') {
            Object.keys(obj.content).forEach(function(k) {
                if (flat[k] === undefined || flat[k] === null) flat[k] = obj.content[k];
            });
        }
        // Recursively flatten nested elements (Sections)
        if (Array.isArray(flat.elements)) {
            flat.elements = flat.elements.map(flattenElement);
        }
        return flat;
    }

    JsonFormEditor.prototype.open = function(dataObj, elementType) {
        // Deep clone so edits don't affect the caller's object until getData()
        this._data = flattenElement(JSON.parse(JSON.stringify(dataObj)));
        this._buildForm(elementType);
        if (this.jsonTextarea) {
            this.jsonTextarea.value = JSON.stringify(this._data, null, 2);
        }
        // Force switch to form tab
        this._tab = 'json';
        this.switchTab('form');
    };

    JsonFormEditor.prototype.getData = function() {
        if (this._tab === 'json' && this.jsonTextarea) {
            try {
                this._data = JSON.parse(this.jsonTextarea.value);
            } catch(e) {
                throw new Error('Invalid JSON: ' + e.message);
            }
        }
        return this._data;
    };

    JsonFormEditor.prototype.switchTab = function(tab) {
        if (tab === this._tab) return;

        if (tab === 'json') {
            if (this.jsonTextarea && this._data) {
                this.jsonTextarea.value = JSON.stringify(this._data, null, 2);
            }
        } else {
            if (this.jsonTextarea) {
                try {
                    this._data = JSON.parse(this.jsonTextarea.value);
                } catch(e) {
                    alert('Invalid JSON: ' + e.message);
                    return;
                }
                this._buildForm(this._data && this._data.element ? this._data.element : '');
            }
        }

        this._tab = tab;
        if (this.formView) this.formView.classList.toggle('hidden', tab !== 'form');
        if (this.jsonView) this.jsonView.classList.toggle('hidden', tab !== 'json');
        if (this.tabForm) this.tabForm.classList.toggle('tab-active', tab === 'form');
        if (this.tabJson) this.tabJson.classList.toggle('tab-active', tab === 'json');
    };

    JsonFormEditor.prototype._buildForm = function(elementType) {
        var container = this.fieldsContainer;
        if (!container) return;
        container.innerHTML = '';

        var schema = this.schemas[elementType];
        if (!schema || schema.length === 0) {
            var msg = document.createElement('p');
            msg.className = 'text-xs text-base-content/50 py-2';
            msg.textContent = 'No form schema for "' + elementType + '". Use the JSON tab.';
            container.appendChild(msg);
            return;
        }

        var data = this._data;
        schema.forEach(function(field) {
            container.appendChild(buildFieldEl(field, data));
        });

        if (this.complexTypes.indexOf(elementType) >= 0) {
            var note = document.createElement('p');
            note.className = 'text-xs text-base-content/40 mt-3 pt-3 border-t border-base-200';
            note.textContent = 'Array data fields (items / data) must be edited in the JSON tab.';
            container.appendChild(note);
        }
    };

    // Export
    global.JsonFormEditor = JsonFormEditor;

})(window);
