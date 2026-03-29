import BaseRenderer from 'diagram-js/lib/draw/BaseRenderer';

const HIGH_PRIORITY = 1500;

/**
 * Custom renderer that draws an agent icon on tasks marked as agentic.
 *
 * Detection (in order):
 * 1. Attribute: agentic:taskType="agent" on bpmn:Task (via moddle extends)
 * 2. Fallback: task name contains "[agent]" (for imported diagrams)
 */
export default function AgentRenderer(eventBus, bpmnRenderer) {
  BaseRenderer.call(this, eventBus, HIGH_PRIORITY);
  this._bpmnRenderer = bpmnRenderer;
}

AgentRenderer.$inject = ['eventBus', 'bpmnRenderer'];

AgentRenderer.prototype = Object.create(BaseRenderer.prototype);
AgentRenderer.prototype.constructor = AgentRenderer;

AgentRenderer.prototype.canRender = function(element) {
  return isAgentTask(element);
};

AgentRenderer.prototype.drawShape = function(parentGfx, element) {
  // Delegate base shape to default renderer
  const shape = this._bpmnRenderer.drawShape(parentGfx, element);

  // Overlay agent icon
  drawAgentMarker(parentGfx);

  return shape;
};

/**
 * Check if element is an agent task via attribute or name fallback.
 */
function isAgentTask(element) {
  const bo = element.businessObject;
  if (!bo || !bo.$instanceOf) return false;

  // Only match generic bpmn:Task (not serviceTask, userTask, etc.)
  if (!bo.$instanceOf('bpmn:Task')) return false;
  if (bo.$instanceOf('bpmn:ServiceTask') ||
      bo.$instanceOf('bpmn:UserTask') ||
      bo.$instanceOf('bpmn:ScriptTask') ||
      bo.$instanceOf('bpmn:SendTask') ||
      bo.$instanceOf('bpmn:ReceiveTask') ||
      bo.$instanceOf('bpmn:ManualTask') ||
      bo.$instanceOf('bpmn:BusinessRuleTask')) {
    return false;
  }

  // 1. Check agentic:taskType attribute (set via moddle extends)
  if (bo.get('agentic:taskType') === 'agent') {
    return true;
  }

  // 2. Fallback: name contains [agent]
  const name = (bo.name || '').toLowerCase();
  return name.includes('[agent]');
}

/**
 * Draw a purple robot icon SVG in the top-left corner of the task.
 */
function drawAgentMarker(parentGfx) {
  const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
  g.setAttribute('transform', 'translate(5, 3)');

  // Robot head outline (simplified)
  const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
  path.setAttribute('d', [
    // Antenna
    'M 7 1 L 7 3',
    // Head
    'M 2 3 L 12 3 L 12 10 L 2 10 Z',
    // Left eye
    'M 4 5.5 L 5.5 5.5 L 5.5 7 L 4 7 Z',
    // Right eye
    'M 8.5 5.5 L 10 5.5 L 10 7 L 8.5 7 Z',
    // Mouth
    'M 5 8.5 L 9 8.5',
    // Body
    'M 3 10.5 L 11 10.5 L 11 14 L 3 14 Z',
    // Left arm
    'M 1 11 L 3 11',
    // Right arm
    'M 11 11 L 13 11',
  ].join(' '));
  path.setAttribute('fill', 'none');
  path.setAttribute('stroke', '#7c3aed');
  path.setAttribute('stroke-width', '1.2');
  path.setAttribute('stroke-linecap', 'round');
  path.setAttribute('stroke-linejoin', 'round');

  g.appendChild(path);
  parentGfx.appendChild(g);
}
