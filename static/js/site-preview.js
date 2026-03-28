/**
 * site-preview.js
 *
 * Client-side content preview for the site entry editor.
 * Renders elements_above, MDX body (markdown), elements_below, and FAQ
 * into a scrollable preview panel.
 */

/* global marked */

// Configure marked for safe rendering
if (typeof marked !== 'undefined') {
    marked.setOptions({ breaks: true, gfm: true });
}

// Markdown preview CSS (injected once since prose class is not available in admin)
(function() {
    if (document.getElementById('_preview-md-style')) return;
    var style = document.createElement('style');
    style.id = '_preview-md-style';
    style.textContent = [
        '.md-preview { line-height: 1.7; color: inherit; }',
        '.md-preview h1 { font-size: 1.75em; font-weight: 700; margin: 1.2em 0 0.5em; border-bottom: 1px solid oklch(0.7 0 0 / 0.2); padding-bottom: 0.3em; }',
        '.md-preview h2 { font-size: 1.4em; font-weight: 600; margin: 1em 0 0.4em; }',
        '.md-preview h3 { font-size: 1.15em; font-weight: 600; margin: 0.8em 0 0.3em; }',
        '.md-preview h4, .md-preview h5, .md-preview h6 { font-size: 1em; font-weight: 600; margin: 0.6em 0 0.2em; }',
        '.md-preview p { margin: 0.6em 0; }',
        '.md-preview ul, .md-preview ol { margin: 0.5em 0; padding-left: 1.5em; }',
        '.md-preview ul { list-style: disc; }',
        '.md-preview ol { list-style: decimal; }',
        '.md-preview li { margin: 0.2em 0; }',
        '.md-preview blockquote { border-left: 3px solid oklch(0.6 0.15 250); padding: 0.3em 1em; margin: 0.6em 0; opacity: 0.85; }',
        '.md-preview pre { background: oklch(0.2 0 0 / 0.08); padding: 0.8em 1em; border-radius: 0.375rem; overflow-x: auto; font-size: 0.85em; margin: 0.6em 0; }',
        '.md-preview code { font-family: ui-monospace, monospace; font-size: 0.9em; }',
        '.md-preview :not(pre) > code { background: oklch(0.5 0 0 / 0.1); padding: 0.15em 0.35em; border-radius: 0.25rem; }',
        '.md-preview a { color: oklch(0.55 0.2 250); text-decoration: underline; }',
        '.md-preview img { max-width: 100%; border-radius: 0.375rem; margin: 0.5em 0; }',
        '.md-preview hr { border: none; border-top: 1px solid oklch(0.7 0 0 / 0.2); margin: 1.2em 0; }',
        '.md-preview strong { font-weight: 600; }',
        '.md-preview table { border-collapse: collapse; width: 100%; margin: 0.6em 0; }',
        '.md-preview th, .md-preview td { border: 1px solid oklch(0.7 0 0 / 0.2); padding: 0.4em 0.6em; text-align: left; }',
        '.md-preview th { font-weight: 600; background: oklch(0.5 0 0 / 0.05); }',
    ].join('\n');
    document.head.appendChild(style);
})();

