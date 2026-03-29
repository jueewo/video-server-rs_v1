/**
 * Custom bpmn-js bundle entry point.
 *
 * Includes:
 * - bpmn-js Modeler (core editor)
 * - bpmn-js-token-simulation (visual process simulation)
 * - bpmn-js-properties-panel (element properties editing)
 * - Agent task renderer (custom icon for agentic tasks)
 * - Agent task palette entry (drag "Agent Task" from sidebar)
 * - Agent properties provider (agentic attributes in properties panel)
 * - Agent simulation behavior (auto-continue for agent tasks)
 * - Agentic moddle descriptor (BPMN 2.0 extension for agent metadata)
 *
 * Exposes globalThis.BpmnJS — drop-in replacement for the stock modeler.
 */

import BpmnModeler from 'bpmn-js/lib/Modeler';
import TokenSimulationModule from 'bpmn-js-token-simulation';
import { BpmnPropertiesPanelModule, BpmnPropertiesProviderModule } from 'bpmn-js-properties-panel';
import AgentRendererModule from './modules/agent-renderer/index.js';
import AgentPaletteModule from './modules/agent-palette/index.js';
import AgentPropertiesModule from './modules/agent-properties/index.js';
import AgentSimulationModule from './modules/agent-simulation/index.js';
import agenticDescriptor from './modules/agentic.json';

class CustomBpmnModeler extends BpmnModeler {
  constructor(options = {}) {
    const additionalModules = [
      TokenSimulationModule,
      BpmnPropertiesPanelModule,
      BpmnPropertiesProviderModule,
      AgentRendererModule,
      AgentPaletteModule,
      AgentPropertiesModule,
      AgentSimulationModule,
      ...(options.additionalModules || [])
    ];

    const moddleExtensions = {
      agentic: agenticDescriptor,
      ...(options.moddleExtensions || {})
    };

    super({
      ...options,
      additionalModules,
      moddleExtensions
    });
  }
}

globalThis.BpmnJS = CustomBpmnModeler;
