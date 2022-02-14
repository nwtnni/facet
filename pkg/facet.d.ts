/* tslint:disable */
/* eslint-disable */
/**
* @param {Stone} stone
* @param {number} line_0
* @param {number} line_1
* @param {number} line_2
* @param {number} roll_0
* @param {number} roll_1
* @param {number} roll_2
* @param {number} precision
* @returns {Float64Array}
*/
export function expectimax_wasm(stone: Stone, line_0: number, line_1: number, line_2: number, roll_0: number, roll_1: number, roll_2: number, precision: number): Float64Array;
/**
* Represents the set of valid success rates during faceting.
*/
export enum Chance {
  P25,
  P35,
  P45,
  P55,
  P65,
  P75,
}
/**
* Represents the current state of an ability stone being faceted.
*/
export class Stone {
  free(): void;
/**
* @param {number} chance
* @param {number} line_0
* @param {number} line_1
* @param {number} line_2
* @param {number} roll_0
* @param {number} roll_1
* @param {number} roll_2
*/
  constructor(chance: number, line_0: number, line_1: number, line_2: number, roll_0: number, roll_1: number, roll_2: number);
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_stone_free: (a: number) => void;
  readonly stone_new_wasm: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
  readonly expectimax_wasm: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
