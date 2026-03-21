# Strategic Feedback — Session 1

## Direction: Decouple Satellite Apps

The idea of moving `3d-gallery` and `course-viewer` into their own standalone products
(loading data from this platform) makes strong architectural sense.

### The Pattern

- **This platform** = the backend (store, transcode, stream, organize, access-control)
- **Satellite apps** = the frontends (consume content via API/WebDAV, own their UX)

This is essentially a **headless CMS / content backend** model — well-established,
and a natural fit given the primitives already in place:

- WebDAV — satellite apps mount workspaces like a drive
- API keys — satellite apps authenticate programmatically
- Media serving routes — clean URLs any external app can embed
- Access codes — share content without full user accounts
- MCP — AI-agent access follows the same pattern

### Why It Strengthens the USP

Right now the 3D gallery and course viewer blur the line: is this a media server,
a course platform, or a 3D experience? Pulling them out answers that definitively.

What remains is a sharp, coherent core:

- **Workspaces** with typed folders
- **Media** managed, transcoded, streamed
- **Apps** assigned to folder types — but apps are *consumers*, not part of the platform
- **Access** via WebDAV, API, MCP — open to anything

The platform becomes the **substrate** that other things are built on top of,
rather than trying to be everything itself.

### Sharpened USP

> **"A self-hosted content workspace where you bring your own apps."**

This is stronger and more defensible than any individual feature. The folder-type +
app registry system also becomes more meaningful: app links can point to external URLs —
your course platform, your 3D gallery, a third-party tool. The registry becomes a real
integration layer, not just internal routing.

### One Thing to Design For

The API surface becomes a **contract**. Media serving routes, API key auth, and WebDAV
are currently built for internal use. If satellite apps depend on them, treat them as
stable public APIs — breaking changes need more care. Not a blocker, just worth being
deliberate about.

### Action Items

- [ ] Move `crates/standalone/3d-gallery` → own repository / product
- [ ] Move `crates/standalone/course-viewer` → own repository / product
- [ ] Define stable public API surface (media routes, WebDAV, API keys)
- [ ] Update `bpmn-simulator` app link pattern as a reference for external app URLs
- [ ] Document the integration model (how satellite apps connect to this platform)
