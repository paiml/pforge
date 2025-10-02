#!/bin/bash

# Phase 2 Tickets (2001-2010)
cat > TICKET-2001_STATE_MANAGEMENT.md << 'EOF'
# TICKET-2001: State Management (Sled Backend)

**Phase**: 2 - Advanced Features
**Priority**: High  
**Time**: 4h
**Status**: Ready

## Objective
Implement StateManager with Sled backend for persistent key-value storage with TTL support.

## Implementation
- StateManager trait
- Sled backend implementation
- Memory backend for testing
- TTL support
- Concurrent access handling

## Tests
- test_sled_state_backend
- test_state_ttl
- test_concurrent_access
- test_persistence

## Acceptance
- State persists across restarts
- TTL works correctly
- Thread-safe operations
EOF

cat > TICKET-2002_RESOURCES_PROMPTS.md << 'EOF'
# TICKET-2002: Resources and Prompts

**Phase**: 2
**Priority**: High
**Time**: 3h  
**Status**: Ready

## Objective
Implement MCP Resources and Prompts support.

## Implementation
- ResourceDef handling
- URI template matching
- PromptDef with templates
- Subscribe capability

## Tests
- test_resource_uri_matching
- test_prompt_rendering
- test_resource_subscription

## Acceptance
- Resources accessible
- Prompts render correctly
- Type-safe arguments
EOF

echo "Created 2 Phase 2 tickets..."

