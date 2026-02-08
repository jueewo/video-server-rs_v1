/**
 * Tag Picker Component
 * Reusable autocomplete input for adding tags to media items
 *
 * Usage:
 *   <div data-tag-picker
 *        data-api-url="/api/tags/search"
 *        data-selected-tags='["tag1", "tag2"]'>
 *   </div>
 *
 * Features:
 * - Autocomplete with existing tags
 * - Create new tags inline
 * - Multi-select with visual badges
 * - Keyboard navigation (up/down/enter/escape)
 * - Click outside to close
 */

class TagPicker {
    constructor(element) {
        this.element = element;
        this.apiUrl = element.dataset.apiUrl || '/api/tags/search';
        this.selectedTags = JSON.parse(element.dataset.selectedTags || '[]');
        this.onChangeCallback = null;
        this.suggestions = [];
        this.selectedIndex = -1;

        this.init();
    }

    init() {
        this.render();
        this.attachEventListeners();
    }

    render() {
        this.element.innerHTML = `
            <div class="tag-picker">
                <!-- Selected Tags Display -->
                <div class="tag-picker-selected">
                    ${this.renderSelectedTags()}
                </div>

                <!-- Input Container -->
                <div class="tag-picker-input-container">
                    <input
                        type="text"
                        class="tag-picker-input"
                        placeholder="Add tags..."
                        autocomplete="off"
                    />
                </div>

                <!-- Suggestions Dropdown -->
                <div class="tag-picker-suggestions" style="display: none;">
                    <div class="tag-picker-suggestions-list"></div>
                </div>

                <!-- Hidden input for form submission -->
                <input
                    type="hidden"
                    name="tags"
                    value='${JSON.stringify(this.selectedTags)}'
                />
            </div>
        `;

        this.input = this.element.querySelector('.tag-picker-input');
        this.suggestionsContainer = this.element.querySelector('.tag-picker-suggestions');
        this.suggestionsList = this.element.querySelector('.tag-picker-suggestions-list');
        this.hiddenInput = this.element.querySelector('input[name="tags"]');
    }

    renderSelectedTags() {
        if (this.selectedTags.length === 0) {
            return '<div class="text-gray-500 text-sm">No tags selected</div>';
        }

        return this.selectedTags.map(tag => `
            <span class="tag-badge">
                <span class="tag-name">${this.escapeHtml(tag)}</span>
                <button
                    type="button"
                    class="tag-remove"
                    data-tag="${this.escapeHtml(tag)}"
                    aria-label="Remove tag"
                >
                    ×
                </button>
            </span>
        `).join('');
    }

    attachEventListeners() {
        // Input events
        this.input.addEventListener('input', this.handleInput.bind(this));
        this.input.addEventListener('keydown', this.handleKeydown.bind(this));
        this.input.addEventListener('focus', this.handleFocus.bind(this));

        // Remove tag clicks
        this.element.addEventListener('click', (e) => {
            if (e.target.classList.contains('tag-remove')) {
                const tag = e.target.dataset.tag;
                this.removeTag(tag);
            }
        });

        // Click outside to close
        document.addEventListener('click', (e) => {
            if (!this.element.contains(e.target)) {
                this.hideSuggestions();
            }
        });

        // Suggestion clicks
        this.suggestionsList.addEventListener('click', (e) => {
            const item = e.target.closest('.tag-suggestion-item');
            if (item) {
                const tag = item.dataset.tag;
                const isNew = item.dataset.isNew === 'true';
                this.addTag(tag, isNew);
            }
        });
    }

    async handleInput(e) {
        const query = this.input.value.trim();

        if (query.length === 0) {
            this.hideSuggestions();
            return;
        }

        if (query.length < 2) {
            // Show "keep typing" message
            this.showSuggestions([{
                type: 'message',
                text: 'Type at least 2 characters...'
            }]);
            return;
        }

        // Fetch suggestions
        await this.fetchSuggestions(query);
    }

    async fetchSuggestions(query) {
        try {
            const url = `${this.apiUrl}?q=${encodeURIComponent(query)}&limit=10`;
            const response = await fetch(url);

            if (!response.ok) {
                throw new Error('Failed to fetch suggestions');
            }

            const data = await response.json();
            this.suggestions = data.tags || [];

            // Filter out already selected tags
            this.suggestions = this.suggestions.filter(
                tag => !this.selectedTags.includes(tag.slug)
            );

            // Add "create new" option if query doesn't match exactly
            const exactMatch = this.suggestions.find(
                tag => tag.slug === query.toLowerCase().replace(/\s+/g, '-')
            );

            if (!exactMatch && query.length >= 2) {
                this.suggestions.push({
                    slug: query.toLowerCase().replace(/\s+/g, '-'),
                    name: query,
                    isNew: true
                });
            }

            this.showSuggestions(this.suggestions);

        } catch (error) {
            console.error('Error fetching tag suggestions:', error);
            this.showSuggestions([{
                type: 'error',
                text: 'Failed to load suggestions'
            }]);
        }
    }

