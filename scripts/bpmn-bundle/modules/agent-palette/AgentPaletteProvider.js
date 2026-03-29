/**
 * Palette provider that adds an "Agent Task" entry to the BPMN modeler palette.
 * Creates a bpmn:Task with agentic:AgenticProperties extension element.
 * Uses a custom robot SVG icon rendered via the palette `html` property.
 */

// Palette icon: task rectangle (black, like standard BPMN palette) with robot inside (also black)
const PALETTE_SVG = `<svg xmlns="http://www.w3.org/2000/svg" width="28" height="22" viewBox="0 0 28 22" fill="none">` +
  // Task rectangle
  `<rect x="1" y="1" width="26" height="20" rx="3" stroke="#333" stroke-width="1.5" fill="white"/>` +
  // Robot (centered in rect, black stroke)
  `<g transform="translate(8, 3.5)" stroke="#333" stroke-width="1" stroke-linecap="round" stroke-linejoin="round">` +
    `<line x1="6" y1="0" x2="6" y2="1.5"/>` +
    `<rect x="1.5" y="1.5" width="9" height="6.5" rx="0.8"/>` +
    `<rect x="3.5" y="3.5" width="1.5" height="1.2" rx="0.2"/>` +
    `<rect x="7" y="3.5" width="1.5" height="1.2" rx="0.2"/>` +
    `<line x1="4" y1="6.5" x2="8" y2="6.5"/>` +
    `<rect x="3" y="8.5" width="6" height="2.5" rx="0.4"/>` +
    `<line x1="0" y1="4" x2="1.5" y2="4"/>` +
    `<line x1="10.5" y1="4" x2="12" y2="4"/>` +
  `</g></svg>`;

const ENTRY_HTML = `<div class="entry" draggable="true" style="display:flex;align-items:center;justify-content:center">${PALETTE_SVG}</div>`;

export default function AgentPaletteProvider(palette, create, elementFactory, moddle) {
  this._create = create;
  this._elementFactory = elementFactory;
  this._moddle = moddle;

  palette.registerProvider(this);
}

AgentPaletteProvider.$inject = ['palette', 'create', 'elementFactory', 'moddle'];

AgentPaletteProvider.prototype.getPaletteEntries = function() {
  const create = this._create;
  const elementFactory = this._elementFactory;
  const moddle = this._moddle;

  return {
    'create.agent-task': {
      group: 'activity',
      html: ENTRY_HTML,
      title: 'Create Agent Task',
      action: {
        dragstart: createAgentTask,
        click: createAgentTask
      }
    }
  };

  function createAgentTask(event) {
    // Let elementFactory create the shape+businessObject (generates ID + DI)
    const shape = elementFactory.createShape({ type: 'bpmn:Task' });

    // Then set agent properties on the business object
    shape.businessObject.name = 'Agent Task';
    shape.businessObject.set('agentic:taskType', 'agent');

    create.start(event, shape);
  }
};
