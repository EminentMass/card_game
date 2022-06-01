use std::time::{Duration, Instant};

use bevy_ecs::{schedule::ShouldRun, system::ResMut};

// TODO: This system should be split so that unsimulated time is updated and then update systems are ran before the current frame is drawn. That change will reduce felt latency
// TODO: implement blending in the render system
pub fn frame_criteria(mut time: ResMut<TimeResource>) -> ShouldRun {
    // acts as a frame limiter
    let elapsed = time.last_frame.elapsed();
    if elapsed >= time.frame_dt {
        // register passed time for update_criteria and update last frame so that next call to frame_criteria calculated the correct elapsed time
        time.last_frame = Instant::now();
        time.unsimulated_time += elapsed;

        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub fn update_criteria(mut time: ResMut<TimeResource>) -> ShouldRun {
    let dt = time.update_dt;
    // This will cause all update systems to loop as long as there is still unsimulated time.
    if time.unsimulated_time >= dt {
        // move dt time from unsimulated to ingame
        time.unsimulated_time -= dt;
        time.ingame_time += dt;

        ShouldRun::YesAndCheckAgain
    } else {
        ShouldRun::No
    }
}

/*
fn do_frame<F: FnMut(f64) -> ()>(&self, mut draw: F) {
    let blend = self.acc.as_secs_f64() / self.frame_dt.as_secs_f64();
    draw(blend);
}
*/

#[derive(Clone, Debug)]
pub struct TimeResource {
    // target delta time
    pub update_dt: Duration,
    pub frame_dt: Duration, // actual dt will be variable

    pub ingame_time: Duration, // amount of ingame time elapsed. Maybe should be replaced with tick counter and getter

    pub last_frame: Instant,
    pub unsimulated_time: Duration, // amount of realtime passed that hasn't been simulated yet. This will increase when the amount of realtime passed is not an exact multiple of update_dt
}

impl TimeResource {
    pub fn new(update_dt: Duration, frame_dt: Duration) -> Self {
        Self {
            update_dt,
            frame_dt,

            ingame_time: Duration::default(),
            last_frame: Instant::now(),
            unsimulated_time: Duration::default(),
        }
    }
}
