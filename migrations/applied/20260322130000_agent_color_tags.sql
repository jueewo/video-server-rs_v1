-- Add color and tags to agent definitions
ALTER TABLE agent_definitions ADD COLUMN color TEXT NOT NULL DEFAULT '';
ALTER TABLE agent_definitions ADD COLUMN tags TEXT NOT NULL DEFAULT '[]';