function renderPreview(fmData, bodyText) {
    var parts = [];

    // Page title + meta
    if (fmData.title) {
        parts.push('<div class="mb-6 pb-4 border-b border-base-300">');
        parts.push('<h1 class="text-2xl font-bold">' + _prevEsc(fmData.title) + '</h1>');
        if (fmData.desc || fmData.description) {
            parts.push('<p class="text-base-content/60 mt-1">' + _prevEsc(fmData.desc || fmData.description) + '</p>');
        }
        var meta = [];
        if (fmData.pubDate) meta.push(_prevEsc(fmData.pubDate));
        if (fmData.author) meta.push(_prevEsc(fmData.author));
        if (meta.length) parts.push('<p class="text-xs text-base-content/40 mt-2">' + meta.join(' · ') + '</p>');
        if (fmData.draft) parts.push('<span class="badge badge-warning badge-sm mt-2">Draft</span>');
        parts.push('</div>');
    }

    // Elements above (Pre)
    var above = fmData.elements_above || [];
    if (above.length) {
        parts.push('<div class="space-y-4 mb-6">');
        above.forEach(function(el) { parts.push(renderElement(el)); });
        parts.push('</div>');
        parts.push('<hr class="border-base-300 my-4">');
    }

    // MDX body → markdown
    if (bodyText && bodyText.trim()) {
        parts.push('<div class="md-preview text-sm">');
        if (typeof marked !== 'undefined') {
            // Strip JSX expressions like {frontmatter.title} and import statements
            var cleaned = bodyText
                .replace(/^import\s+.*$/gm, '')
                .replace(/\{frontmatter\.\w+\}/g, '')
                .replace(/\{\/\*.*?\*\/\}/gs, '');
            parts.push(marked.parse(cleaned));
        } else {
            parts.push('<pre class="whitespace-pre-wrap text-sm">' + _prevEsc(bodyText) + '</pre>');
        }
        parts.push('</div>');
    }

    // Elements below (Post)
    var below = fmData.elements_below || [];
    if (below.length) {
        parts.push('<hr class="border-base-300 my-4">');
        parts.push('<div class="space-y-4 mt-6">');
        below.forEach(function(el) { parts.push(renderElement(el)); });
        parts.push('</div>');
    }

    // FAQ
    var faq = fmData.faqdata || [];
    if (faq.length) {
        parts.push('<hr class="border-base-300 my-4">');
        parts.push('<div class="mt-6">');
        parts.push('<h2 class="text-lg font-bold mb-3">FAQ</h2>');
        parts.push('<div class="space-y-2">');
        faq.forEach(function(item) {
            parts.push('<details class="collapse collapse-arrow bg-base-200/50 border border-base-300 rounded-lg">');
            parts.push('<summary class="collapse-title text-sm font-medium py-2 min-h-0">' + _prevEsc(item.quest || '') + '</summary>');
            parts.push('<div class="collapse-content text-sm text-base-content/70"><p>' + _prevEsc(item.ans || '') + '</p></div>');
            parts.push('</details>');
        });
        parts.push('</div></div>');
    }

    if (!parts.length) {
        return '<p class="text-base-content/40 text-center py-8">No content to preview.</p>';
    }
    return parts.join('\n');
}

function renderElement(el) {
    if (el.draft) return '';

    var type = el.element || 'Unknown';
    // Flatten legacy content/props nesting
    var p = Object.assign({}, el.content || {}, el.props || {}, el);

    switch (type) {
        case 'Hero':
        case 'Hero2':
            return renderHero(p, type);
        case 'TitleHero':
            return renderTitleHero(p);
        case 'TitleAlertBanner':
            return renderTitleAlert(p);
        case 'Collection':
            return renderCollection(p);
        case 'MdText':
            return renderMdText(p);
        case 'NewsBanner':
            return renderNewsBanner(p);
        case 'Video':
            return renderVideo(p);
        case 'Presentation':
        case 'Process':
            return renderDataFile(p, type);
        case 'TeamGrid':
            return renderPlaceholder(type, p.filtertype ? 'Filter: ' + p.filtertype : '');
        case 'Section':
            return renderSection(p);
        case 'Carousel':
            return renderCarousel(p);
        case 'StatData':
            return renderStatData(p);
        case 'CTARemote':
            return renderCTARemote(p);
        case 'FAQ':
            return renderPlaceholder('FAQ', '');
        default:
            return renderPlaceholder(type, getElementLabel(el));
    }
}

// ── Element renderers ────────────────────────────────────────────────────────

