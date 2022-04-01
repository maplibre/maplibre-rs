//! Main (platform-specific) main loop which handles:
//! * Input (Mouse/Keyboard)
//! * Platform Events like suspend/resume
//! * Render a new frame

use log::{error, info, trace};
use style_spec::Style;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use crate::input::{InputController, UpdateState};

use crate::io::scheduler::IOScheduler;
use crate::platform::Instant;
use crate::render::render_state::RenderState;

#[cfg(feature = "enable-tracing")]
fn enable_tracing() {
    use opentelemetry::sdk::export::trace::stdout;
    use opentelemetry_jaeger;
    use tracing::{error, span};
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    // Install a new OpenTelemetry trace pipeline
    /*let tracer = stdout::new_pipeline().install_simple();*/
    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("mapr")
        .install_simple()
        .unwrap();

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    let subscriber = Registry::default().with(telemetry);
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

pub async fn run(
    window: winit::window::Window,
    event_loop: EventLoop<()>,
    mut scheduler: Box<IOScheduler>,
    style: Box<Style>,
    max_frames: Option<u64>,
) {
    #[cfg(feature = "enable-tracing")]
    enable_tracing();
    #[cfg(feature = "enable-tracing")]
    let root = tracing::span!(tracing::Level::TRACE, "app_start", work_units = 2);
    #[cfg(feature = "enable-tracing")]
    let _enter = root.enter();

    let mut input = InputController::new(0.2, 100.0, 0.1);
    let mut maybe_state: Option<RenderState> = {
        #[cfg(target_os = "android")]
        {
            None
        }
        #[cfg(not(target_os = "android"))]
        {
            Some(RenderState::new(&window, style).await)
        }
    };

    let mut last_render_time = Instant::now();

    let mut current_frame: u64 = 0;

    event_loop.run(move |event, _, control_flow| {
        /* FIXME:   On Android we need to initialize the surface on Event::Resumed. On desktop this
                    event is not fired and we can do surface initialization anytime. Clean this up.
        */
        #[cfg(target_os = "android")]
        if maybe_state.is_none() && event == Event::Resumed {
            use tokio::runtime::Handle;
            use tokio::task;

            let state = task::block_in_place(|| {
                Handle::current().block_on(async { RenderState::new(&window, style.clone()).await })
            });
            maybe_state = Some(state);
            return;
        }

        if let Some(state) = maybe_state.as_mut() {
            match event {
                Event::DeviceEvent {
                    ref event,
                    .. // We're not using device_id currently
                } => {
                    trace!("{:?}", event);
                    input.device_input(event);
                }

                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => {
                    if !input.window_input(event, state) {
                        match event {
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            WindowEvent::Resized(physical_size) => {
                                state.resize(*physical_size);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                // new_inner_size is &mut so w have to dereference it twice
                                state.resize(**new_inner_size);
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(_) => {
                    let _span_ = tracing::span!(tracing::Level::TRACE, "redraw requested").entered();
                    let now = Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;

                    scheduler.try_populate_cache();

                    input.update_state(state, scheduler.get_tile_cache(), dt);
                    state.prepare_render_data(&mut scheduler);
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            error!("Surface Lost");
                        },
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            error!("Out of Memory");
                            *control_flow = ControlFlow::Exit;
                        },
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    };

                    current_frame += 1;

                    if let Some(max_frames) = max_frames {
                        if current_frame >= max_frames {
                            info!("Exiting because maximum frames reached.");
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                Event::Suspended => {
                    state.suspend();
                }
                Event::Resumed => {
                    state.recreate_surface(&window);
                    state.resize(window.inner_size()); // FIXME: Resumed is also called when the app launches for the first time. Instead of first using a "fake" inner_size() in State::new we should initialize with a proper size from the beginning
                    state.resume();
                }
                Event::MainEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    window.request_redraw();
                }
                _ => {}
            }
        }
    });
}
