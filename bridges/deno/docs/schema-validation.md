# Schema Validation Guide

> Runtime type validation for TypeScript MCP handlers

## Overview

The pforge Deno bridge includes a built-in schema validation system that provides runtime type checking for handler inputs. This ensures that your handlers receive correctly-typed data and provides clear error messages when validation fails.

**Key Benefits:**
- ✅ Zero external dependencies (no Zod/Joi required)
- ✅ Type-safe schema definitions
- ✅ Clear validation error messages
- ✅ Optional - only validates when schema is provided
- ✅ Composable schemas via SchemaBuilder

## Quick Start

```typescript
import { PforgeBridge } from "../bridge.ts";
import { SchemaBuilder } from "../schema.ts";

const bridge = new PforgeBridge();

bridge.register({
  name: "create_user",
  description: "Create a new user",
  handler: (input: { name: string; age: number; email: string }) => ({
    success: true,
    data: { id: 123, ...input }
  }),
  inputSchema: SchemaBuilder.object({
    name: SchemaBuilder.string({ minLength: 1, maxLength: 100 }),
    age: SchemaBuilder.number({ min: 0, max: 120 }),
    email: SchemaBuilder.string({ minLength: 5 }),
  }, ["name", "age", "email"]), // Required fields
});

// Valid input - passes validation
const result1 = await bridge.execute("create_user", {
  name: "Alice",
  age: 30,
  email: "alice@example.com"
});
// => { success: true, data: { id: 123, name: "Alice", age: 30, email: "..." } }

// Invalid input - fails validation
const result2 = await bridge.execute("create_user", {
  name: "Alice",
  age: -5, // Invalid: age must be >= 0
});
// => { success: false, error: "Validation failed: Field 'age' must be at least 0" }
```

## Schema Types

### String

Validate string values with optional length constraints.

```typescript
SchemaBuilder.string()                         // Any string
SchemaBuilder.string({ minLength: 3 })        // At least 3 characters
SchemaBuilder.string({ maxLength: 20 })       // At most 20 characters
SchemaBuilder.string({ minLength: 3, maxLength: 20 }) // Between 3-20 characters
```

**Example:**

```typescript
bridge.register({
  name: "set_username",
  handler: (input: { username: string }) => ({
    success: true,
    data: { username: input.username }
  }),
  inputSchema: SchemaBuilder.object({
    username: SchemaBuilder.string({ minLength: 3, maxLength: 20 }),
  }, ["username"]),
});
```

### Number

Validate numeric values with optional range constraints.

```typescript
SchemaBuilder.number()              // Any number
SchemaBuilder.number({ min: 0 })    // At least 0
SchemaBuilder.number({ max: 100 })  // At most 100
SchemaBuilder.number({ min: 0, max: 100 }) // Between 0-100
```

**Example:**

```typescript
bridge.register({
  name: "set_age",
  handler: (input: { age: number }) => ({
    success: true,
    data: { age: input.age }
  }),
  inputSchema: SchemaBuilder.object({
    age: SchemaBuilder.number({ min: 0, max: 120 }),
  }, ["age"]),
});
```

### Boolean

Validate boolean values.

```typescript
SchemaBuilder.boolean() // true or false
```

**Example:**

```typescript
bridge.register({
  name: "set_active",
  handler: (input: { active: boolean }) => ({
    success: true,
    data: { active: input.active }
  }),
  inputSchema: SchemaBuilder.object({
    active: SchemaBuilder.boolean(),
  }, ["active"]),
});
```

### Array

Validate array values.

```typescript
SchemaBuilder.array()              // Any array
SchemaBuilder.array(itemSchema)    // Array with item type validation (future)
```

**Example:**

```typescript
bridge.register({
  name: "set_tags",
  handler: (input: { tags: string[] }) => ({
    success: true,
    data: { tags: input.tags }
  }),
  inputSchema: SchemaBuilder.object({
    tags: SchemaBuilder.array(),
  }, ["tags"]),
});
```

### Object

Validate object values.

```typescript
SchemaBuilder.object(properties, required)
```

**Example:**

```typescript
bridge.register({
  name: "create_profile",
  handler: (input: {
    personal: { name: string; age: number };
    preferences: { theme: string };
  }) => ({
    success: true,
    data: { ...input }
  }),
  inputSchema: SchemaBuilder.object({
    personal: SchemaBuilder.object(),
    preferences: SchemaBuilder.object(),
  }, ["personal"]), // personal is required, preferences is optional
});
```

## Required vs Optional Fields

Fields are **optional by default**. Specify required fields in the second parameter:

```typescript
SchemaBuilder.object({
  name: SchemaBuilder.string(),
  email: SchemaBuilder.string(),
  nickname: SchemaBuilder.string(),
}, ["name", "email"]) // name and email required, nickname optional
```

**Example:**

```typescript
// Valid: includes required fields
const result1 = await bridge.execute("handler", {
  name: "Alice",
  email: "alice@example.com"
});

// Valid: includes optional field
const result2 = await bridge.execute("handler", {
  name: "Alice",
  email: "alice@example.com",
  nickname: "Ally"
});

// Invalid: missing required field 'email'
const result3 = await bridge.execute("handler", {
  name: "Alice"
});
// => { success: false, error: "Validation failed: Missing required field: email" }
```

## Validation Errors

When validation fails, the error message includes all validation failures:

```typescript
const result = await bridge.execute("create_user", {
  name: "", // Too short (minLength: 1)
  age: 150, // Too high (max: 120)
});

// => {
//   success: false,
//   error: "Validation failed: Field 'name' must be at least 1 characters, Field 'age' must be at most 120"
// }
```

**Error Message Format:**
- Missing required fields: `"Missing required field: <field>"`
- Type mismatch: `"Field '<field>' must be a <type>"`
- String too short: `"Field '<field>' must be at least <n> characters"`
- String too long: `"Field '<field>' must be at most <n> characters"`
- Number too small: `"Field '<field>' must be at least <n>"`
- Number too large: `"Field '<field>' must be at most <n>"`

## Common Patterns

### Email Validation (Basic)

```typescript
bridge.register({
  name: "send_email",
  handler: (input: { to: string; subject: string; body: string }) => ({
    success: true,
    data: { sent: true }
  }),
  inputSchema: SchemaBuilder.object({
    to: SchemaBuilder.string({ minLength: 5 }), // Basic email check
    subject: SchemaBuilder.string({ minLength: 1, maxLength: 200 }),
    body: SchemaBuilder.string({ minLength: 1 }),
  }, ["to", "subject", "body"]),
});
```

### Pagination Parameters

```typescript
bridge.register({
  name: "list_items",
  handler: (input: { page: number; limit: number }) => ({
    success: true,
    data: { items: [], page: input.page, limit: input.limit }
  }),
  inputSchema: SchemaBuilder.object({
    page: SchemaBuilder.number({ min: 1 }),
    limit: SchemaBuilder.number({ min: 1, max: 100 }),
  }, ["page", "limit"]),
});
```

### User Registration

```typescript
bridge.register({
  name: "register_user",
  handler: (input: {
    username: string;
    password: string;
    email: string;
    age?: number;
  }) => ({
    success: true,
    data: { id: 123, username: input.username }
  }),
  inputSchema: SchemaBuilder.object({
    username: SchemaBuilder.string({ minLength: 3, maxLength: 20 }),
    password: SchemaBuilder.string({ minLength: 8 }),
    email: SchemaBuilder.string({ minLength: 5 }),
    age: SchemaBuilder.number({ min: 13, max: 120 }),
  }, ["username", "password", "email"]), // age is optional
});
```

### Configuration Update

```typescript
bridge.register({
  name: "update_config",
  handler: (input: {
    maxRetries?: number;
    timeout?: number;
    enabled?: boolean;
  }) => ({
    success: true,
    data: { updated: true }
  }),
  inputSchema: SchemaBuilder.object({
    maxRetries: SchemaBuilder.number({ min: 0, max: 10 }),
    timeout: SchemaBuilder.number({ min: 100, max: 60000 }),
    enabled: SchemaBuilder.boolean(),
  }), // All fields optional (no required array)
});
```

## Without Schema Validation

Schema validation is **optional**. Handlers without schemas skip validation:

```typescript
bridge.register({
  name: "flexible_handler",
  description: "Accepts any input",
  handler: (input: unknown) => ({
    success: true,
    data: { received: input }
  }),
  // No inputSchema - accepts any input
});

// Any input is valid
const result = await bridge.execute("flexible_handler", {
  anything: "goes",
  number: 42,
  nested: { data: true }
});
// => { success: true, data: { received: { ... } } }
```

## Custom Validation in Handler

For complex validation logic, use schema for basic type checking and add custom validation in the handler:

```typescript
bridge.register({
  name: "create_order",
  handler: (input: { items: unknown[]; total: number }) => {
    // Schema validated: items is array, total is number

    // Custom validation
    if (input.items.length === 0) {
      return {
        success: false,
        error: "Order must contain at least one item"
      };
    }

    if (input.total < 0) {
      return {
        success: false,
        error: "Order total cannot be negative"
      };
    }

    // Process order...
    return {
      success: true,
      data: { orderId: 12345 }
    };
  },
  inputSchema: SchemaBuilder.object({
    items: SchemaBuilder.array(),
    total: SchemaBuilder.number(),
  }, ["items", "total"]),
});
```