function renderHero(p, type) {
    var html = '<div class="bg-base-200/30 border border-base-300 rounded-lg p-5">';
    html += '<div class="flex items-start gap-4">';
    html += '<div class="flex-1">';
    if (p.title) html += '<h2 class="text-xl font-bold">' + _prevEsc(p.title) + '</h2>';
    html += descParagraphs(p.desc);
    if (p.button) html += '<span class="btn btn-sm btn-primary mt-3">' + _prevEsc(p.button) + '</span>';
    html += '</div>';
    if (p.image) html += '<div class="w-24 h-24 bg-base-300 rounded flex items-center justify-center text-xs text-base-content/40 shrink-0 overflow-hidden">' + imgOrPlaceholder(p.image) + '</div>';
    html += '</div>';
    html += typeBadge(type);
    html += '</div>';
    return html;
}

function renderTitleHero(p) {
    var tag = p.h1 ? 'h1' : 'h2';
    var html = '<div class="bg-base-200/30 border border-base-300 rounded-lg p-5">';
    html += '<div class="flex items-start gap-4">';
    html += '<div class="flex-1">';
    if (p.title) html += '<' + tag + ' class="text-xl font-bold">' + _prevEsc(p.title) + '</' + tag + '>';
    html += descParagraphs(p.desc);
    html += descParagraphs(p.desc2);
    html += '</div>';
    if (p.image) html += '<div class="w-20 h-20 bg-base-300 rounded flex items-center justify-center text-xs text-base-content/40 shrink-0 overflow-hidden">' + imgOrPlaceholder(p.image) + '</div>';
    html += '</div>';
    html += typeBadge('TitleHero');
    html += '</div>';
    return html;
}

function renderTitleAlert(p) {
    var html = '<div class="bg-warning/10 border border-warning/30 rounded-lg p-4">';
    if (p.title) html += '<h3 class="font-bold text-warning">' + _prevEsc(p.title) + '</h3>';
    html += descParagraphs(p.desc);
    html += typeBadge('TitleAlertBanner');
    html += '</div>';
    return html;
}

function renderCollection(p) {
    var html = '<div class="bg-base-200/30 border border-base-300 rounded-lg p-4">';
    if (p.title) html += '<h3 class="font-bold mb-1">' + _prevEsc(p.title) + '</h3>';
    html += '<p class="text-sm text-base-content/50">Collection: <span class="font-mono">' + _prevEsc(p.collection || '—') + '</span></p>';
    if (p.card) html += '<p class="text-xs text-base-content/40">Card style: ' + _prevEsc(p.card) + '</p>';
    html += typeBadge('Collection');
    html += '</div>';
    return html;
}

function renderMdText(p) {
    var html = '<div class="bg-base-200/30 border border-base-300 rounded-lg p-4">';
    if (p.title) html += '<h3 class="font-bold mb-1">' + _prevEsc(p.title) + '</h3>';
    html += '<p class="text-sm text-base-content/50">MDX include: <span class="font-mono">' + _prevEsc(p.mdcollslug || '—') + '</span></p>';
    html += typeBadge('MdText');
    html += '</div>';
    return html;
}

function renderNewsBanner(p) {
    var html = '<div class="bg-info/10 border border-info/30 rounded-lg p-4">';
    if (p.title) html += '<h3 class="font-bold">' + _prevEsc(p.title) + '</h3>';
    html += descParagraphs(p.desc);
    html += typeBadge('NewsBanner');
    html += '</div>';
    return html;
}

function renderVideo(p) {
    var html = '<div class="bg-base-200/30 border border-base-300 rounded-lg p-4">';
    if (p.title) html += '<h3 class="font-bold mb-1">' + _prevEsc(p.title) + '</h3>';
    html += '<div class="flex items-center gap-2 text-sm text-base-content/50">';
    html += '<svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><polygon points="5 3 19 12 5 21 5 3"/></svg>';
    html += '<span class="font-mono text-xs">' + _prevEsc(p.videoUrl || '—') + '</span>';
    html += '</div>';
    html += typeBadge('Video');
    html += '</div>';
    return html;
}

