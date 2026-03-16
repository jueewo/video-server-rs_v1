# 02 - Usability Roast

## Who Can Actually Use This?

Let's be honest: right now, **you** can use this. Maybe 2-3 other developers who've spent a week reading the code. That's it.

The platform has no onboarding flow, no in-app help, no tooltips, no empty states that guide users, and no self-service setup wizard. The mental model ("workspace > folder > app") is elegant in a strategy doc but invisible in the actual UI.

---

## Onboarding Experience

### First Boot
A new user who runs `docker compose up` will see:
1. A login page (good)
2. Emergency login option if enabled (useful for dev)
3. After login: ...what? Where do they go? What do they do first?

**Missing:**
- Welcome wizard or setup flow
- "Create your first workspace" prompt
- Sample content or demo workspace
- Quick-start checklist

### The "Demo Moment" Problem
Your strategy doc says the demo moment is: *"Create a folder > assign it a type > an app opens it."*

But a new user has to:
1. Know that workspaces exist (not obvious from the UI)
2. Create a workspace (where's the button?)
3. Navigate into it (how does the file browser work?)
4. Create a folder (is this a filesystem folder? a virtual folder?)
5. Know that folder types exist (this isn't a standard concept anywhere)
6. Assign a type (from a dropdown? a right-click menu?)
7. Understand that assigning "media-server" auto-creates a vault (invisible magic)

That's 7 conceptual steps before the "aha" moment. Most users give up after 2.

---

## Daily Workflow Friction

### Media Upload
The upload flow is actually decent — drop zone, file preview, progress bar with WebSocket updates, media type auto-detection. This is one of the better-polished parts of the UI.

**But:**
- The form has too many fields for a simple upload (vault picker, group assignment, category, tags, transcode toggle, keep_original toggle). Most users want: choose file, give it a title, upload.
- The "slug" field is exposed to users. Slugs are an implementation detail — auto-generate and hide.
- `transcode_for_streaming` is developer jargon. Call it "Optimize for web playback" or just do it automatically.
- `keep_original` is a storage concern. Default to keeping it and let advanced users change it in settings.

### Media Browsing
The gallery view (`media_list_tailwind.html`) is functional:
- Grid layout, responsive
- Type filters (All, Videos, Images, Documents)
- Search, vault/tag/group selectors
- Pagination

**But:**
- No drag-and-drop reordering
- No bulk operations (select multiple, delete, move, tag)
- No folder/album concept within the gallery (everything is a flat list filtered by vault)
- Sort options are limited (created_at, title, file_size — no "most viewed" or "recently modified")

### Workspace Browser
This is the core navigation but it tries to be too many things:
- File browser (like Finder/Explorer)
- Folder type manager (unique concept)
- App launcher (when clicking typed folders)
- Access code manager (sharing panel)

Each of these is a different UX pattern. Cramming them into one view creates cognitive overload.

### Access Code Sharing
The sharing model is powerful (codes work without accounts, cover multiple folders, have expiry). But the UX for creating and managing codes is buried in API endpoints and management panels rather than being a one-click "Share" button on any resource.

**Ideal flow:** Click share on a folder > get a link > done.
**Current flow:** Navigate to access code management > create code > assign folders to code > copy link > share.

---

## Template UI Quality

### What's Working
- TailwindCSS + DaisyUI provides a consistent visual baseline
- Responsive grid layouts
- Dark/light theme support
- Lucide icons for visual consistency
- HTMX for snappy interactions without full-page reloads

### What's Not
- **No design system.** Templates across 15+ crates each do their own thing. Button styles, spacing, card layouts vary between views.
- **331 HTML templates** with no shared component library beyond `base-tailwind.html` and a few partials. That's template sprawl.
- **No loading states.** When you click something, does it freeze or is it working? No spinners, no skeleton screens, no optimistic UI.
- **No keyboard shortcuts.** Power users expect Ctrl+K for search, keyboard navigation in lists, etc.
- **Mobile experience is an afterthought.** Responsive grids are the bare minimum. The workspace browser, BPMN editor, 3D gallery — these need dedicated mobile UX or an honest "desktop only" disclaimer.

---

## The Persona Gap

### Persona 1 (Juergen - the consultant)
This persona is you. The platform works for you because you built it and understand every concept. Fair enough.

### Persona 2 (Maria - SMB owner, "not a developer")
Maria will bounce within 5 minutes. She doesn't know what:
- A "vault" is (storage isolation concept)
- A "slug" is (URL identifier)
- "HLS transcoding" means (video processing)
- A "folder type" is (app binding)
- An "access code" vs "access group" is (sharing mechanisms)

The UI exposes all of these developer concepts directly. For Maria, the platform needs to hide 80% of this behind sensible defaults and surface only: Upload, Organize, Share.

### Persona 3 (Dr. Stefan - regulated industry)
Stefan needs compliance features that don't exist yet:
- Audit log UI (the backend logs access decisions but there's no UI to view them)
- Data retention policies
- Export/deletion for GDPR compliance
- User activity reports
- Role-based access control UI (not just API)

---

## Specific UX Problems

| Area | Problem | Impact |
|------|---------|--------|
| Navigation | No breadcrumbs in most views | Users get lost |
| Search | No global search (each section has its own) | Users can't find content |
| Error messages | Technical error strings shown to users | Confusion, support burden |
| Empty states | Generic "no items" messages | No guidance on what to do |
| Forms | Too many fields, no progressive disclosure | Intimidating for new users |
| Settings | No settings page | Users can't customize anything |
| Help | No in-app help, tooltips, or documentation links | Users rely on external docs |
| Feedback | No toast notifications for async operations | Users don't know if actions succeeded |
| Accessibility | No ARIA labels, skip links, focus management | Excludes users with disabilities |

---

## What Would Make It Usable

### For developers/consultants (your actual current audience)
1. Better documentation in the UI itself (inline help, field descriptions)
2. Global search (Ctrl+K omnibar)
3. Bulk operations in the media gallery
4. One-click sharing from any resource
5. Activity feed (what happened recently?)

### For non-technical users (your aspirational audience)
1. Setup wizard on first login
2. Hide all technical concepts (vaults, slugs, HLS, MIME types)
3. Simplify upload to: file + title
4. Add guided workflows ("Upload your first video", "Share with a client")
5. Redesign workspace browser for clarity over power

### The Hard Truth
You need to pick one. Building for developers AND non-technical SMB owners simultaneously is how you end up with a UI that satisfies neither. My recommendation: own the developer/consultant audience first, nail that UX, then add a "simple mode" later.
