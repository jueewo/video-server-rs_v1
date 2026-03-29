# Agentic BPMN Extension

Custom BPMN 2.0 extension for modeling human-agentic collaborative workflows.
Based on the research paper *"Towards Modeling Human-Agentic Collaborative Workflows: A BPMN Extension"* (Ait, Cánovas Izquierdo, Cabot — University of Luxembourg / UOC, 2024).

Paper reference: `agentic-bpmn-2412.05958.pdf`
Open-source reference implementation: https://github.com/BESSER-PEARL/agentic-bpmn

## Paper Summary

The paper identifies five gaps in standard BPMN for modeling agentic workflows:

| Concept | Problem | Proposed Extension |
|---|---|---|
| **Agent profiling** | Lanes/pools can't express role or reliability | `AgenticLane` with role (manager/worker) and trust score |
| **Reflection** | No way to model self-check loops formally | `AgenticTask` with reflection mode (self/cross/human) |
| **Confidence** | Non-deterministic LLM output has no quality signal | Trust score on tasks, lanes, gateways |
| **Collaboration** | Groups/annotations are informal | `AgenticGateway` with collaboration strategy (voting/role/debate/competition) |
| **Merging** | Complex gateway with text annotation is ambiguous | Merging strategy on converging gateways and message flows |

## Our Implementation

We use the BPMN 2.0 moddle `extends` pattern (same approach as `bioc` for colors) to add custom attributes directly on standard BPMN elements. No new element types — just attributes in the `agentic` namespace.

### Namespace

```
URI:    http://agentic-bpmn/schema/1.0
Prefix: agentic
```

### Moddle Descriptor (`scripts/bpmn-bundle/modules/agentic.json`)

```json
{
  "name": "Agentic",
  "prefix": "agentic",
  "uri": "http://agentic-bpmn/schema/1.0",
  "xml": { "tagAlias": "lowerCase" },
  "types": [
    {
      "name": "TaskProperties",
      "extends": ["bpmn:Task"],
      "properties": [
        { "name": "taskType",       "isAttr": true, "type": "String"  },
        { "name": "agentId",        "isAttr": true, "type": "String"  },
        { "name": "model",          "isAttr": true, "type": "String"  },
        { "name": "maxIterations",  "isAttr": true, "type": "Integer" },
        { "name": "reflectionMode", "isAttr": true, "type": "String"  },
        { "name": "confidence",     "isAttr": true, "type": "Integer" }
      ]
    },
    {
      "name": "LaneProperties",
      "extends": ["bpmn:Lane"],
      "properties": [
        { "name": "agentRole",  "isAttr": true, "type": "String"  },
        { "name": "trustScore", "isAttr": true, "type": "Integer" }
      ]
    },
    {
      "name": "GatewayProperties",
      "extends": ["bpmn:Gateway"],
      "properties": [
        { "name": "collaborationMode", "isAttr": true, "type": "String" },
        { "name": "mergingStrategy",   "isAttr": true, "type": "String" }
      ]
    }
  ],
  "enumerations": [],
  "associations": []
}
```

### Resulting XML

```xml
<!-- Agent task with self-reflection -->
<bpmn:task id="Task_1" name="Review Code"
           agentic:taskType="agent"
           agentic:agentId="code-reviewer"
           agentic:model="claude-sonnet"
           agentic:reflectionMode="self"
           agentic:confidence="80"
           agentic:maxIterations="5" />

<!-- Agent lane with role and trust -->
<bpmn:lane id="Lane_1" name="Reviewer"
           agentic:agentRole="manager"
           agentic:trustScore="90" />

<!-- Collaboration gateway -->
<bpmn:parallelGateway id="Gw_1"
           agentic:collaborationMode="role" />

<!-- Merging gateway -->
<bpmn:parallelGateway id="Gw_2"
           agentic:mergingStrategy="leader-driven" />
```

### Attribute Reference

#### Task attributes (`agentic:` on `bpmn:Task`)

| Attribute | Type | Values | Description |
|---|---|---|---|
| `taskType` | String | `agent` | Marks task as agentic (triggers robot icon rendering) |
| `agentId` | String | any | References an agent definition (from agent collection) |
| `model` | String | any | LLM model override for this task |
| `maxIterations` | Integer | 1-100 | Max reflection/retry loops |
| `reflectionMode` | String | `self`, `cross`, `human` | How the agent validates its output |
| `confidence` | Integer | 0-100 | Minimum confidence threshold to accept output |

#### Lane attributes (`agentic:` on `bpmn:Lane`)

| Attribute | Type | Values | Description |
|---|---|---|---|
| `agentRole` | String | `manager`, `worker` | Role in collaboration (manager decides) |
| `trustScore` | Integer | 0-100 | Overall reliability of this agent |

#### Gateway attributes (`agentic:` on `bpmn:Gateway`)

| Attribute | Type | Values | Description |
|---|---|---|---|
| `collaborationMode` | String | `voting`, `role`, `debate`, `competition` | How agents collaborate (diverging gateway) |
| `mergingStrategy` | String | `majority`, `leader-driven`, `composed`, `fastest`, `most-complete` | How results are merged (converging gateway) |

## Visual Rendering

### Current (implemented)