    showSuggestions(items) {
        if (items.length === 0) {
            this.suggestionsList.innerHTML = `
                <div class="tag-suggestion-empty">
                    No matching tags found
                </div>
            `;
        } else if (items[0].type === 'message' || items[0].type === 'error') {
            this.suggestionsList.innerHTML = `
                <div class="tag-suggestion-message ${items[0].type}">
                    ${this.escapeHtml(items[0].text)}
                </div>
            `;
        } else {
            this.suggestionsList.innerHTML = items.map((item, index) => {
                const isNew = item.isNew || false;
                const icon = isNew ? '✨' : '🏷️';
                const label = isNew ? 'Create new tag' : '';
                const usageCount = item.usage_count ? `(${item.usage_count} uses)` : '';

                return `
                    <div
                        class="tag-suggestion-item ${index === this.selectedIndex ? 'selected' : ''}"
                        data-tag="${this.escapeHtml(item.slug)}"
                        data-is-new="${isNew}"
                    >
                        <span class="tag-icon">${icon}</span>
                        <div class="tag-info">
                            <div class="tag-name-row">
                                <span class="tag-name">${this.escapeHtml(item.name)}</span>
                                ${label ? `<span class="tag-label">${label}</span>` : ''}
                            </div>
                            ${usageCount ? `<div class="tag-usage">${usageCount}</div>` : ''}
                        </div>
                    </div>
                `;
            }).join('');
        }

        this.suggestionsContainer.style.display = 'block';
        this.selectedIndex = -1;
    }

    hideSuggestions() {
        this.suggestionsContainer.style.display = 'none';
        this.selectedIndex = -1;
    }

    handleKeydown(e) {
        if (this.suggestionsContainer.style.display === 'none') {
            return;
        }

        const items = this.suggestionsList.querySelectorAll('.tag-suggestion-item');

        switch (e.key) {
            case 'ArrowDown':
                e.preventDefault();
                this.selectedIndex = Math.min(this.selectedIndex + 1, items.length - 1);
                this.updateSelectedSuggestion(items);
                break;

            case 'ArrowUp':
                e.preventDefault();
                this.selectedIndex = Math.max(this.selectedIndex - 1, -1);
                this.updateSelectedSuggestion(items);
                break;

            case 'Enter':
                e.preventDefault();
                if (this.selectedIndex >= 0 && items[this.selectedIndex]) {
                    const item = items[this.selectedIndex];
                    const tag = item.dataset.tag;
                    const isNew = item.dataset.isNew === 'true';
                    this.addTag(tag, isNew);
                } else if (items.length > 0) {
                    // Select first item if no selection
                    const firstItem = items[0];
                    const tag = firstItem.dataset.tag;
                    const isNew = firstItem.dataset.isNew === 'true';
                    this.addTag(tag, isNew);
                }
                break;

            case 'Escape':
                e.preventDefault();
                this.hideSuggestions();
                this.input.value = '';
                break;

            case ',':
            case 'Tab':
                // Allow comma or tab to add tag quickly
                if (this.input.value.trim().length >= 2) {
                    e.preventDefault();
                    const tag = this.input.value.trim();
                    this.addTag(tag, true);
                }
                break;
        }
    }

    updateSelectedSuggestion(items) {
        items.forEach((item, index) => {
            if (index === this.selectedIndex) {
                item.classList.add('selected');
                item.scrollIntoView({ block: 'nearest' });
            } else {
                item.classList.remove('selected');
            }
        });
    }

    handleFocus() {
        if (this.input.value.trim().length >= 2) {
            this.fetchSuggestions(this.input.value.trim());
        }
    }

    addTag(tag, isNew = false) {
        // Normalize tag
        const normalizedTag = tag.toLowerCase().replace(/\s+/g, '-');

        // Avoid duplicates
        if (this.selectedTags.includes(normalizedTag)) {
            this.input.value = '';
            this.hideSuggestions();
            return;
        }

        // Add to selected tags
        this.selectedTags.push(normalizedTag);

        // Update UI
        this.updateSelectedTagsUI();
        this.input.value = '';
        this.hideSuggestions();

        // Trigger change callback
        if (this.onChangeCallback) {
            this.onChangeCallback(this.selectedTags);
        }

        // Show brief success feedback
        if (isNew) {
            this.showBriefMessage(`✨ Created new tag: ${tag}`);
        }
    }

    removeTag(tag) {
        this.selectedTags = this.selectedTags.filter(t => t !== tag);
        this.updateSelectedTagsUI();

        // Trigger change callback
        if (this.onChangeCallback) {
            this.onChangeCallback(this.selectedTags);
        }
    }

    updateSelectedTagsUI() {
        const container = this.element.querySelector('.tag-picker-selected');
        container.innerHTML = this.renderSelectedTags();

        // Update hidden input
        this.hiddenInput.value = JSON.stringify(this.selectedTags);
    }

    showBriefMessage(message) {
        const messageDiv = document.createElement('div');
        messageDiv.className = 'tag-picker-message';
        messageDiv.textContent = message;
        this.element.appendChild(messageDiv);

        setTimeout(() => {
            messageDiv.remove();
        }, 2000);
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // Public API
    getTags() {
        return [...this.selectedTags];
    }

    setTags(tags) {
        this.selectedTags = [...tags];
        this.updateSelectedTagsUI();
    }

    onChange(callback) {
        this.onChangeCallback = callback;
    }

    clear() {
        this.selectedTags = [];
        this.updateSelectedTagsUI();
        this.input.value = '';
        this.hideSuggestions();
    }
}

// Auto-initialize all tag pickers on page load
document.addEventListener('DOMContentLoaded', () => {
    const pickers = document.querySelectorAll('[data-tag-picker]');
    pickers.forEach(element => {
        element.tagPicker = new TagPicker(element);
    });
});

// Export for use in modules
if (typeof module !== 'undefined' && module.exports) {
    module.exports = TagPicker;
}
