# Workspace Processors

Specialized processors for different types of workspace folders. Each processor handles a specific folder type defined in `workspace.yaml`.

## Architecture

```
crates/workspace-processors/
├── static-site/          → Build and deploy static websites
├── bpmn-simulator/       → Execute BPMN process diagrams
└── agent-collection/     → Manage AI agent definitions
```

## Folder Types

### 1. Static Site (`static-site`)

**Purpose:** Build static websites from workspace files and publish to vault.

**Example workspace.yaml:**
```yaml
folders:
  "website-project":
    type: static-site
    entry_point: index.html
    build:
      framework: hugo
      theme: minimal
      output_dir: _site
    deploy:
      target: /media/{slug}
```

**Supported frameworks:**
- `plain-html` — Bundle HTML/CSS/JS files
- `hugo` — Hugo static site generator
- `jekyll` — Jekyll static site generator

**Future:** 11ty, Astro, Next.js static export

---

### 2. BPMN Simulator (`bpmn-simulator`)

**Purpose:** Execute BPMN process diagrams with test data and generate execution traces.

**Example workspace.yaml:**
```yaml
folders:
  "processes/order-flow":
    type: bpmn-simulator
    main_process: order-flow.bpmn
    config: sim-config.json
    variables:
      default_timeout: 3600
      max_retries: 3
```

**Features:**
- Parse BPMN 2.0 XML
- Execute process flows
- Track state and variables
- Generate execution reports
- Identify bottlenecks

**Future:** Performance analysis, cost estimation, SLA validation

---

### 3. Agent Collection (`agent-collection`)

**Purpose:** Manage AI agents defined in markdown files with YAML frontmatter.

**Example workspace.yaml:**
```yaml
folders:
  "agents":
    type: agent-collection
    agents:
      - file: coder.md
        role: code-generation
        model: claude-sonnet-4.5
      - file: reviewer.md
        role: code-review
        model: claude-opus-4.6
    shared_context: context/project-docs.md
    memory_dir: .memory/
```

**Agent file format (markdown):**
```markdown
---
role: code-generation
model: claude-sonnet-4.5
tools: [read, write, bash]
temperature: 0.7
---

# Code Generation Agent

You are an expert software engineer specializing in Rust...
```

**Features:**
- Parse agent definitions
- Load into Claude Code
- Export for API integration
- Shared context and memory
- Agent workflows

**Future:** Agent composition, team workflows, memory persistence

---

## Implementation Status

| Processor | Status | Priority |
|-----------|--------|----------|
| static-site | 🔲 Placeholder | High |
| bpmn-simulator | 🔲 Placeholder | Medium |
| agent-collection | 🔲 Placeholder | High |

## Adding a New Processor

1. **Create crate:** `crates/workspace-processors/{processor-name}/`
2. **Define config struct:** Parse metadata from `workspace.yaml`
3. **Implement processor:** Main processing logic
4. **Add to workspace:** Add to `Cargo.toml` members
5. **Integrate UI:** Add buttons/actions in workspace browser
6. **Add to FolderType enum:** In `workspace_config.rs`

## Integration Points

### workspace-manager

Processors are invoked by workspace-manager when:
- User clicks "Build Site" button on static-site folder
- User clicks "Run Simulation" on bpmn-simulator folder
- User clicks "Load Agents" on agent-collection folder

### UI Flow

```
User clicks processor button
    ↓
POST /api/workspaces/{id}/process/{folder_path}
    ↓
workspace-manager loads workspace.yaml
    ↓
Determines folder type
    ↓
Calls appropriate processor
    ↓
Returns result (built site, execution trace, agent configs)
    ↓
UI displays result or publishes to vault
```

## Development Roadmap

### Phase 1: Static Site Processor
- [ ] Plain HTML bundling
- [ ] Hugo integration
- [ ] Publish to vault as single-page media item

### Phase 2: Agent Collection Processor
- [ ] Parse markdown + YAML frontmatter
- [ ] Load agents into Claude Code
- [ ] Export for API integration

### Phase 3: BPMN Simulator
- [ ] Parse BPMN XML
- [ ] Execute simple flows
- [ ] Generate execution traces

### Phase 4: Advanced Features
- [ ] Agent workflows (multi-agent collaboration)
- [ ] BPMN performance analysis
- [ ] Static site preview server
