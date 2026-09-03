//! Drag-pan inertia with the velocity model and easing curve of GL JS.

use std::{collections::VecDeque, time::Duration};

use cgmath::{InnerSpace, Vector2, Zero};
use instant::Instant;

/// Samples older than this no longer contribute to the release velocity.
const BUFFER_CUTOFF: Duration = Duration::from_millis(160);
const LINEARITY: f64 = 0.3;
/// Pixels per second squared.
const DECELERATION: f64 = 2500.0;
/// Pixels per second.
const MAX_SPEED: f64 = 1400.0;

struct Motion {
    started: Instant,
    duration: Duration,
    amount: Vector2<f64>,
    applied: Vector2<f64>,
}

/// Records pointer movement during a drag and eases the pan out after release.
#[derive(Default)]
pub struct PanInertia {
    samples: VecDeque<(Instant, Vector2<f64>)>,
    motion: Option<Motion>,
}

impl PanInertia {
    /// Records one pointer movement while dragging.
    pub fn record(&mut self, now: Instant, delta: Vector2<f64>) {
        self.motion = None;
        self.drain(now);
        self.samples.push_back((now, delta));
    }

    /// Drops recorded movement and any running ease-out.
    pub fn cancel(&mut self) {
        self.samples.clear();
        self.motion = None;
    }

    /// Starts easing out from the recorded movement; returns whether any motion follows.
    pub fn release(&mut self, now: Instant) -> bool {
        self.drain(now);
        let (Some((first, _)), true) = (self.samples.front(), self.samples.len() >= 2) else {
            self.samples.clear();
            return false;
        };
        let elapsed = now.saturating_duration_since(*first).as_secs_f64();
        let travelled: Vector2<f64> = self.samples.iter().map(|(_, delta)| *delta).sum();
        let distance = travelled.magnitude();
        self.samples.clear();
        if elapsed <= 0.0 || distance <= 0.0 {
            return false;
        }
        let Some(easing) = ease_out(distance, elapsed) else {
            return false;
        };
        self.motion = Some(Motion {
            started: now,
            duration: easing.duration,
            amount: travelled * (easing.distance / distance),
            applied: Vector2::zero(),
        });
        true
    }

    /// Returns the pixel delta to apply this frame, or `None` once the ease-out has finished.
    pub fn step(&mut self, now: Instant) -> Option<Vector2<f64>> {
        let motion = self.motion.as_mut()?;
        let progress = (now.saturating_duration_since(motion.started).as_secs_f64()
            / motion.duration.as_secs_f64())
        .min(1.0);
        let target = motion.amount * ease(progress);
        let delta = target - motion.applied;
        motion.applied = target;
        if progress >= 1.0 {
            self.motion = None;
        }
        Some(delta)
    }

    fn drain(&mut self, now: Instant) {
        while self
            .samples
            .front()
            .is_some_and(|(time, _)| now.saturating_duration_since(*time) > BUFFER_CUTOFF)
        {
            self.samples.pop_front();
        }
    }
}

struct EaseOut {
    duration: Duration,
    distance: f64,
}

/// GL JS `calculateEasing`: release speed clamped to `MAX_SPEED`, decelerated to a stop.
fn ease_out(distance: f64, elapsed_seconds: f64) -> Option<EaseOut> {
    let speed = (distance * LINEARITY / elapsed_seconds).min(MAX_SPEED);
    let duration = speed / (DECELERATION * LINEARITY);
    if !duration.is_finite() || duration <= 0.0 {
        return None;
    }
    Some(EaseOut {
        duration: Duration::from_secs_f64(duration),
        distance: speed * duration / 2.0,
    })
}

/// GL JS pan inertia easing: the cubic Bézier `(0, 0, 0.3, 1)`.
fn ease(progress: f64) -> f64 {
    unit_bezier(0.0, 0.0, 0.3, 1.0, progress)
}

fn unit_bezier(p1x: f64, p1y: f64, p2x: f64, p2y: f64, x: f64) -> f64 {
    let cx = 3.0 * p1x;
    let bx = 3.0 * (p2x - p1x) - cx;
    let ax = 1.0 - cx - bx;
    let cy = 3.0 * p1y;
    let by = 3.0 * (p2y - p1y) - cy;
    let ay = 1.0 - cy - by;
    let sample_x = |t: f64| ((ax * t + bx) * t + cx) * t;
    let sample_y = |t: f64| ((ay * t + by) * t + cy) * t;
    let derivative_x = |t: f64| (3.0 * ax * t + 2.0 * bx) * t + cx;

    let mut t = x;
    for _ in 0..8 {
        let error = sample_x(t) - x;
        if error.abs() < 1e-6 {
            return sample_y(t);
        }
        let slope = derivative_x(t);
        if slope.abs() < 1e-6 {
            break;
        }
        t -= error / slope;
    }
    let (mut low, mut high) = (0.0, 1.0);
    t = x;
    while low < high {
        let error = sample_x(t) - x;
        if error.abs() < 1e-6 {
            break;
        }
        if error > 0.0 {
            high = t;
        } else {
            low = t;
        }
        t = (low + high) / 2.0;
    }
    sample_y(t)
}

#[cfg(test)]
mod tests;
