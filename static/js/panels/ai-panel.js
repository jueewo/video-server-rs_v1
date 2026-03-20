/**
 * AI Panel — reusable Alpine.js component for LLM-assisted editing.
 *
 * Usage:
 *   <script src="/static/js/panels/ai-panel.js"></script>
 *   <div id="panel-ai" class="insert-tab-content" x-data="aiPanel()" x-html="renderHtml()"></div>
 *
 * Expects `window.activeEditor` to be a Monaco editor instance.
 */
function aiPanel() {
    return {
        // State
        providers: [],
        selectedProvider: '',
        selectedModel: '',
        customPrompt: '',
        result: '',
        streaming: false,
        showResult: false,
        error: '',
        hasSelection: false,
        _selectionListener: null,

        // Chat mode
        chatMode: false,
        chatHistory: [],  // Array of { role, content } for multi-turn
        chatInput: '',

        // Quick action presets
        actions: [
            { label: 'Improve', icon: 'sparkles', prompt: 'Improve the following text. Make it clearer, more concise, and better structured. Preserve the original meaning and tone.' },
            { label: 'Simplify', icon: 'minimize-2', prompt: 'Simplify the following text. Use shorter sentences and simpler vocabulary while preserving the key information.' },
            { label: 'Expand', icon: 'maximize-2', prompt: 'Expand the following text with more detail, examples, and explanation. Keep the same style and tone.' },
            { label: 'Fix Grammar', icon: 'spell-check', prompt: 'Fix grammar, spelling, and punctuation errors in the following text. Do not change the meaning or style.' },
            { label: 'Translate EN', icon: 'languages', prompt: 'Translate the following text to English. Preserve formatting and structure.' },
            { label: 'Translate DE', icon: 'languages', prompt: 'Translate the following text to German. Preserve formatting and structure.' },
        ],

        async init() {
            await this.loadProviders();
            this._pollEditor();
        },

        // Poll until the editor is available, then attach a selection listener
        _pollEditor() {
            const check = () => {
                const ed = window.activeEditor || window.editor;
                if (ed) {
                    this._attachSelectionListener(ed);
                } else {
                    setTimeout(check, 300);
                }
            };
            check();
        },

        _attachSelectionListener(ed) {
            this._updateSelection(ed);
            ed.onDidChangeCursorSelection(() => this._updateSelection(ed));
        },

        _updateSelection(ed) {
            const sel = ed.getSelection();
            this.hasSelection = sel && !sel.isEmpty();
        },

        async loadProviders() {
            try {
                const resp = await fetch('/api/llm/providers');
                if (resp.ok) {
                    this.providers = await resp.json();
                    // Auto-select default
                    const def = this.providers.find(p => p.is_default);
                    if (def) {
                        this.selectedProvider = def.name;
                        this.selectedModel = def.default_model;
                    } else if (this.providers.length > 0) {
                        this.selectedProvider = this.providers[0].name;
                        this.selectedModel = this.providers[0].default_model;
                    }
                }
            } catch (e) {
                console.error('Failed to load AI providers:', e);
            }
        },

        onProviderChange() {
            const p = this.providers.find(p => p.name === this.selectedProvider);
            if (p) this.selectedModel = p.default_model;
        },

        getSelectedText() {
            const ed = window.activeEditor || window.editor;
            if (!ed) return '';
            const selection = ed.getSelection();
            if (selection && !selection.isEmpty()) {
                return ed.getModel().getValueInRange(selection);
            }
            return '';
        },

        async runAction(systemPrompt) {
            const text = this.getSelectedText();
            if (!text.trim()) {
                this.error = 'Select text in the editor first';
                return;
            }
            if (!this.selectedProvider) {
                this.error = 'No AI provider selected. Add one in Settings > AI Providers.';
                return;
            }

            this.error = '';
            this.result = '';
            this.showResult = true;
            this.streaming = true;

            const messages = [
                { role: 'system', content: systemPrompt + '\n\nReturn ONLY the improved text, no explanations or markdown code fences.' },
                { role: 'user', content: text }
            ];

            await this.streamChat(messages);
        },

        async runCustom() {
            if (!this.customPrompt.trim()) {
                this.error = 'Please enter a prompt';
                return;
            }
            if (!this.selectedProvider) {
                this.error = 'No AI provider selected. Add one in Settings > AI Providers.';
                return;
            }

            const text = this.getSelectedText();

            this.error = '';
            this.result = '';
            this.showResult = true;
            this.streaming = true;

            let messages;
            if (text.trim()) {
                // Selection exists — transform the selected text
                messages = [
                    { role: 'system', content: this.customPrompt + '\n\nReturn ONLY the result, no explanations or markdown code fences.' },
                    { role: 'user', content: text }
                ];
            } else {
                // No selection — generate new content to insert at cursor
                messages = [
                    { role: 'system', content: 'You are a helpful writing assistant. Return ONLY the requested content, no explanations or markdown code fences.' },
                    { role: 'user', content: this.customPrompt }
                ];
            }

            await this.streamChat(messages);
        },

        // ---- Chat Mode ----

        _buildRequestBody(messages) {
            const ctx = window.aiContext;
            const body = {
                messages,
                provider_name: this.selectedProvider,
                model: this.selectedModel || undefined,
                max_tokens: 4096,
            };
            if (ctx && ctx.workspaceId) body.workspace_id = ctx.workspaceId;
            if (ctx && ctx.folderPath) body.folder_path = ctx.folderPath;
            return body;
        },

        _buildSystemPrompt() {
            const ctx = window.aiContext;
            let prompt = 'You are a helpful writing assistant. Be concise and direct.';
            if (ctx) {
                const content = typeof ctx.getContent === 'function' ? ctx.getContent() : '';
                const truncated = content.length > 6000 ? content.slice(0, 6000) + '\n... (truncated)' : content;
                prompt += '\n\nYou are currently helping the user edit a file.';
                if (ctx.filename) prompt += '\nFilename: ' + ctx.filename;
                if (ctx.language) prompt += '\nLanguage: ' + ctx.language;
                if (truncated) prompt += '\n\nCurrent file content:\n```\n' + truncated + '\n```';
            }
            return prompt;
        },

        toggleChat() {
            this.chatMode = !this.chatMode;
            if (this.chatMode && this.chatHistory.length === 0) {
                this.chatHistory.push({
                    role: 'system',
                    content: this._buildSystemPrompt()
                });
            }
        },

        async sendChatMessage() {
            const msg = this.chatInput.trim();
            if (!msg) return;
            if (!this.selectedProvider) {
                this.error = 'No AI provider selected.';
                return;
            }

            // Update system prompt with latest file content
            if (this.chatHistory.length > 0 && this.chatHistory[0].role === 'system') {
                this.chatHistory[0].content = this._buildSystemPrompt();
            }

            // Add user message to history
            this.chatHistory.push({ role: 'user', content: msg });
            this.chatInput = '';
            this.error = '';
            this.streaming = true;

            // Add placeholder for assistant response
            this.chatHistory.push({ role: 'assistant', content: '' });
            const assistantIdx = this.chatHistory.length - 1;

            try {
                const body = this._buildRequestBody(this.chatHistory.slice(0, -1));

                const resp = await fetch('/api/llm/chat', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body),
                });

                if (!resp.ok) {
                    const errText = await resp.text();
                    this.error = 'Request failed: ' + (errText || resp.statusText);
                    this.chatHistory.pop(); // remove empty assistant message
                    this.streaming = false;
                    return;
                }

                const reader = resp.body.getReader();
                const decoder = new TextDecoder();
                let buffer = '';

                while (true) {
                    const { done, value } = await reader.read();
                    if (done) break;

                    buffer += decoder.decode(value, { stream: true });
                    const lines = buffer.split('\n');
                    buffer = lines.pop() || '';

                    for (const line of lines) {
                        const trimmed = line.trim();
                        if (!trimmed.startsWith('data: ')) continue;
                        const data = trimmed.slice(6);
                        if (!data) continue;

                        try {
                            const event = JSON.parse(data);
                            if (event.token) {
                                this.chatHistory[assistantIdx].content += event.token;
                            } else if (event.error) {
                                this.error = event.error;
                            }
                        } catch (e) {
                            // Ignore malformed events
                        }
                    }
                }
            } catch (e) {
                this.error = 'Streaming error: ' + e.message;
                if (!this.chatHistory[assistantIdx].content) {
                    this.chatHistory.pop();
                }
            } finally {
                this.streaming = false;
            }
        },

        clearChat() {
            this.chatHistory = [{
                role: 'system',
                content: this._buildSystemPrompt()
            }];
            this.error = '';
        },

        insertChatMessage(content) {
            const ed = window.activeEditor || window.editor;
            if (!ed || !content) return;

            const selection = ed.getSelection();
            if (selection && !selection.isEmpty()) {
                ed.executeEdits('ai-panel', [{
                    range: selection,
                    text: content,
                    forceMoveMarkers: true
                }]);
            } else {
                const position = ed.getPosition();
                if (position) {
                    ed.executeEdits('ai-panel', [{
                        range: new monaco.Range(position.lineNumber, position.column, position.lineNumber, position.column),
                        text: content,
                        forceMoveMarkers: true
                    }]);
                }
            }
            ed.focus();
        },

        // ---- Streaming (non-chat) ----

        async streamChat(messages) {
            try {
                const body = this._buildRequestBody(messages);

                const resp = await fetch('/api/llm/chat', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body),
                });

                if (!resp.ok) {
                    const errText = await resp.text();
                    this.error = 'Request failed: ' + (errText || resp.statusText);
                    this.streaming = false;
                    return;
                }

                const reader = resp.body.getReader();
                const decoder = new TextDecoder();
                let buffer = '';

                while (true) {
                    const { done, value } = await reader.read();
                    if (done) break;

                    buffer += decoder.decode(value, { stream: true });

                    // Parse SSE lines
                    const lines = buffer.split('\n');
                    buffer = lines.pop() || '';

                    for (const line of lines) {
                        const trimmed = line.trim();
                        if (!trimmed.startsWith('data: ')) continue;
                        const data = trimmed.slice(6);
                        if (!data) continue;

                        try {
                            const event = JSON.parse(data);
                            if (event.token) {
                                this.result += event.token;
                            } else if (event.done) {
                                // Streaming complete
                            } else if (event.error) {
                                this.error = event.error;
                            }
                        } catch (e) {
                            // Ignore malformed events
                        }
                    }
                }
            } catch (e) {
                this.error = 'Streaming error: ' + e.message;
            } finally {
                this.streaming = false;
            }
        },

        acceptResult() {
            const ed = window.activeEditor || window.editor;
            if (!ed || !this.result) return;

            const selection = ed.getSelection();
            if (selection && !selection.isEmpty()) {
                // Replace the selection
                ed.executeEdits('ai-panel', [{
                    range: selection,
                    text: this.result,
                    forceMoveMarkers: true
                }]);
            } else {
                // Insert at cursor position
                const position = ed.getPosition();
                if (position) {
                    ed.executeEdits('ai-panel', [{
                        range: new monaco.Range(position.lineNumber, position.column, position.lineNumber, position.column),
                        text: this.result,
                        forceMoveMarkers: true
                    }]);
                }
            }
            ed.focus();
            this.dismissResult();
        },

        dismissResult() {
            this.result = '';
            this.showResult = false;
            this.error = '';
        },

        renderHtml() {
            return '<div class="space-y-3">' +
                // Provider selector
                '<div>' +
                    '<label class="text-xs font-semibold opacity-60 mb-1 block">Provider</label>' +
                    '<select class="select select-xs w-full" x-model="selectedProvider" @change="onProviderChange()">' +
                        '<template x-for="p in providers" :key="p.name">' +
                            '<option :value="p.name" x-text="p.name + \' (\' + p.provider + \')\'"></option>' +
                        '</template>' +
                    '</select>' +
                    '<template x-if="providers.length === 0">' +
                        '<p class="text-xs opacity-50 mt-1">No providers configured. <a href="/settings/llm-providers" class="link link-primary">Add one</a></p>' +
                    '</template>' +
                '</div>' +

                // Model override
                '<div>' +
                    '<label class="text-xs font-semibold opacity-60 mb-1 block">Model</label>' +
                    '<input type="text" class="input input-xs w-full" x-model="selectedModel" placeholder="Model ID" />' +
                '</div>' +

                // Mode toggle
                '<div class="flex gap-1">' +
                    '<button class="btn btn-xs flex-1" :class="!chatMode ? \'btn-primary\' : \'btn-ghost\'" @click="chatMode = false">Actions</button>' +
                    '<button class="btn btn-xs flex-1" :class="chatMode ? \'btn-primary\' : \'btn-ghost\'" @click="toggleChat()">Chat</button>' +
                '</div>' +

                // --- Actions Mode ---
                '<template x-if="!chatMode">' +
                '<div class="space-y-3">' +
                    // Quick actions
                    '<div>' +
                        '<label class="text-xs font-semibold opacity-60 mb-1 block">Quick Actions</label>' +
                        '<div class="flex flex-wrap gap-1">' +
                            '<template x-for="action in actions" :key="action.label">' +
                                '<button class="btn btn-xs btn-outline gap-1" :disabled="streaming || !hasSelection" @click="runAction(action.prompt)">' +
                                    '<span x-text="action.label"></span>' +
                                '</button>' +
                            '</template>' +
                        '</div>' +
                        '<template x-if="!hasSelection">' +
                            '<p class="text-xs opacity-40 mt-1">Select text in the editor to enable</p>' +
                        '</template>' +
                    '</div>' +

                    // Custom prompt
                    '<div>' +
                        '<label class="text-xs font-semibold opacity-60 mb-1 block">Custom Prompt</label>' +
                        '<textarea class="textarea textarea-xs w-full" rows="2" x-model="customPrompt" :placeholder="hasSelection ? \'Describe what to do with the selected text...\' : \'Describe what to generate (inserted at cursor)...\'" :disabled="streaming"></textarea>' +
                        '<button class="btn btn-primary btn-xs w-full mt-1 gap-1" :disabled="streaming || !customPrompt.trim()" @click="runCustom()">' +
                            '<span x-show="!streaming" x-text="hasSelection ? \'Apply to Selection\' : \'Generate & Insert\'"></span>' +
                            '<span x-show="streaming" class="loading loading-spinner loading-xs"></span>' +
                            '<span x-show="streaming">Streaming...</span>' +
                        '</button>' +
                    '</div>' +

                    // Error
                    '<template x-if="error && !chatMode">' +
                        '<div class="text-xs text-error" x-text="error"></div>' +
                    '</template>' +

                    // Result preview
                    '<template x-if="showResult">' +
                        '<div class="border border-base-300 rounded-lg p-2 bg-base-200">' +
                            '<label class="text-xs font-semibold opacity-60 mb-1 block">Result</label>' +
                            '<div class="text-xs max-h-64 overflow-y-auto whitespace-pre-wrap font-mono bg-base-100 p-2 rounded" x-text="result"></div>' +
                            '<div class="flex gap-1 mt-2">' +
                                '<button class="btn btn-success btn-xs flex-1" :disabled="streaming" @click="acceptResult()">Accept</button>' +
                                '<button class="btn btn-ghost btn-xs flex-1" :disabled="streaming" @click="dismissResult()">Dismiss</button>' +
                            '</div>' +
                        '</div>' +
                    '</template>' +
                '</div>' +
                '</template>' +

                // --- Chat Mode ---
                '<template x-if="chatMode">' +
                '<div class="space-y-2">' +
                    // Chat messages
                    '<div class="max-h-72 overflow-y-auto space-y-2 border border-base-300 rounded-lg p-2 bg-base-100">' +
                        '<template x-if="chatHistory.filter(m => m.role !== \'system\').length === 0">' +
                            '<p class="text-xs opacity-40 text-center py-4">Start a conversation with the AI assistant</p>' +
                        '</template>' +
                        '<template x-for="(msg, idx) in chatHistory" :key="idx">' +
                            '<template x-if="msg.role !== \'system\'">' +
                                '<div class="text-xs" :class="msg.role === \'user\' ? \'text-right\' : \'\'">' +
                                    '<div class="inline-block max-w-[90%] rounded-lg p-2" :class="msg.role === \'user\' ? \'bg-primary text-primary-content\' : \'bg-base-200\'">' +
                                        '<div class="whitespace-pre-wrap" x-text="msg.content"></div>' +
                                        '<template x-if="msg.role === \'assistant\' && msg.content">' +
                                            '<button class="btn btn-ghost btn-xs mt-1 opacity-60" @click="insertChatMessage(msg.content)">Insert</button>' +
                                        '</template>' +
                                    '</div>' +
                                '</div>' +
                            '</template>' +
                        '</template>' +
                    '</div>' +

                    // Chat input
                    '<div class="flex gap-1">' +
                        '<input type="text" class="input input-xs flex-1" x-model="chatInput" placeholder="Ask the AI..." :disabled="streaming" @keydown.enter="sendChatMessage()" />' +
                        '<button class="btn btn-primary btn-xs" :disabled="streaming || !chatInput.trim()" @click="sendChatMessage()">' +
                            '<span x-show="!streaming">Send</span>' +
                            '<span x-show="streaming" class="loading loading-spinner loading-xs"></span>' +
                        '</button>' +
                    '</div>' +

                    // Chat controls
                    '<div class="flex justify-between">' +
                        '<button class="btn btn-ghost btn-xs opacity-60" @click="clearChat()">Clear</button>' +
                        '<span class="text-xs opacity-40" x-text="(chatHistory.filter(m => m.role !== \'system\').length) + \' messages\'"></span>' +
                    '</div>' +

                    // Error
                    '<template x-if="error && chatMode">' +
                        '<div class="text-xs text-error" x-text="error"></div>' +
                    '</template>' +
                '</div>' +
                '</template>' +

            '</div>';
        }
    };
}