- **Agent task**: Purple robot icon in top-left corner of task shape
- **Detection**: `agentic:taskType === "agent"` attribute, fallback to `[agent]` in name
- **Palette**: Custom "Agent Task" entry with task rectangle + robot icon

### Planned

- **Reflection badge**: Small letter marker at bottom of agent task (`S`/`C`/`H`)
- **Agent lane marker**: Robot icon below lane name
- **Agentic gateway marker**: Robot icon at top-left of gateway shape
- **Trust score display**: Small badge on lanes and tasks showing score

## Properties Panel (how to edit agentic attributes)

bpmn-js provides `bpmn-js-properties-panel` for editing element properties. We need to add a custom properties panel that shows agentic attributes when an agent task (or agentic lane/gateway) is selected.

### Implementation approach

1. **Install**: Add `@bpmn-io/properties-panel` and `bpmn-js-properties-panel` to the bundle
2. **Custom provider**: Create `AgentPropertiesProvider` that adds entries for agentic attributes
3. **Panel container**: Add a side panel div in `view.html` (only visible in edit mode)
4. **Entries**: Dropdowns for `reflectionMode`, `agentRole`, `collaborationMode`; number inputs for `confidence`, `trustScore`, `maxIterations`; text input for `agentId`, `model`

### Package additions (`scripts/bpmn-bundle/package.json`)

```json
{
  "dependencies": {
    "bpmn-js": "18.13.2",
    "bpmn-js-token-simulation": "^0.39.0",
    "@bpmn-io/properties-panel": "^3.0.0",
    "bpmn-js-properties-panel": "^5.0.0"
  }
}
```

### Module structure

```
scripts/bpmn-bundle/modules/
  agentic.json                          # moddle descriptor (extended)
  agent-renderer/                       # visual rendering (existing)
  agent-palette/                        # palette entry (existing)
  agent-simulation/                     # token sim behavior (existing)
  agent-properties/                     # NEW: properties panel
    index.js                            # module definition
    AgentPropertiesProvider.js           # custom entries for agentic attrs
```

### Entry point change (`entry.js`)

```js
import PropertiesPanelModule from 'bpmn-js-properties-panel';
import AgentPropertiesModule from './modules/agent-properties/index.js';

// Add to additionalModules:
// PropertiesPanelModule, AgentPropertiesModule
```

### Template change (`view.html`)

```html
<!-- Add next to #bpmn-canvas in edit mode -->
<div id="properties-panel" class="hidden"></div>
```

```js
// In enterEditMode():
const panel = document.getElementById('properties-panel');
panel.classList.remove('hidden');

// Constructor option:
new BpmnJS({
  container: '#bpmn-canvas',
  propertiesPanel: { parent: '#properties-panel' }
});
```

## Mapping to Paper Concepts

| Paper Element | Our Implementation | Status |
|---|---|---|
| AgenticLane (profile) | `agentic:agentRole` + `agentic:trustScore` on `bpmn:Lane` | Moddle ready, no UI yet |
| AgenticTask (reflection) | `agentic:reflectionMode` + `agentic:confidence` on `bpmn:Task` | Moddle ready, no UI yet |
| AgenticTask (basic) | `agentic:taskType="agent"` + robot icon | Done |
| AgenticGateway (collab) | `agentic:collaborationMode` on `bpmn:Gateway` | Moddle ready, no UI yet |
| AgenticGateway (merge) | `agentic:mergingStrategy` on `bpmn:Gateway` | Moddle ready, no UI yet |
| Agentic message flow | Not yet planned | Future |
| Uncertainty propagation | Not yet planned (paper lists as future work too) | Future |
| Notation markers (S/C/H badges) | Not yet implemented | Planned |

## Runtime Mapping

The agentic attributes map to the process runtime engine:

| BPMN Attribute | Runtime Behavior |
|---|---|
| `taskType="agent"` | Execute task via agent framework instead of human assignment |
| `agentId` | Look up agent definition from agent collection |
| `model` | Override LLM model for this specific task |
| `reflectionMode="self"` | Agent re-evaluates own output before proceeding |
| `reflectionMode="cross"` | Route output to another agent for review |
| `reflectionMode="human"` | Pause execution, wait for human approval |
| `confidence` | Minimum threshold — keep reflecting until met or maxIterations reached |
| `maxIterations` | Hard cap on reflection loops |
| `agentRole="manager"` | This agent makes final decisions in role-based collaboration |
| `collaborationMode` | How to dispatch work at a diverging gateway |
| `mergingStrategy` | How to select/combine results at a converging gateway |

## Files

| File | Purpose |
|---|---|
| `scripts/bpmn-bundle/modules/agentic.json` | Moddle descriptor |
| `scripts/bpmn-bundle/modules/agent-renderer/` | Custom visual rendering |
| `scripts/bpmn-bundle/modules/agent-palette/` | Palette entry |
| `scripts/bpmn-bundle/modules/agent-simulation/` | Token simulation behavior |
| `scripts/bpmn-bundle/entry.js` | Bundle entry point |
| `scripts/build-bpmn-bundle.sh` | Build script |
| `crates/bpmn-viewer/templates/bpmn/view.html` | Viewer/editor template |
| `crates/bpmn-viewer/src/lib.rs` | Rust folder renderer |
