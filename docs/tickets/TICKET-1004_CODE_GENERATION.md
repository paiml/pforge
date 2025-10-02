# TICKET-1004: Code Generation (build.rs) Infrastructure

**Phase**: 1 - Foundation
**Cycle**: 4
**Priority**: Critical
**Estimated Time**: 4 hours
**Status**: Ready for Development
**Methodology**: EXTREME TDD
**Depends On**: TICKET-1002, TICKET-1003

See docs/specifications/pforge-specification.md for full details.

## Objective

Implement code generation infrastructure that reads pforge.yaml at build time and generates Rust code for parameter structs, handler registration, and JSON schemas.

## Key Requirements

- build.rs reads pforge.yaml
- Generate parameter structs with derives (Serialize, Deserialize, JsonSchema)
- Generate handler registration code
- Compile-time validation

## Tests (RED Phase)

- test_generate_param_struct
- test_generate_handler_registration
- test_generated_code_compiles
- test_schema_generation_correctness