function renderDataFile(p, type) {
    var html = '<div class="bg-base-200/30 border border-base-300 rounded-lg p-4">';
    if (p.title) html += '<h3 class="font-bold mb-1">' + _prevEsc(p.title) + '</h3>';
    html += descParagraphs(p.desc);
    html += '<p class="text-xs text-base-content/40 font-mono mt-1">' + _prevEsc(p.datafile || '—') + '</p>';
    html += typeBadge(type);
    html += '</div>';
    return html;
}

function renderSection(p) {
    var html = '<div class="border-l-4 border-primary/30 pl-4 space-y-3">';
    html += '<div class="text-xs text-base-content/40 font-mono">Section' + (p.styleclass ? ' .' + _prevEsc(p.styleclass) : '') + '</div>';
    var nested = p.elements || [];
    nested.forEach(function(child) { html += renderElement(child); });
    if (!nested.length) html += '<p class="text-xs text-base-content/30 italic">Empty section</p>';
    html += '</div>';
    return html;
}

function renderCarousel(p) {
    var items = p.data || [];
    var html = '<div class="bg-base-200/30 border border-base-300 rounded-lg p-4">';
    html += '<div class="text-xs text-base-content/40 mb-2">Carousel — ' + items.length + ' slide' + (items.length !== 1 ? 's' : '') + '</div>';
    items.forEach(function(item, i) {
        html += '<div class="text-sm">' + (i + 1) + '. ' + _prevEsc(item.title || '') + '</div>';
    });
    html += typeBadge('Carousel');
    html += '</div>';
    return html;
}

function renderStatData(p) {
    var items = p.data || [];
    var html = '<div class="bg-base-200/30 border border-base-300 rounded-lg p-4">';
    html += '<div class="grid grid-cols-3 gap-3">';
    items.forEach(function(item) {
        html += '<div class="text-center">';
        html += '<div class="text-xl font-bold text-primary">' + _prevEsc(item.value || '') + '</div>';
        html += '<div class="text-xs text-base-content/60">' + _prevEsc(item.title || '') + '</div>';
        html += '</div>';
    });
    html += '</div>';
    html += typeBadge('StatData');
    html += '</div>';
    return html;
}

function renderCTARemote(p) {
    var html = '<div class="bg-base-200/30 border border-base-300 rounded-lg p-4 text-center">';
    if (p.title) html += '<h3 class="font-bold mb-2">' + _prevEsc(p.title) + '</h3>';
    html += '<span class="btn btn-sm btn-primary">' + _prevEsc(p.submitbuttontxt || 'Submit') + '</span>';
    html += typeBadge('CTARemote');
    html += '</div>';
    return html;
}

function renderPlaceholder(type, label) {
    var html = '<div class="bg-base-200/20 border border-dashed border-base-300 rounded-lg p-3 text-center">';
    html += '<span class="badge badge-outline badge-sm">' + _prevEsc(type) + '</span>';
    if (label) html += '<span class="text-xs text-base-content/40 ml-2">' + _prevEsc(label) + '</span>';
    html += '</div>';
    return html;
}

// ── Helpers ──────────────────────────────────────────────────────────────────

function _prevEsc(s) {
    return String(s || '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function descParagraphs(arr) {
    if (!arr || !Array.isArray(arr) || !arr.length) return '';
    return arr.map(function(line) {
        return '<p class="text-sm text-base-content/70 mt-1">' + _prevEsc(line) + '</p>';
    }).join('');
}

function typeBadge(type) {
    return '<div class="mt-2 text-right"><span class="badge badge-ghost badge-xs font-mono">' + _prevEsc(type) + '</span></div>';
}

function imgOrPlaceholder(path) {
    if (!path) return '—';
    // Show a small placeholder with the filename
    var name = String(path).split('/').pop();
    return '<span class="text-center text-[0.6rem] leading-tight p-1 break-all">' + _prevEsc(name) + '</span>';
}
