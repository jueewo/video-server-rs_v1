/**
 * Properties panel provider for agentic BPMN attributes.
 *
 * Adds an "Agent" group to the properties panel when an agentic element
 * is selected (task with agentic:taskType, lane, or gateway).
 *
 * Uses Preact's h() via the @bpmn-io/properties-panel entry components.
 */

import { TextFieldEntry, SelectEntry, NumberFieldEntry, isTextFieldEntryEdited, isSelectEntryEdited, isNumberFieldEntryEdited } from '@bpmn-io/properties-panel';
import { useService } from 'bpmn-js-properties-panel';
import { Group } from '@bpmn-io/properties-panel';

const LOW_PRIORITY = 500;

export default function AgentPropertiesProvider(propertiesPanel) {
  propertiesPanel.registerProvider(LOW_PRIORITY, this);
}

AgentPropertiesProvider.$inject = ['propertiesPanel'];

AgentPropertiesProvider.prototype.getGroups = function(element) {
  return function(groups) {
    const bo = element.businessObject;
    if (!bo) return groups;

    // Agent Task group — only show for tasks marked as agentic
    if (bo.$instanceOf && bo.$instanceOf('bpmn:Task') && bo.get('agentic:taskType') === 'agent') {
      const taskGroup = {
        id: 'agentTask',
        label: 'Agent',
        component: Group,
        entries: agentTaskEntries(element)
      };
      if (taskGroup.entries.length) {
        groups.push(taskGroup);
      }
    }

    // Agent Participant (pool) group
    if (bo.$instanceOf && bo.$instanceOf('bpmn:Participant')) {
      const participantGroup = {
        id: 'agentParticipant',
        label: 'Agent System',
        component: Group,
        entries: agentParticipantEntries(element)
      };
      if (participantGroup.entries.length) {
        groups.push(participantGroup);
      }
    }

    // Agent Lane group
    if (bo.$instanceOf && bo.$instanceOf('bpmn:Lane')) {
      const laneGroup = {
        id: 'agentLane',
        label: 'Agent',
        component: Group,
        entries: agentLaneEntries(element)
      };
      if (laneGroup.entries.length) {
        groups.push(laneGroup);
      }
    }

    // Agent Gateway group
    if (bo.$instanceOf && (bo.$instanceOf('bpmn:ParallelGateway') || bo.$instanceOf('bpmn:InclusiveGateway') || bo.$instanceOf('bpmn:ExclusiveGateway') || bo.$instanceOf('bpmn:ComplexGateway'))) {
      const gwGroup = {
        id: 'agentGateway',
        label: 'Agent Collaboration',
        component: Group,
        entries: agentGatewayEntries(element)
      };
      if (gwGroup.entries.length) {
        groups.push(gwGroup);
      }
    }

    return groups;
  };
};

// ── Task entries ────────────────────────────────────────────────────────────

function agentTaskEntries(element) {
  return [
    {
      id: 'agentic-agentId',
      component: AgentIdEntry,
      isEdited: isTextFieldEntryEdited
    },
    {
      id: 'agentic-model',
      component: ModelEntry,
      isEdited: isTextFieldEntryEdited
    },
    {
      id: 'agentic-reflectionMode',
      component: ReflectionModeEntry,
      isEdited: isSelectEntryEdited
    },
    {
      id: 'agentic-confidence',
      component: ConfidenceEntry,
      isEdited: isNumberFieldEntryEdited
    },
    {
      id: 'agentic-maxIterations',
      component: MaxIterationsEntry,
      isEdited: isNumberFieldEntryEdited
    }
  ];
}

function AgentIdEntry(props) {
  const { element } = props;
  const modeling = useService('modeling');
  const debounce = useService('debounceInput');

  return TextFieldEntry({
    id: 'agentic-agentId',
    element,
    label: 'Agent ID',
    description: 'Reference to agent definition',
    debounce,
    getValue: function(element) {
      return element.businessObject.get('agentic:agentId') || '';
    },
    setValue: function(value) {
      modeling.updateProperties(element, {
        'agentic:agentId': value || undefined
      });
    }
  });
}

