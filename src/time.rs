use std::time::{Duration, Instant};

use bevy_ecs::system::ResMut;

pub fn time_update(mut time: ResMut<TimeResource>) {
    let el = time.last.elapsed();
    time.time += el;
    time.last = Instant::now();
}

//pub fn time_frame(mut time: ResMut<TimeResource>) {}

#[derive(Clone, Debug)]
pub struct TimeResource {
    pub update_dt: Duration,
    pub frame_dt: Duration,

    pub time: Duration, // amount of ingame time elapsed
    pub last: Instant,
    pub acc: Duration,

    pub last_frame: Duration, // length of time since last call to do_update
}

impl TimeResource {
    pub fn new(update_dt: Duration, frame_dt: Duration) -> Self {
        Self {
            update_dt,
            frame_dt,

            time: Duration::default(),
            last: Instant::now(),
            acc: Duration::default(),

            last_frame: Duration::default(),
        }
    }

    pub fn init(&mut self) {
        self.last = Instant::now();
    }
}
/*
    fn do_update<F: FnMut(Duration) -> ()>(&mut self, mut update: F) {
        // Measure elapsed time since last call to init or do_update.
        // This value is than used to calculate the amount of update time, and how long to wait extra for the next frame
        self.last_frame = self.last.elapsed();
        self.last = Instant::now();
        self.acc += self.last_frame;

        while self.acc >= self.update_dt {
            update(self.update_dt);
            self.acc -= self.update_dt;
            self.time += self.update_dt;
        }
    }

    fn do_frame<F: FnMut(f64) -> ()>(&self, mut draw: F) {
        let blend = self.acc.as_secs_f64() / self.frame_dt.as_secs_f64();
        draw(blend);
    }
}
*/
