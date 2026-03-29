/**
 * Custom token simulation behavior for agent tasks.
 *
 * By default, bpmn:Task (generic) requires manual triggering in the simulation.
 * This behavior overrides that: when the simulator detects an agent task,
 * it auto-continues after a brief delay (simulating agent processing time).
 *
 * We hook into the simulator's 'trace' event to detect when a bpmn:Task with
 * agent markers enters, and automatically trigger its continuation.
 */
export default function AgentTaskBehavior(eventBus, simulator) {

  eventBus.on('tokenSimulation.simulator.trace', function(event) {

    if (event.action !== 'enter') {
      return;
    }

    const element = event.element;

    if (!element || element.type !== 'bpmn:Task') {
      return;
    }

    if (!isAgentTask(element)) {
      return;
    }

    // Find the subscription (wait/continue event) for this element
    // and trigger it after a short delay to simulate processing
    setTimeout(function() {
      const subscriptions = simulator.findSubscriptions({
        element
      });

      if (subscriptions.length) {
        simulator.trigger({
          event: subscriptions[0].event,
          scope: subscriptions[0].scope
        });
      }
    }, 500);
  });
}

AgentTaskBehavior.$inject = ['eventBus', 'simulator'];

function isAgentTask(element) {
  const bo = element.businessObject;
  if (!bo) return false;

  // Check agentic:taskType attribute (set via moddle extends)
  if (bo.get && bo.get('agentic:taskType') === 'agent') {
    return true;
  }

  // Fallback: name contains [agent]
  const name = (bo.name || '').toLowerCase();
  return name.includes('[agent]');
}
