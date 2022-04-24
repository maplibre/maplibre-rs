declare global {
    interface Window {
        schedule_tile_request: (url: string, request_id: number) => void;
        newWorker: () => void;
    }
}
export declare const startMapLibre: () => Promise<void>;
declare const _default: "test";
export default _default;
