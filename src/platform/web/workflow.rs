use crate::io::workflow::Workflow;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn create_workflow() -> *mut Workflow {
    let workflow = Box::new(Workflow::create());
    let workflow_ptr = Box::into_raw(workflow);
    return workflow_ptr;
}

#[wasm_bindgen]
pub async fn run_worker_loop(workflow_ptr: *mut Workflow) {
    let mut workflow: Box<Workflow> = unsafe { Box::from_raw(workflow_ptr) };

    // Either call forget or the worker loop to keep it alive
    workflow.download_tessellate_loop.run_loop().await;
    //std::mem::forget(workflow);
}
