/**
 * Hoists expression to the outermost Hoisting Scope.
 * 
 * @param v Expression.
 * @returns Expression annotated for hoisting.
 */
export const hoist = (v) => v;

/**
 * Creates a new Hoisting Scope.
 * 
 * @param v Internal function.
 * @returns Internal function annotated as Hoisting Scope.
 */
export const scope = (v) => v;

/**
 * Annotates expression for deduplication.
 * 
 * @param v Expression.
 * @returns Expression annotated for deduplication.
 */
export const dedupe = (v) => v;

/**
 * Renames a string literal as a property name.
 * 
 * @param v String literal.
 * @returns Renamed property name.
 */
export const key = (v) => v;