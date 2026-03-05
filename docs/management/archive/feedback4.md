# Strategic Feedback — Session 4

## Direction: Communicating the USP

### The Message

The internal anchor — "a self-hosted content workspace where folders have apps" —
is right but too abstract for external use. Tailor by audience:

**For developers:**
> "A self-hosted Rust platform that stores, streams, and organizes your content —
> and lets any app connect to it."

**For teams / SMBs:**
> "Your files, your server, your apps. Stop paying five SaaS subscriptions for
> things you could own."

**For technical decision-makers:**
> "A content backend with WebDAV, API, and MCP access built in. Bring your own frontend."

### The Demo Moment

Show the thing nothing else does:
> Create a folder → assign it a type → an app opens it.

That's the moment. Everything else is table stakes.

---

## What Must Be Done Before Communicating

Communicating prematurely is worse than not communicating. The gap between
the story and the current reality would be visible to any serious evaluator.

### Must Be Done First

1. **Phase 1 of the roadmap — storage consolidation**
   You cannot say "one place files live" while the vault/workspace dual model exists.
   The story and the product must match.

2. **Satellite apps extracted**
   The 3D gallery and course viewer in the same repo contradict "bring your own apps."
   Extract them first, then point to them as proof of the model.

3. **Setup must be simple**
   A Docker Compose with a single command. The current dependency list (ffmpeg,
   mediamtx, gs, cwebp, Casdoor) is a credibility killer for first impressions.

4. **User documentation + demo video**
   A clear README and a 2-minute demo video. Ironic for a media platform not to have one.

### Should Be Done Before Broader Communication

5. **Stable public API surface documented**
   If the pitch includes "external tools can connect," those interfaces need to be
   described and stable. Undocumented APIs are not a feature.

6. **At least one real satellite app**
   The "bring your own apps" claim needs a working example. The extracted course viewer
   or 3D gallery as a separate repo connecting back to this platform proves the model.

### Can Come After

- App ecosystem / plugin registry
- Polish on individual features
- Broader marketing

---

## Realistic Timeline

Probably 3–4 months of focused work before the story is honest enough to tell publicly:

| Milestone | Phase |
|---|---|
| Storage consolidation complete | Roadmap Phase 1 |
| Transcoding as a service | Roadmap Phase 2 |
| Stable open access layer | Roadmap Phase 3 |
| Satellite apps extracted + one working demo | Roadmap Phase 4 |
| Docker Compose setup + README + demo video | Pre-launch |
| **Communicate** | ✓ |
