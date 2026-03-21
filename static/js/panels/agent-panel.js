/**
 * Agent Panel — Alpine.js component for AI agent interaction in the file browser.
 *
 * Usage (in browser.html):
 *   <div x-data="agentPanel()" x-show="open" ...>
 *
 * Expects `window.browserContext` to provide:
 *   { workspaceId, currentPath, folderTypeId }
 */
function agentPanel() {
    return {
        open: false,
        loading: false,
        agents: [],
        agentRoles: [],
        folderType: null,
        selectedAgent: null,

        // Chat state
        chatHistory: [],
        chatInput: '',
        streaming: false,
        error: '',

        // Provider state (reuses LLM provider infra)
        providers: [],
        selectedProvider: '',
        selectedModel: '',

        // Approval queue
        pendingApproval: null,

        async init() {
            // Expose toggle globally so the toolbar button can call it
            window.toggleAgentPanel = () => this.toggle();
            await this.loadProviders();
        },

        toggle() {
            this.open = !this.open;
            if (this.open && this.agents.length === 0) {
                this.loadAgents();
            }
        },

        async loadProviders() {
            try {
                const resp = await fetch('/api/llm/providers');
                if (resp.ok) {
                    this.providers = await resp.json();
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

        async loadAgents() {
            const ctx = window.browserContext;
            if (!ctx || !ctx.workspaceId) return;

            this.loading = true;
            try {
                const path = ctx.currentPath || '';
                const url = `/api/workspaces/${ctx.workspaceId}/folders/agents?path=${encodeURIComponent(path)}`;
                const resp = await fetch(url);
                if (resp.ok) {
                    const data = await resp.json();
                    this.agents = data.agents || [];
                    this.agentRoles = data.agent_roles || [];
                    this.folderType = data.folder_type || null;
                }
            } catch (e) {
                console.error('Failed to load agents:', e);
            } finally {
                this.loading = false;
            }
        },

        selectAgent(agent) {
            this.selectedAgent = agent;
            this.chatHistory = [];
            this.error = '';
        },

        deselectAgent() {
            this.selectedAgent = null;
            this.chatHistory = [];
            this.error = '';
        },

        async runQuickAction(action) {
            if (!this.selectedAgent) return;
            this.chatInput = action;
            await this.sendMessage();
        },

        async sendMessage() {
            const msg = this.chatInput.trim();
            if (!msg || this.streaming) return;
            if (!this.selectedProvider) {
                this.error = 'No AI provider configured. Add one in Settings > AI Providers.';
                return;
            }

            this.chatInput = '';
            this.error = '';

            // Add user message
            this.chatHistory.push({ role: 'user', content: msg });

            // Build system prompt from agent + folder context
            const systemPrompt = await this._buildSystemPrompt();

            // Build message list
            const messages = [
                { role: 'system', content: systemPrompt },
                ...this.chatHistory.filter(m => m.role !== 'system')
            ];

            // Add assistant placeholder
            this.chatHistory.push({ role: 'assistant', content: '' });
            const assistantIdx = this.chatHistory.length - 1;
            this.streaming = true;

            try {
                const body = {
                    messages,
                    provider_name: this.selectedProvider,
                    model: this.selectedAgent.model || this.selectedModel || undefined,
                    max_tokens: 4096,
                };

                const ctx = window.browserContext;
                if (ctx && ctx.workspaceId) body.workspace_id = ctx.workspaceId;
                if (ctx && ctx.currentPath) body.folder_path = ctx.currentPath;

                const resp = await fetch('/api/llm/chat', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(body),
                });

                if (!resp.ok) {
                    const errText = await resp.text();
                    this.error = 'Request failed: ' + (errText || resp.statusText);
                    this.chatHistory.pop();
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
                        } catch (_e) {
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
                this._scrollChat();
            }
        },

        async _buildSystemPrompt() {
            const agent = this.selectedAgent;
            let prompt = agent.system_prompt || 'You are a helpful AI assistant.';

            // Fetch folder context
            const ctx = window.browserContext;
            if (ctx && ctx.workspaceId) {
                try {
                    const path = ctx.currentPath || '';
                    const url = `/api/workspaces/${ctx.workspaceId}/folders/ai-context?path=${encodeURIComponent(path)}`;
                    const resp = await fetch(url);
                    if (resp.ok) {
                        const data = await resp.json();
                        if (data.ai_instructions) {
                            prompt += '\n\n## Additional Instructions\n' + data.ai_instructions;
                        }
                        if (data.folder_type) {
                            prompt += '\n\n## Folder Type: ' + data.folder_type.name;
                            if (data.folder_type.description) {
                                prompt += '\n' + data.folder_type.description;
                            }
                        }
                        if (data.context_files && data.context_files.length > 0) {
                            prompt += '\n\n## Files in this folder:';
                            for (const f of data.context_files) {
                                const truncated = f.content.length > 3000
                                    ? f.content.slice(0, 3000) + '\n... (truncated)'
                                    : f.content;
                                prompt += '\n\n--- ' + f.path + ' ---\n```\n' + truncated + '\n```';
                            }
                        }
                    }
                } catch (e) {
                    console.error('Failed to fetch AI context:', e);
                }
            }

            return prompt;
        },

        clearChat() {
            this.chatHistory = [];
            this.error = '';
        },

        _scrollChat() {
            this.$nextTick(() => {
                const el = document.getElementById('agent-chat-messages');
                if (el) el.scrollTop = el.scrollHeight;
            });
        },

        // ---- Approval flow (for future ZeroClaw integration) ----

        approveAction() {
            if (this.pendingApproval) {
                // In a real ZeroClaw integration, this would send approval back
                this.pendingApproval = null;
            }
        },

        rejectAction() {
            if (this.pendingApproval) {
                this.pendingApproval = null;
            }
        },

        // ---- Rendering ----

        renderAgentList() {
            if (this.loading) {
                return '<div class="flex justify-center p-8"><span class="loading loading-spinner"></span></div>';
            }
            if (this.agents.length === 0 && this.agentRoles.length === 0) {
                return '<div class="text-center p-6 opacity-50">' +
                    '<p class="text-sm">No agents available for this folder.</p>' +
                    '<p class="text-xs mt-1">Create an agent-collection folder with .md agent files.</p>' +
                    '</div>';
            }

            let html = '';

            // Show agent roles from folder type
            if (this.agentRoles.length > 0) {
                html += '<div class="px-3 pt-2 pb-1"><span class="text-xs font-semibold opacity-60 uppercase tracking-wide">Expected Roles</span></div>';
                for (const role of this.agentRoles) {
                    const matched = this.agents.find(a => a.role === role.role);
                    html += '<div class="px-3 py-1.5 flex items-center gap-2 text-xs">' +
                        '<span class="w-2 h-2 rounded-full ' + (matched ? 'bg-success' : 'bg-base-300') + '"></span>' +
                        '<span class="font-medium">' + this._esc(role.role) + '</span>' +
                        '<span class="opacity-50 truncate flex-1">' + this._esc(role.description) + '</span>' +
                        '</div>';
                }
                html += '<div class="divider my-1 px-3"></div>';
            }

            // Show available agents
            if (this.agents.length > 0) {
                html += '<div class="px-3 pt-1 pb-1"><span class="text-xs font-semibold opacity-60 uppercase tracking-wide">Available Agents</span></div>';
                for (const agent of this.agents) {
                    html += '<button class="w-full text-left px-3 py-2 hover:bg-base-200 transition-colors" ' +
                        '@click="selectAgent(' + JSON.stringify(agent).replace(/"/g, '&quot;') + ')">' +
                        '<div class="flex items-center gap-2">' +
                        '<span class="font-medium text-sm">' + this._esc(agent.name) + '</span>' +
                        '<span class="badge badge-xs badge-ghost">' + this._esc(agent.role) + '</span>' +
                        '</div>' +
                        '<div class="text-xs opacity-50 mt-0.5 truncate">' + this._esc(agent.model) + '</div>' +
                        '</button>';
                }
            }

            return html;
        },

        _esc(str) {
            if (!str) return '';
            return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
        }
    };
}
