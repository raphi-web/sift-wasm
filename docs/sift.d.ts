/* tslint:disable */
/* eslint-disable */
export function resize_image(image_buffer: Uint8Array, original_width: number, original_height: number, target_long_edge: number): ResizeResult;
export function rgba_to_gray(image_buffer: Uint8Array, width: number, height: number): Uint8Array;
export function sift(image_buffer: Uint8Array, width: number, height: number, scales: number): SiftResult;
export function match_descriptors_topk(desc1: Float32Array, desc2: Float32Array, d: number, ratio: number, cross_check: boolean, top_k: number): Uint32Array;
export class ResizeResult {
  private constructor();
  free(): void;
  readonly data: Uint8Array;
  readonly width: number;
  readonly height: number;
}
export class SiftResult {
  private constructor();
  free(): void;
  readonly keypoints: Float32Array;
  readonly descriptors: Float32Array;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_resizeresult_free: (a: number, b: number) => void;
  readonly resizeresult_data: (a: number, b: number) => void;
  readonly resizeresult_width: (a: number) => number;
  readonly resizeresult_height: (a: number) => number;
  readonly resize_image: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly rgba_to_gray: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly __wbg_siftresult_free: (a: number, b: number) => void;
  readonly siftresult_keypoints: (a: number, b: number) => void;
  readonly siftresult_descriptors: (a: number, b: number) => void;
  readonly sift: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly match_descriptors_topk: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_export_0: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export_1: (a: number, b: number) => number;
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
