use bevy::prelude::*;
use std::time::Instant;

/// Tracks frame timing to identify bottlenecks
#[derive(Resource)]
pub struct FrameProfiler {
    last_frame_start: Instant,
    frame_count: u64,
    lua_time_ms: f64,
    update_queue_time_ms: f64,
}

impl Default for FrameProfiler {
    fn default() -> Self {
        Self {
            last_frame_start: Instant::now(),
            frame_count: 0,
            lua_time_ms: 0.0,
            update_queue_time_ms: 0.0,
        }
    }
}

impl FrameProfiler {
    pub fn record_lua_time(&mut self, ms: f64) {
        self.lua_time_ms = ms;
    }

    pub fn record_update_queue_time(&mut self, ms: f64) {
        self.update_queue_time_ms = ms;
    }
}

/// System that runs at the very start of the frame
fn frame_start_system(mut profiler: ResMut<FrameProfiler>) {
    profiler.last_frame_start = Instant::now();
}

/// System that runs at the very end of the frame to report timings
fn frame_end_system(mut profiler: ResMut<FrameProfiler>) {
    let total_frame_time = profiler.last_frame_start.elapsed().as_secs_f64() * 1000.0;
    profiler.frame_count += 1;

    // Log every frame when FPS is low, or every second when FPS is good
    let fps = 1000.0 / total_frame_time;
    let should_log = fps < 40.0 || profiler.frame_count % 60 == 0;

    if should_log {
        let other_time = total_frame_time - profiler.lua_time_ms - profiler.update_queue_time_ms;

        debug!(
            "[FRAME_PROFILE] frame={} fps={:.1} total={:.2}ms | lua={:.2}ms ({:.1}%) queue={:.2}ms ({:.1}%) OTHER={:.2}ms ({:.1}%)",
            profiler.frame_count,
            fps,
            total_frame_time,
            profiler.lua_time_ms,
            (profiler.lua_time_ms / total_frame_time * 100.0),
            profiler.update_queue_time_ms,
            (profiler.update_queue_time_ms / total_frame_time * 100.0),
            other_time,
            (other_time / total_frame_time * 100.0)
        );
    }
}

pub struct FrameProfilerPlugin;

impl Plugin for FrameProfilerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameProfiler>();

        // Run at the very start of First
        app.add_systems(First, frame_start_system);

        // Run at the very end of Last
        app.add_systems(Last, frame_end_system);
    }
}
