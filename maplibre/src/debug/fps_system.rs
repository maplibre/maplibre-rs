use crate::{context::MapContext, util::FPSMeter};

pub fn fps_system(MapContext { world, .. }: &mut MapContext) {
    let Some(fps_meter) = world.resources.get_mut::<FPSMeter>() else {
        return;
    };

    fps_meter.update_and_print()
}
