use crate::{
    context::MapContext,
    tcs::system::{SystemError, SystemResult},
    util::FPSMeter,
};

pub fn fps_system(MapContext { world, .. }: &mut MapContext) -> SystemResult {
    let Some(fps_meter) = world.resources.get_mut::<FPSMeter>() else {
        return Err(SystemError::Dependencies);
    };

    fps_meter.update_and_print();

    Ok(())
}
