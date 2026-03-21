# Site Structure Visualization

The site-overview dashboard includes a visual canvas showing pages, collections, and their relationships.

## Current Implementation

**Plain SVG + vanilla JS** — zero dependencies.

- Pages rendered as blue nodes (left column)
- Collections rendered as green nodes (right column)
- Bezier curves connect pages to collections they reference
- Drag nodes to reposition
- Click a node to navigate to its editor

### Relationship detection

Edges are discovered from three sources:

| Source | Element type | Field | Target collection |
|---|---|---|---|
| Page element | `Collection` | `collection` | Named collection |
| Page element | `MdText` | `mdcollslug` | `mdcontent` collection |
| sitedef.yaml | `legal[]` | `collection` | Named collection |
| Implicit | — | page slug = collection name | Matching collection |

Nested elements inside `Section` containers are scanned recursively.

### Data flow

1. `build_structure_json()` in `crates/site-overview/src/lib.rs` scans page data files and sitedef.yaml
2. Produces a JSON graph `{ nodes: [...], edges: [...] }`
3. Embedded in the SVG element's `data-graph` attribute
4. Parsed and rendered client-side by vanilla JS in `overview.html`

## Future upgrade options

### Rete.js (v2)

- Full visual programming framework with node editor
- Supports custom nodes, connections, and context menus
- Could enable editing element pipelines visually (drag Collection element → set target collection)
- Heavyweight (~150KB gzipped)
- https://rete.js.org

**Best for:** If the canvas evolves into a visual page builder where users wire elements together.

### Drawflow

- Lightweight (~15KB) node-based flow editor
- HTML-based nodes (easy to style with DaisyUI)
- Drag, connect, delete, zoom, pan
- Simple API, easy to integrate
- https://github.com/jerosoler/Drawflow

**Best for:** A middle ground — interactive node editing without the full weight of Rete.js.

### Cytoscape.js

- Graph theory / network visualization library
- Automatic layouts (hierarchical, force-directed, breadthfirst)
- Rich styling, events, and interaction model
- ~180KB but very mature
- https://js.cytoscape.org

**Best for:** If the graph needs automatic layout, analytics, or graph-theory features.

### D3.js (force layout)

- General-purpose data visualization
- Force-directed layout for dynamic positioning
- Maximum flexibility, steep learning curve
- Often overkill for a fixed-structure site map

**Best for:** Complex, data-heavy visualizations beyond simple site structure.

## Recommendation

Start with the current SVG implementation. Upgrade to **Drawflow** if users need to:
- Add/remove connections visually
- Create new pages or collections from the canvas
- Edit element properties in popover panels

Upgrade to **Rete.js** only if the canvas becomes a full visual programming environment.
