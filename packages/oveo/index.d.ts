/**
 * Hoists expression to the outermost Hoisting Scope.
 * 
 * @param v Expression.
 * @returns Expression annotated for hoisting.
 */
export function hoist<T>(v: T): T;

/**
 * Creates a new Hoisting Scope.
 * 
 * @param v Internal function.
 * @returns Internal function annotated as Hoisting Scope.
 */
export function scope<T extends Function>(v: T): T;

/**
 * Annotates expression for deduplication.
 * 
 * @param v Expression.
 * @returns Expression annotated for deduplication.
 */
export function dedupe<T>(v: T): T;

/**
 * Renames a string literal as a property name.
 * 
 * @param v String literal.
 * @returns Renamed property name.
 */
export function key(v: string): string;
