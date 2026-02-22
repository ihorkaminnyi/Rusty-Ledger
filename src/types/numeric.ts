/**
 * Branded type for non-negative numbers.
 * Ensures type-level safety by requiring values to be validated through helper functions.
 */
export type NonNegativeNumber = number & { readonly __brand: 'NonNegative' };

/**
 * Creates a non-negative number from a regular number.
 * Throws an error if the value is negative.
 *
 * @param value - The number to validate
 * @returns The validated non-negative number
 * @throws {Error} If the value is negative
 */
export function nonNegative(value: number): NonNegativeNumber {
  if (value < 0) {
    throw new Error(`Expected non-negative number, got ${value}`);
  }
  return value as NonNegativeNumber;
}

/**
 * Safely attempts to create a non-negative number.
 * Returns undefined instead of throwing if the value is negative.
 *
 * @param value - The number to validate
 * @returns The validated non-negative number, or undefined if negative
 */
export function tryNonNegative(value: number): NonNegativeNumber | undefined {
  return value >= 0 ? (value as NonNegativeNumber) : undefined;
}

/**
 * Type guard to check if a number is non-negative.
 *
 * @param value - The number to check
 * @returns True if the value is non-negative
 */
export function isNonNegative(value: number): value is NonNegativeNumber {
  return value >= 0;
}
