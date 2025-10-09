/**
 * JSON Schema Validation using Zod
 *
 * Provides runtime type validation for handler inputs.
 *
 * Quality Standards:
 * - Zero SATD comments
 * - Complexity ≤ 20 per function
 * - Type-safe schema definitions
 */

// Using native TypeScript for now (Zod would be an external dependency)
// This provides a simple schema validation interface

/**
 * Schema type for validation
 */
export type Schema = {
  type: "object";
  properties: Record<string, PropertySchema>;
  required?: string[];
};

export type PropertySchema =
  | { type: "string"; minLength?: number; maxLength?: number }
  | { type: "number"; min?: number; max?: number }
  | { type: "boolean" }
  | { type: "array"; items?: PropertySchema }
  | { type: "object"; properties?: Record<string, PropertySchema> };

/**
 * Validation result
 */
export type ValidationResult =
  | { valid: true }
  | { valid: false; errors: string[] };

/**
 * Validate input against schema
 *
 * @param input Input to validate
 * @param schema JSON schema
 * @returns Validation result
 */
export function validate(input: unknown, schema: Schema): ValidationResult {
  const errors: string[] = [];

  // Check if input is an object
  if (typeof input !== "object" || input === null || Array.isArray(input)) {
    errors.push("Input must be an object");
    return { valid: false, errors };
  }

  const inputObj = input as Record<string, unknown>;

  // Check required fields
  if (schema.required) {
    for (const field of schema.required) {
      if (!(field in inputObj)) {
        errors.push(`Missing required field: ${field}`);
      }
    }
  }

  // Validate each property
  for (const [key, propSchema] of Object.entries(schema.properties)) {
    if (!(key in inputObj)) {
      continue; // Skip if not present and not required
    }

    const value = inputObj[key];
    const propErrors = validateProperty(value, propSchema, key);
    errors.push(...propErrors);
  }

  return errors.length === 0 ? { valid: true } : { valid: false, errors };
}

/**
 * Validate a single property
 */
function validateProperty(
  value: unknown,
  schema: PropertySchema,
  fieldName: string,
): string[] {
  const errors: string[] = [];

  switch (schema.type) {
    case "string":
      if (typeof value !== "string") {
        errors.push(`Field '${fieldName}' must be a string`);
      } else {
        if (schema.minLength && value.length < schema.minLength) {
          errors.push(
            `Field '${fieldName}' must be at least ${schema.minLength} characters`,
          );
        }
        if (schema.maxLength && value.length > schema.maxLength) {
          errors.push(
            `Field '${fieldName}' must be at most ${schema.maxLength} characters`,
          );
        }
      }
      break;

    case "number":
      if (typeof value !== "number") {
        errors.push(`Field '${fieldName}' must be a number`);
      } else {
        if (schema.min !== undefined && value < schema.min) {
          errors.push(`Field '${fieldName}' must be at least ${schema.min}`);
        }
        if (schema.max !== undefined && value > schema.max) {
          errors.push(`Field '${fieldName}' must be at most ${schema.max}`);
        }
      }
      break;

    case "boolean":
      if (typeof value !== "boolean") {
        errors.push(`Field '${fieldName}' must be a boolean`);
      }
      break;

    case "array":
      if (!Array.isArray(value)) {
        errors.push(`Field '${fieldName}' must be an array`);
      }
      break;

    case "object":
      if (typeof value !== "object" || value === null || Array.isArray(value)) {
        errors.push(`Field '${fieldName}' must be an object`);
      }
      break;
  }

  return errors;
}

/**
 * Create a schema builder for easier schema definitions
 */
export const SchemaBuilder = {
  object(
    properties: Record<string, PropertySchema>,
    required?: string[],
  ): Schema {
    return {
      type: "object",
      properties,
      required,
    };
  },

  string(options?: { minLength?: number; maxLength?: number }): PropertySchema {
    return { type: "string", ...options };
  },

  number(options?: { min?: number; max?: number }): PropertySchema {
    return { type: "number", ...options };
  },

  boolean(): PropertySchema {
    return { type: "boolean" };
  },

  array(items?: PropertySchema): PropertySchema {
    return { type: "array", items };
  },
};
