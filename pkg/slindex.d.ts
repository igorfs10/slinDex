/* tslint:disable */
/* eslint-disable */

export function start_wasm(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly start_wasm: () => void;
    readonly __wasm_bindgen_func_elem_4257: (a: number, b: number, c: any, d: any) => void;
    readonly __wasm_bindgen_func_elem_8730: (a: number, b: number, c: number, d: any) => void;
    readonly __wasm_bindgen_func_elem_2891: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_2891_10: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_2891_11: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_2891_15: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_2891_6: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_2891_8: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_4252: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_4253: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_4253_12: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_4253_13: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_4253_14: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_4253_16: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_4253_4: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_4253_7: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_4253_9: (a: number, b: number, c: any) => void;
    readonly __wasm_bindgen_func_elem_4265: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_4905: (a: number, b: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export4: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_export5: (a: number) => void;
    readonly __wbindgen_export6: (a: number, b: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export7: (a: number) => void;
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