function ModelEntry(props) {
  const { element } = props;
  const modeling = useService('modeling');
  const debounce = useService('debounceInput');

  return TextFieldEntry({
    id: 'agentic-model',
    element,
    label: 'LLM Model',
    description: 'Model override (e.g. claude-sonnet)',
    debounce,
    getValue: function(element) {
      return element.businessObject.get('agentic:model') || '';
    },
    setValue: function(value) {
      modeling.updateProperties(element, {
        'agentic:model': value || undefined
      });
    }
  });
}

function ReflectionModeEntry(props) {
  const { element } = props;
  const modeling = useService('modeling');
  const debounce = useService('debounceInput');

  return SelectEntry({
    id: 'agentic-reflectionMode',
    element,
    label: 'Reflection Mode',
    description: 'How the agent validates its output',
    debounce,
    getOptions: function() {
      return [
        { value: '', label: '(none)' },
        { value: 'self', label: 'Self-reflection' },
        { value: 'cross', label: 'Cross-reflection' },
        { value: 'human', label: 'Human-reflection' }
      ];
    },
    getValue: function(element) {
      return element.businessObject.get('agentic:reflectionMode') || '';
    },
    setValue: function(value) {
      modeling.updateProperties(element, {
        'agentic:reflectionMode': value || undefined
      });
    }
  });
}

function ConfidenceEntry(props) {
  const { element } = props;
  const modeling = useService('modeling');
  const debounce = useService('debounceInput');

  return NumberFieldEntry({
    id: 'agentic-confidence',
    element,
    label: 'Confidence (%)',
    description: 'Minimum confidence to accept output (0-100)',
    debounce,
    min: 0,
    max: 100,
    step: 5,
    getValue: function(element) {
      return element.businessObject.get('agentic:confidence');
    },
    setValue: function(value) {
      modeling.updateProperties(element, {
        'agentic:confidence': value !== '' ? parseInt(value, 10) : undefined
      });
    }
  });
}

function MaxIterationsEntry(props) {
  const { element } = props;
  const modeling = useService('modeling');
  const debounce = useService('debounceInput');

  return NumberFieldEntry({
    id: 'agentic-maxIterations',
    element,
    label: 'Max Iterations',
    description: 'Maximum reflection/retry loops',
    debounce,
    min: 1,
    max: 100,
    step: 1,
    getValue: function(element) {
      return element.businessObject.get('agentic:maxIterations');
    },
    setValue: function(value) {
      modeling.updateProperties(element, {
        'agentic:maxIterations': value !== '' ? parseInt(value, 10) : undefined
      });
    }
  });
}

// ── Participant (pool) entries ───────────────────────────────────────────────

function agentParticipantEntries(element) {
  return [
    {
      id: 'agentic-systemType',
      component: SystemTypeEntry,
      isEdited: isSelectEntryEdited
    },
    {
      id: 'agentic-systemDescription',
      component: SystemDescriptionEntry,
      isEdited: isTextFieldEntryEdited
    }
  ];
}

function SystemTypeEntry(props) {
  const { element } = props;
  const modeling = useService('modeling');
  const debounce = useService('debounceInput');

  return SelectEntry({
    id: 'agentic-systemType',
    element,
    label: 'System Type',
    description: 'Is this pool a human team or an agentic system?',
    debounce,
    getOptions: function() {
      return [
        { value: '', label: '(default — human)' },
        { value: 'agentic', label: 'Agentic system' },
        { value: 'hybrid', label: 'Hybrid (human + agents)' }
      ];
    },
    getValue: function(element) {
      return element.businessObject.get('agentic:systemType') || '';
    },
    setValue: function(value) {
      modeling.updateProperties(element, {
        'agentic:systemType': value || undefined
      });
    }
  });
}

function SystemDescriptionEntry(props) {
  const { element } = props;
  const modeling = useService('modeling');
  const debounce = useService('debounceInput');

  return TextFieldEntry({
    id: 'agentic-systemDescription',
    element,
    label: 'System Description',
    description: 'Purpose of this agentic system',
    debounce,
    getValue: function(element) {
      return element.businessObject.get('agentic:systemDescription') || '';
    },
    setValue: function(value) {
      modeling.updateProperties(element, {
        'agentic:systemDescription': value || undefined
      });
    }
  });
}