## Performance

Schema validation is fast and efficient:

- **O(n) complexity** where n = number of fields
- **Only runs when schema is provided**
- **No impact on handlers without schemas**
- **Validation happens before handler execution**

Typical validation time: **< 1µs** for simple schemas

## Type Safety

Schema validation works seamlessly with TypeScript:

```typescript
// TypeScript type
interface CreateUserInput {
  name: string;
  age: number;
  email: string;
}

// Runtime schema (matches TypeScript type)
const createUserSchema = SchemaBuilder.object({
  name: SchemaBuilder.string({ minLength: 1 }),
  age: SchemaBuilder.number({ min: 0, max: 120 }),
  email: SchemaBuilder.string({ minLength: 5 }),
}, ["name", "age", "email"]);

// Handler with both
bridge.register({
  name: "create_user",
  handler: (input: CreateUserInput) => {
    // input is typed AND validated
    return {
      success: true,
      data: { id: 123, ...input }
    };
  },
  inputSchema: createUserSchema,
});
```

## API Reference

### `SchemaBuilder.object(properties, required?)`

Create an object schema.

**Parameters:**
- `properties`: Record of field names to property schemas
- `required`: Optional array of required field names

**Returns:** `Schema`

### `SchemaBuilder.string(options?)`

Create a string schema.

**Options:**
- `minLength`: Minimum string length
- `maxLength`: Maximum string length

**Returns:** `PropertySchema`

### `SchemaBuilder.number(options?)`

Create a number schema.

**Options:**
- `min`: Minimum value (inclusive)
- `max`: Maximum value (inclusive)

**Returns:** `PropertySchema`

### `SchemaBuilder.boolean()`

Create a boolean schema.

**Returns:** `PropertySchema`

### `SchemaBuilder.array(items?)`

Create an array schema.

**Parameters:**
- `items`: Optional item type schema (future feature)

**Returns:** `PropertySchema`

### `validate(input, schema)`

Validate input against a schema.

**Parameters:**
- `input`: Input to validate
- `schema`: Schema to validate against

**Returns:** `ValidationResult`
- `{ valid: true }` if validation passed
- `{ valid: false, errors: string[] }` if validation failed

## Testing

Test schemas with the `validate` function:

```typescript
import { SchemaBuilder, validate } from "../schema.ts";

const schema = SchemaBuilder.object({
  name: SchemaBuilder.string({ minLength: 1 }),
  age: SchemaBuilder.number({ min: 0, max: 120 }),
}, ["name", "age"]);

// Test valid input
const result1 = validate({ name: "Alice", age: 30 }, schema);
console.log(result1); // => { valid: true }

// Test invalid input
const result2 = validate({ name: "Alice" }, schema);
console.log(result2);
// => { valid: false, errors: ["Missing required field: age"] }
```

## Best Practices

1. **Use schemas for public APIs**: Always validate input from external sources
2. **Keep schemas simple**: Use custom validation in handler for complex rules
3. **Match TypeScript types**: Keep runtime schemas in sync with TypeScript types
4. **Provide clear descriptions**: Help users understand what's expected
5. **Test edge cases**: Include tests for validation errors

## Examples

See [`examples/hello_server.ts`](../examples/hello_server.ts) for working examples with schema validation.

## Troubleshooting

### Validation Passes But Handler Fails

If validation passes but the handler still fails with type errors, check:
- Is the TypeScript type broader than the schema?
- Are you handling all edge cases in the handler?

### Schema Feels Repetitive with TypeScript Types

This is by design - TypeScript provides compile-time checking, schemas provide runtime checking. Both are needed for robust applications.

Consider extracting shared types:

```typescript
// types.ts
export interface User {
  name: string;
  age: number;
}

export const userSchema = SchemaBuilder.object({
  name: SchemaBuilder.string({ minLength: 1 }),
  age: SchemaBuilder.number({ min: 0, max: 120 }),
}, ["name", "age"]);

// handler.ts
import { User, userSchema } from "./types.ts";

bridge.register({
  handler: (input: User) => ({ ... }),
  inputSchema: userSchema,
});
```

## Future Enhancements

Planned features:
- Array item type validation
- Nested object schemas
- Custom validation functions
- Regex pattern matching for strings
- Enum/union type support

---

**Related:**
- [README](../README.md) - Main documentation
- [API Reference](../README.md#api-reference) - Complete API docs
- [Examples](../examples/) - Working code examples
