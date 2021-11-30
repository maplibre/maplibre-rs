use lyon::math::Vector;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::Window;

use crate::{DEFAULT_WINDOW_HEIGHT, DEFAULT_WINDOW_WIDTH};

pub struct SceneParams {
    pub target_zoom: f32,
    pub zoom: f32,
    pub target_scroll: Vector,
    pub scroll: Vector,
    pub show_points: bool,
    pub stroke_width: f32,
    pub target_stroke_width: f32,
    pub window_size: PhysicalSize<u32>,
    pub size_changed: bool,
    pub render: bool,
}

impl SceneParams {
    pub const DEFAULT: SceneParams = SceneParams {
        target_zoom: 5.0,
        zoom: 5.0,
        target_scroll: Vector::new(70.0, 70.0),
        scroll: Vector::new(70.0, 70.0),
        show_points: false,
        stroke_width: 1.0,
        target_stroke_width: 1.0,
        window_size: PhysicalSize::new(DEFAULT_WINDOW_WIDTH as u32, DEFAULT_WINDOW_HEIGHT as u32),
        size_changed: true,
        render: false,
    };

    pub fn update_inputs(
        self: &mut SceneParams,
        event: Event<()>,
        window: &Window,
        control_flow: &mut ControlFlow,
    ) -> bool {
        match event {
            Event::RedrawRequested(_) => {
                self.render = true;
            }
            Event::RedrawEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::Destroyed,
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
                return false;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                self.window_size = size;
                self.size_changed = true
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(key),
                                ..
                            },
                        ..
                    },
                ..
            } => match key {
                VirtualKeyCode::Escape => {
                    *control_flow = ControlFlow::Exit;
                    return false;
                }
                VirtualKeyCode::PageDown => {
                    self.target_zoom *= 0.8;
                }
                VirtualKeyCode::PageUp => {
                    self.target_zoom *= 1.25;
                }
                VirtualKeyCode::Left => {
                    self.target_scroll.x -= 50.0 / self.target_zoom;
                }
                VirtualKeyCode::Right => {
                    self.target_scroll.x += 50.0 / self.target_zoom;
                }
                VirtualKeyCode::Up => {
                    self.target_scroll.y -= 50.0 / self.target_zoom;
                }
                VirtualKeyCode::Down => {
                    self.target_scroll.y += 50.0 / self.target_zoom;
                }
                VirtualKeyCode::P => {
                    self.show_points = !self.show_points;
                }
                VirtualKeyCode::A => {
                    self.target_stroke_width /= 0.8;
                }
                VirtualKeyCode::Z => {
                    self.target_stroke_width *= 0.8;
                }
                _key => {}
            },
            _evt => {
                //println!("{:?}", _evt);
            }
        }
        //println!(" -- zoom: {}, scroll: {:?}", self.target_zoom, self.target_scroll);

        self.zoom += (self.target_zoom - self.zoom) / 3.0;
        self.scroll = self.scroll + (self.target_scroll - self.scroll) / 3.0;
        self.stroke_width =
            self.stroke_width + (self.target_stroke_width - self.stroke_width) / 5.0;

        *control_flow = ControlFlow::Poll;

        true
    }
}