// ── Lane entries ────────────────────────────────────────────────────────────

function agentLaneEntries(element) {
  return [
    {
      id: 'agentic-agentRole',
      component: AgentRoleEntry,
      isEdited: isSelectEntryEdited
    },
    {
      id: 'agentic-trustScore',
      component: TrustScoreEntry,
      isEdited: isNumberFieldEntryEdited
    }
  ];
}

function AgentRoleEntry(props) {
  const { element } = props;
  const modeling = useService('modeling');
  const debounce = useService('debounceInput');

  return SelectEntry({
    id: 'agentic-agentRole',
    element,
    label: 'Agent Role',
    description: 'Role in collaboration (manager decides)',
    debounce,
    getOptions: function() {
      return [
        { value: '', label: '(none — human participant)' },
        { value: 'manager', label: 'Manager' },
        { value: 'worker', label: 'Worker' }
      ];
    },
    getValue: function(element) {
      return element.businessObject.get('agentic:agentRole') || '';
    },
    setValue: function(value) {
      modeling.updateProperties(element, {
        'agentic:agentRole': value || undefined
      });
    }
  });
}

function TrustScoreEntry(props) {
  const { element } = props;
  const modeling = useService('modeling');
  const debounce = useService('debounceInput');

  return NumberFieldEntry({
    id: 'agentic-trustScore',
    element,
    label: 'Trust Score (%)',
    description: 'Overall reliability of this agent (0-100)',
    debounce,
    min: 0,
    max: 100,
    step: 5,
    getValue: function(element) {
      return element.businessObject.get('agentic:trustScore');
    },
    setValue: function(value) {
      modeling.updateProperties(element, {
        'agentic:trustScore': value !== '' ? parseInt(value, 10) : undefined
      });
    }
  });
}

// ── Gateway entries ─────────────────────────────────────────────────────────

function agentGatewayEntries(element) {
  return [
    {
      id: 'agentic-collaborationMode',
      component: CollaborationModeEntry,
      isEdited: isSelectEntryEdited
    },
    {
      id: 'agentic-mergingStrategy',
      component: MergingStrategyEntry,
      isEdited: isSelectEntryEdited
    }
  ];
}

function CollaborationModeEntry(props) {
  const { element } = props;
  const modeling = useService('modeling');
  const debounce = useService('debounceInput');

  return SelectEntry({
    id: 'agentic-collaborationMode',
    element,
    label: 'Collaboration Mode',
    description: 'How agents collaborate (diverging gateway)',
    debounce,
    getOptions: function() {
      return [
        { value: '', label: '(none)' },
        { value: 'voting', label: 'Voting' },
        { value: 'role', label: 'Role-based' },
        { value: 'debate', label: 'Debate' },
        { value: 'competition', label: 'Competition' }
      ];
    },
    getValue: function(element) {
      return element.businessObject.get('agentic:collaborationMode') || '';
    },
    setValue: function(value) {
      modeling.updateProperties(element, {
        'agentic:collaborationMode': value || undefined
      });
    }
  });
}

function MergingStrategyEntry(props) {
  const { element } = props;
  const modeling = useService('modeling');
  const debounce = useService('debounceInput');

  return SelectEntry({
    id: 'agentic-mergingStrategy',
    element,
    label: 'Merging Strategy',
    description: 'How to select/combine results (converging gateway)',
    debounce,
    getOptions: function() {
      return [
        { value: '', label: '(none)' },
        { value: 'majority', label: 'Majority vote' },
        { value: 'leader-driven', label: 'Leader-driven (manager decides)' },
        { value: 'composed', label: 'Composed (combine all)' },
        { value: 'fastest', label: 'Fastest response' },
        { value: 'most-complete', label: 'Most complete' }
      ];
    },
    getValue: function(element) {
      return element.businessObject.get('agentic:mergingStrategy') || '';
    },
    setValue: function(value) {
      modeling.updateProperties(element, {
        'agentic:mergingStrategy': value || undefined
      });
    }
  });
}
