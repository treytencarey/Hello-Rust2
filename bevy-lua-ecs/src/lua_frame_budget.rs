//! Lua frame budget system for time-sliced execution
//! 
//! Ensures Lua systems don't exceed a configurable time budget per frame.
//! Systems that exceed the budget are deferred to the next frame.

use bevy::prelude::*;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

/// Resource controlling the maximum time Lua systems can run per frame
#[derive(Resource, Clone)]
pub struct LuaFrameBudget {
    /// Maximum time in seconds allocated for Lua systems per frame
    /// Default: 4ms (good for 60fps with overhead buffer)
    pub max_seconds: f32,
    
    /// Whether time-slicing is enabled
    /// When disabled, all systems run every frame regardless of time
    pub enabled: bool,
}

impl Default for LuaFrameBudget {
    fn default() -> Self {
        Self {
            max_seconds: 0.004, // 4ms default
            enabled: true,
        }
    }
}

impl LuaFrameBudget {
    /// Create a new budget with the specified max time in milliseconds
    pub fn with_max_ms(ms: f32) -> Self {
        Self {
            max_seconds: ms / 1000.0,
            enabled: true,
        }
    }
    
    /// Disable time-slicing (all systems run every frame)
    pub fn disabled() -> Self {
        Self {
            max_seconds: 0.0,
            enabled: false,
        }
    }
}

/// Tracks which systems have been run and which are pending
#[derive(Resource, Clone)]
pub struct LuaSystemProgress {
    inner: Arc<Mutex<ProgressInner>>,
}

#[derive(Default)]
struct ProgressInner {
    /// Index of the next system to run
    /// Wraps around when all systems complete
    next_index: usize,
    
    /// Frame number of the last run (for detecting new frames)
    last_frame: u64,
    
    /// Time spent on Lua systems this frame
    time_this_frame: Duration,
    
    /// Number of frames where budget was exceeded (for diagnostics)
    exceeded_count: u64,
    
    /// Per-system timing data: system_name -> timing
    /// Used by the Lua profiler to show which systems are slow
    system_timings: std::collections::HashMap<String, SystemTiming>,
    
    /// Per-query timing data: query_signature -> timing
    /// Used by the Lua profiler to show which queries are slow
    query_timings: std::collections::HashMap<String, QueryTiming>,
}

/// Timing data for a single Lua system
#[derive(Clone, Default)]
pub struct SystemTiming {
    pub call_count: u64,
    pub total_ms: f64,
    pub max_ms: f64,
    pub last_ms: f64,
    /// State ID for parallel execution tracking (0 = primary state)
    pub state_id: usize,
}

/// Timing data for a query type
#[derive(Clone, Default)]
pub struct QueryTiming {
    pub call_count: u64,
    pub total_ms: f64,
    pub max_ms: f64,
    pub last_ms: f64,
    pub last_result_count: usize,
}

impl Default for LuaSystemProgress {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ProgressInner::default())),
        }
    }
}

impl LuaSystemProgress {
    /// Called at start of frame to reset if needed
    pub fn new_frame(&self, current_frame: u64) {
        let mut inner = self.inner.lock().unwrap();
        if inner.last_frame != current_frame {
            // New frame - if we had pending systems, log it
            if inner.time_this_frame > Duration::ZERO {
                debug!(
                    "[LUA_BUDGET] Frame {} spent {:?} on Lua systems (budget exceeded {} times total)",
                    inner.last_frame, inner.time_this_frame, inner.exceeded_count
                );
            }
            inner.last_frame = current_frame;
            inner.time_this_frame = Duration::ZERO;
            // NOTE: We don't reset next_index - we continue from where we left off
            // This ensures fairness: deferred systems get to run first next frame
        }
    }
    
    /// Get the index of the next system to run
    pub fn next_index(&self) -> usize {
        self.inner.lock().unwrap().next_index
    }
    
    /// Advance to the next system, wrapping around at total_count
    pub fn advance(&self, total_count: usize) {
        let mut inner = self.inner.lock().unwrap();
        inner.next_index = (inner.next_index + 1) % total_count.max(1);
    }
    
    /// Record time spent and check if budget is exceeded
    /// Returns true if we should continue, false if budget exceeded
    pub fn record_time(&self, elapsed: Duration, budget: &LuaFrameBudget) -> bool {
        if !budget.enabled {
            return true; // Always continue if budget disabled
        }
        
        let mut inner = self.inner.lock().unwrap();
        inner.time_this_frame += elapsed;
        
        let budget_duration = Duration::from_secs_f32(budget.max_seconds);
        if inner.time_this_frame >= budget_duration {
            inner.exceeded_count += 1;
            false
        } else {
            true
        }
    }
    
    /// Get time spent on Lua systems this frame
    pub fn time_this_frame(&self) -> Duration {
        self.inner.lock().unwrap().time_this_frame
    }
    
    /// Get the count of frames where budget was exceeded
    pub fn exceeded_count(&self) -> u64 {
        self.inner.lock().unwrap().exceeded_count
    }
    
    /// Record timing for a specific system (by script path)
    pub fn record_system_time(&self, script_path: String, elapsed: Duration, state_id: usize) {
        let mut inner = self.inner.lock().unwrap();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        
        let timing = inner.system_timings.entry(script_path).or_default();
        timing.call_count += 1;
        timing.total_ms += elapsed_ms;
        timing.last_ms = elapsed_ms;
        timing.state_id = state_id;
        if elapsed_ms > timing.max_ms {
            timing.max_ms = elapsed_ms;
        }
    }
    
    /// Get all system timings (for Lua profiler)
    pub fn get_system_timings(&self) -> std::collections::HashMap<String, SystemTiming> {
        self.inner.lock().unwrap().system_timings.clone()
    }
    
    /// Clear all timing data
    pub fn clear_timings(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.system_timings.clear();
        inner.query_timings.clear();
    }
    
    /// Record timing for a query
    pub fn record_query_time(&self, signature: String, elapsed: Duration, result_count: usize) {
        let mut inner = self.inner.lock().unwrap();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        
        let timing = inner.query_timings.entry(signature).or_default();
        timing.call_count += 1;
        timing.total_ms += elapsed_ms;
        timing.last_ms = elapsed_ms;
        timing.last_result_count = result_count;
        if elapsed_ms > timing.max_ms {
            timing.max_ms = elapsed_ms;
        }
    }
    
    /// Get all query timings (for Lua profiler)
    pub fn get_query_timings(&self) -> std::collections::HashMap<String, QueryTiming> {
        self.inner.lock().unwrap().query_timings.clone()
    }
}

/// Timer for measuring individual system execution time
pub struct SystemTimer {
    start: Instant,
}

impl SystemTimer {
    pub fn start() -> Self {
        Self { start: Instant::now() }
    }
    
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}
