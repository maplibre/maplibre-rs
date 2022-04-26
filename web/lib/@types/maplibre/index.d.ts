interface Window {
    schedule_tile_request: (url: string, request_id: number) => void;
    newWorker: () => void;
}
