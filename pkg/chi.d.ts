/* tslint:disable */
/* eslint-disable */
/**
 * @param {string} code
 * @returns {any}
 */
export function parse(code: string): any;
/**
 * @param {any} exp
 * @returns {string}
 */
export function format_abstract(exp: any): string;
/**
 * @param {any} exp
 * @returns {string}
 */
export function format_concrete(exp: any): string;
/**
 * @param {any} exp
 * @param {string} from_variable
 * @param {any} to_exp
 * @returns {any}
 */
export function substitute(exp: any, from_variable: string, to_exp: any): any;
/**
 * @param {any} exp
 * @returns {any}
 */
export function eval(exp: any): any;
/**
 * @param {any} exp
 * @returns {any}
 */
export function standard_form(exp: any): any;
export class Context {
  free(): void;
  /**
   * @param {string} variable
   * @param {number} id
   */
  set_variable(variable: string, id: number): void;
  /**
   * @returns {any}
   */
  variable_assignments(): any;
  /**
   * @returns {any}
   */
  constructor_assignments(): any;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly parse: (a: number, b: number) => number;
  readonly format_abstract: (a: number) => Array;
  readonly format_concrete: (a: number) => Array;
  readonly substitute: (a: number, b: number, c: number, d: number) => number;
  readonly eval: (a: number) => number;
  readonly standard_form: (a: number) => number;
  readonly __wbg_context_free: (a: number, b: number) => void;
  readonly context_set_variable: (a: number, b: number, c: number, d: number) => void;
  readonly context_variable_assignments: (a: number) => number;
  readonly context_constructor_assignments: (a: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
