-- Profiler Module
-- System timing, table scanning, and memory profiling
--
-- Usage:
--   local Profiler = require("modules/profiler/init.lua")
--   -- Systems are auto-registered, just require the module
--   -- Toggle UI with F3 key or Profiler.toggle()

local Profiler = {}
Profiler.__index = Profiler

--------------------------------------------------------------------------------
-- Configuration
--------------------------------------------------------------------------------

-- Global state (persisted via define_resource for hot-reload safety)
local state = define_resource("ProfilerState", {
    enabled = false,
    ui_visible = false,
    
    -- Layout settings
    parent_entity = nil,
    left_offset = nil,
    
    -- System timing data: system_name -> {count, total_ms, max_ms, avg_ms, last_ms}
    systems = {},
    
    -- Query timing data: query_signature -> {count, total_ms, max_ms, avg_ms, last_ms, last_result_count}
    queries = {},
    
    -- Table scanning
    scan_enabled = false,
    scan_interval_frames = 60,
    scan_frame_counter = 0,
    scan_max_depth = 5,
    scan_size_threshold = 50,
    scan_snapshots = {},       -- table_path -> {size, frame}
    scan_growth_alerts = {},   -- {path, old_size, new_size, growth_percent}
    
    -- Current scan state (for incremental scanning)
    scan_coroutine = nil,
    scan_results = {},
    
    -- Memory tracking
    lua_memory_kb = 0,

    -- Frame timing
    frame_times = {},          -- Rolling window of frame times (ms)
    frame_time_max = 120,      -- Keep last 120 frames (2 seconds at 60fps)
    current_fps = 0,           -- Current FPS calculated from frame_times
    
    -- UI entities
    panel_entity = nil,
    header_container = nil,
    scroll_container = nil,
    content_entities = {},
    scroll_offset = 0,
    
    -- Parallel execution info
    parallel_enabled = false,
    state_count = 0,  -- Number of unique state_ids
})

--------------------------------------------------------------------------------
-- Colors
--------------------------------------------------------------------------------

local colors = {
    border = {r = 0.08, g = 0.08, b = 0.10, a = 1.0},
    bg = {r = 0.12, g = 0.12, b = 0.14, a = 1.0},
    header_bg = {r = 0.15, g = 0.15, b = 0.18, a = 1.0},
    row_bg = {r = 0.10, g = 0.10, b = 0.12, a = 1.0},
    row_alt = {r = 0.12, g = 0.12, b = 0.14, a = 1.0},
    text = {r = 0.9, g = 0.9, b = 0.9, a = 1.0},
    text_dim = {r = 0.6, g = 0.6, b = 0.6, a = 1.0},
    text_warn = {r = 1.0, g = 0.8, b = 0.3, a = 1.0},
    text_bad = {r = 1.0, g = 0.4, b = 0.4, a = 1.0},
    text_good = {r = 0.4, g = 1.0, b = 0.6, a = 1.0},
    accent = {r = 0.3, g = 0.6, b = 1.0, a = 1.0},
}

--------------------------------------------------------------------------------
-- Table Size Counting (with depth limit to avoid infinite loops)
--------------------------------------------------------------------------------

local function count_table_entries(t, depth, max_depth, visited)
    if type(t) ~= "table" then return 0 end
    if depth > max_depth then return 0 end
    
    visited = visited or {}
    if visited[t] then return 0 end  -- Avoid cycles
    visited[t] = true
    
    local count = 0
    for k, v in pairs(t) do
        count = count + 1
        if type(v) == "table" then
            count = count + count_table_entries(v, depth + 1, max_depth, visited)
        end
    end
    return count
end

--- Count entries in a table (shallow)
local function count_shallow(t)
    if type(t) ~= "table" then return 0 end
    local count = 0
    for _ in pairs(t) do count = count + 1 end
    return count
end

--------------------------------------------------------------------------------
-- System Timing Wrapper
--------------------------------------------------------------------------------

--- Wrap a system function for profiling
--- @param name string Unique name for the system
--- @param fn function The system function to wrap
--- @return function Wrapped function
function Profiler.wrap_system(name, fn)
    return function(world)
        if not state.enabled then
            return fn(world)
        end
        
        local start_time = os.clock() * 1000  -- ms
        local result = fn(world)
        local elapsed = os.clock() * 1000 - start_time
        
        -- Update stats
        local sys = state.systems[name] or {count = 0, total_ms = 0, max_ms = 0, avg_ms = 0, last_ms = 0}
        sys.count = sys.count + 1
        sys.total_ms = sys.total_ms + elapsed
        sys.last_ms = elapsed
        sys.avg_ms = sys.total_ms / sys.count
        if elapsed > sys.max_ms then sys.max_ms = elapsed end
        state.systems[name] = sys
        
        return result
    end
end

--- Record a system timing manually (for systems you can't wrap)
--- @param name string System name
--- @param elapsed_ms number Time in milliseconds
function Profiler.record_system(name, elapsed_ms)
    if not state.enabled then return end
    
    local sys = state.systems[name] or {count = 0, total_ms = 0, max_ms = 0, avg_ms = 0, last_ms = 0}
    sys.count = sys.count + 1
    sys.total_ms = sys.total_ms + elapsed_ms
    sys.last_ms = elapsed_ms
    sys.avg_ms = sys.total_ms / sys.count
    if elapsed_ms > sys.max_ms then sys.max_ms = elapsed_ms end
    state.systems[name] = sys
end

--------------------------------------------------------------------------------
-- Table Scanning
--------------------------------------------------------------------------------

--- Start an incremental scan of global tables
local function start_global_scan()
    state.pending_scan_results = {}
    
    -- Create coroutine for incremental scanning
    state.scan_coroutine = coroutine.create(function()
        local function scan_table(t, path, depth)
            if depth > state.scan_max_depth then return end
            if type(t) ~= "table" then return end
            
            -- Skip certain problematic globals
            local skip_keys = {
                _G = true, package = true, arg = true,
                ["_VERSION"] = true, collectgarbage = true,
            }
            
            for k, v in pairs(t) do
                if type(k) == "string" and not skip_keys[k] then
                    local entry_path = path == "" and k or (path .. "." .. k)
                    
                    if type(v) == "table" then
                        local size = count_table_entries(v, 0, 3, {})  -- Limited depth for size
                        if size >= state.scan_size_threshold then
                            table.insert(state.pending_scan_results, {
                                path = entry_path,
                                size = size,
                            })
                        end
                        
                        -- Recurse (but yield periodically)
                        if depth < 2 then
                            scan_table(v, entry_path, depth + 1)
                            coroutine.yield()  -- Yield to spread work
                        end
                    end
                end
            end
        end
        
        scan_table(_G, "", 0)
        return true  -- Done
    end)
end

--- Resume the incremental scan (call each frame)
local function resume_scan()
    if not state.scan_coroutine then return end
    
    local status = coroutine.status(state.scan_coroutine)
    if status == "dead" then
        -- Scan complete - compare with previous snapshot
        local current_frame = state.scan_frame_counter
        
        -- Commit results
        state.scan_results = state.pending_scan_results or {}
        state.pending_scan_results = nil
        
        state.scan_growth_alerts = {}
        for _, result in ipairs(state.scan_results) do
            local prev = state.scan_snapshots[result.path]
            if prev then
                local growth = result.size - prev.size
                local growth_pct = prev.size > 0 and (growth / prev.size * 100) or 0
                
                -- Alert if grew by >20% or >100 entries
                if growth_pct > 20 or growth > 100 then
                    table.insert(state.scan_growth_alerts, {
                        path = result.path,
                        old_size = prev.size,
                        new_size = result.size,
                        growth = growth,
                        growth_percent = growth_pct,
                    })
                end
            end
            
            -- Update snapshot
            state.scan_snapshots[result.path] = {
                size = result.size,
                frame = current_frame,
            }
        end
        
        -- Sort alerts by growth (largest first)
        table.sort(state.scan_growth_alerts, function(a, b) return a.growth > b.growth end)
        
        state.scan_coroutine = nil
        return
    end
    
    -- Resume for a few iterations this frame
    for _ = 1, 10 do
        if coroutine.status(state.scan_coroutine) ~= "dead" then
            local ok, err = coroutine.resume(state.scan_coroutine)
            if not ok then
                print("[PROFILER] Scan error: " .. tostring(err))
                state.scan_coroutine = nil
                break
            end
        else
            break
        end
    end
end

--- Get the largest tables found in last scan
--- @param n number Max number of results
--- @return table Array of {path, size}
function Profiler.get_largest_tables(n)
    n = n or 10
    -- Sort by size descending
    local sorted = {}
    for _, v in ipairs(state.scan_results) do
        table.insert(sorted, v)
    end
    table.sort(sorted, function(a, b) return a.size > b.size end)
    
    local results = {}
    for i = 1, math.min(n, #sorted) do
        results[i] = sorted[i]
    end
    return results
end

--- Get growth alerts from last scan
--- @return table Array of {path, old_size, new_size, growth, growth_percent}
function Profiler.get_growth_alerts()
    return state.scan_growth_alerts or {}
end

--------------------------------------------------------------------------------
-- UI Panel
--------------------------------------------------------------------------------

local PANEL_WIDTH = 320
local ROW_HEIGHT = 20

--- Create a new profiler instance (singleton)
function Profiler.new()
    return Profiler
end

local function despawn_ui()
    for _, entity in ipairs(state.content_entities or {}) do
        despawn(entity)
    end
    state.content_entities = {}
    
    if state.panel_entity then
        despawn(state.panel_entity)
        state.panel_entity = nil
        state.header_container = nil
        state.scroll_container = nil
    end
end

local function format_ms(ms)
    if ms < 0.1 then return string.format("%.2fms", ms)
    elseif ms < 1 then return string.format("%.1fms", ms)
    else return string.format("%.0fms", ms) end
end

local function get_time_color(ms)
    if ms < 1 then return colors.text_good
    elseif ms < 4 then return colors.text
    elseif ms < 8 then return colors.text_warn
    else return colors.text_bad end
end

--- Spawn the main panel
--- @param left_offset number|nil Optional left offset
--- @param parent_entity number|nil Optional parent entity
function Profiler:spawn_panel(left_offset, parent_entity)
    if state.panel_entity then return state.panel_entity end
    
    state.left_offset = left_offset
    state.parent_entity = parent_entity
    
    -- Node config based on layout mode
    local node_config
    if parent_entity then
        -- Child mode (fill parent height, fixed width)
        node_config = {
            width = {Px = PANEL_WIDTH},
            height = {Percent = 100},
            flex_direction = "Column",
            border = { left = {Px = 1}, right = {Px = 1} },
        }
    elseif left_offset then
        -- Sidebar mode (Absolute left)
        node_config = {
            position_type = "Absolute",
            left = {Px = left_offset},
            top = {Px = 0},
            bottom = {Px = 0},
            width = {Px = PANEL_WIDTH},
            flex_direction = "Column",
            padding = {left = {Px = 0}, right = {Px = 0}, top = {Px = 0}, bottom = {Px = 0}},
            border = { left = {Px = 1}, right = {Px = 1} },
        }
    else
        -- Legacy mode (Absolute top-right)
        node_config = {
            position_type = "Absolute",
            right = {Px = 10},
            top = {Px = 10},
            width = {Px = PANEL_WIDTH},
            max_height = {Percent = 80},
            flex_direction = "Column",
            padding = {left = {Px = 0}, right = {Px = 0}, top = {Px = 0}, bottom = {Px = 0}},
            border = { left = {Px = 1}, right = {Px = 1} },
        }
    end
    
    -- Main panel
    local panel = spawn({
        Node = node_config,
        BackgroundColor = { color = colors.bg },
        BorderColor = {
            left = colors.border,
            right = colors.border,
        },
        GlobalZIndex = { value = 500 },
        -- Component to identify this as a sidebar panel (for escape key handling)
        SidebarPanel = { script = "modules/profiler/init.lua" },
    })
    
    if parent_entity then
        panel:with_parent(parent_entity)
    end
    
    state.panel_entity = panel:id()

    -- Fixed Header Container
    state.header_container = spawn({
        Node = {
            width = {Percent = 100},
            height = {Px = 36},
            flex_direction = "Row",
            flex_shrink = 0, -- Don't shrink
        }
    }):with_parent(state.panel_entity):id()

    -- Scrollable Content Container
    state.scroll_container = spawn({
        Node = {
            display = "Flex",
            width = {Percent = 100},
            flex_grow = 1,
            flex_direction = "Column",
            align_items = "FlexStart",
            overflow = { x = "Visible", y = "Scroll" },
            padding = { top = {Px = 4}, bottom = {Px = 4}, left = {Px = 8}, right = {Px = 8} }, 
        },
        ScrollPosition = { offset = {x = 0, y = state.scroll_offset} },
    })
    :with_parent(state.panel_entity)
    :observe("Pointer<Scroll>", function(entity, event)
        local scroll = event.event
        state.scroll_offset = state.scroll_offset - scroll.y * 20
        if state.scroll_offset < 0 then state.scroll_offset = 0 end
        entity:set({ ScrollPosition = { offset = {x = 0, y = state.scroll_offset} } })
    end)
    :id()
    
    return state.panel_entity
end

local function create_ui()
    -- Compatibility wrapper for legacy internal calls
    Profiler:spawn_panel()
end

local function update_ui_content()
    if not state.panel_entity then return end
    
    -- Clear old content (except panel itself)
    for _, entity in ipairs(state.content_entities or {}) do
        despawn(entity)
    end
    state.content_entities = {}
    
    -- Recreate header
    local header = spawn({
        Node = {
            width = {Percent = 100},
            height = {Percent = 100},
            flex_direction = "Row",
            align_items = "Center",
            justify_content = "SpaceBetween",
            padding = { left = {Px = 12}, right = {Px = 8} },
        },
        BackgroundColor = { color = colors.header_bg },
    }):with_parent(state.header_container):id()
    table.insert(state.content_entities, header)

    spawn({
        Text = { text = "[Profiler]" },
        TextFont = { font_size = 14 },
        TextColor = { color = colors.text },
    }):with_parent(header)

    -- Right side container for FPS and memory
    local right_side = spawn({
        Node = {
            flex_direction = "Row",
            column_gap = {Px = 8},
            align_items = "Center",
        },
    }):with_parent(header):id()

    -- FPS display
    spawn({
        Text = { text = string.format("FPS: %d", state.current_fps) },
        TextFont = { font_size = 11 },
        TextColor = { color = colors.text_dim },
    }):with_parent(right_side)

    -- Memory display
    spawn({
        Text = { text = string.format("%.0f KB", state.lua_memory_kb) },
        TextFont = { font_size = 11 },
        TextColor = { color = colors.text_dim },
    }):with_parent(right_side)
    
    -- Section: System Timings
    local systems_header = spawn({
        Node = {
            width = {Percent = 100},
            flex_direction = "Row",
            justify_content = "SpaceBetween",
            align_items = "Center",
            margin = {top = {Px = 4}, bottom = {Px = 4}},
        },
    }):with_parent(state.scroll_container):id()
    table.insert(state.content_entities, systems_header)
    
    spawn({
        Text = { text = "Systems" },
        TextFont = { font_size = 12 },
        TextColor = { color = colors.text_dim },
    }):with_parent(systems_header)
    
    -- Show state count and parallel mode
    local mode_text = state.parallel_enabled and "PAR" or "SEQ"
    local mode_color = state.parallel_enabled and colors.text_good or colors.text_dim
    spawn({
        Text = { text = string.format("%d states [%s]", state.state_count, mode_text) },
        TextFont = { font_size = 10 },
        TextColor = { color = mode_color },
    }):with_parent(systems_header)
    
    -- Sort systems by avg time (slowest first)
    local sorted_systems = {}
    for name, data in pairs(state.systems) do
        table.insert(sorted_systems, {name = name, data = data})
    end
    table.sort(sorted_systems, function(a, b) return a.data.avg_ms > b.data.avg_ms end)
    
    -- Display top 10 systems
    for i = 1, math.min(10, #sorted_systems) do
        local sys = sorted_systems[i]
        local row = spawn({
            Node = {
                width = {Percent = 100},
                height = {Px = ROW_HEIGHT},
                flex_direction = "Row",
                justify_content = "SpaceBetween",
                align_items = "Center",
            },
            BackgroundColor = { color = (i % 2 == 0) and colors.row_alt or colors.row_bg },
        }):with_parent(state.scroll_container):id()
        table.insert(state.content_entities, row)
        
        -- Left side: state badge + name
        local left_container = spawn({
            Node = {
                flex_direction = "Row",
                align_items = "Center",
                column_gap = {Px = 4},
            },
        }):with_parent(row):id()
        
        -- State ID badge (S0, S1, S2, etc.)
        local state_id = sys.data.state_id or 0
        spawn({
            Text = { text = string.format("S%d", state_id) },
            TextFont = { font_size = 9 },
            TextColor = { color = state_id == 0 and colors.text_dim or colors.accent },
        }):with_parent(left_container)
        
        -- Truncate long names
        local display_name = sys.name
        if #display_name > 18 then
            display_name = "..." .. display_name:sub(-15)
        end
        
        spawn({
            Text = { text = display_name },
            TextFont = { font_size = 11 },
            TextColor = { color = colors.text },
        }):with_parent(left_container)
        
        spawn({
            Text = { text = format_ms(sys.data.avg_ms) },
            TextFont = { font_size = 11 },
            TextColor = { color = get_time_color(sys.data.avg_ms) },
        }):with_parent(row)
    end
    
    -- Section: Query Timings
    local sorted_queries = {}
    for sig, data in pairs(state.queries or {}) do
        table.insert(sorted_queries, {signature = sig, data = data})
    end
    table.sort(sorted_queries, function(a, b) return a.data.avg_ms > b.data.avg_ms end)
    
    if #sorted_queries > 0 then
        local queries_header = spawn({
            Node = {
                width = {Percent = 100},
                margin = {top = {Px = 8}, bottom = {Px = 4}},
            },
        }):with_parent(state.scroll_container):id()
        table.insert(state.content_entities, queries_header)
        
        spawn({
            Text = { text = "Queries" },
            TextFont = { font_size = 12 },
            TextColor = { color = colors.text_dim },
        }):with_parent(queries_header)
        
        -- Display top 8 queries
        for i = 1, math.min(8, #sorted_queries) do
            local q = sorted_queries[i]
            local row = spawn({
                Node = {
                    width = {Percent = 100},
                    height = {Px = ROW_HEIGHT},
                    flex_direction = "Row",
                    justify_content = "SpaceBetween",
                    align_items = "Center",
                },
                BackgroundColor = { color = (i % 2 == 0) and colors.row_alt or colors.row_bg },
            }):with_parent(state.scroll_container):id()
            table.insert(state.content_entities, row)
            
            -- Truncate long signatures
            local display_sig = q.signature
            if #display_sig > 20 then
                display_sig = display_sig:sub(1, 17) .. "..."
            end
            
            spawn({
                Text = { text = display_sig },
                TextFont = { font_size = 10 },
                TextColor = { color = colors.text },
            }):with_parent(row)
            
            -- Show avg time and result count
            spawn({
                Text = { text = string.format("%s (n=%d)", format_ms(q.data.avg_ms), q.data.last_result_count or 0) },
                TextFont = { font_size = 10 },
                TextColor = { color = get_time_color(q.data.avg_ms) },
            }):with_parent(row)
        end
    end
    
    -- Section: Table Growth Alerts
    if #state.scan_growth_alerts > 0 then
        local alerts_header = spawn({
            Node = {
                width = {Percent = 100},
                margin = {top = {Px = 8}, bottom = {Px = 4}},
            },
        }):with_parent(state.scroll_container):id()
        table.insert(state.content_entities, alerts_header)
        
        spawn({
            Text = { text = "⚠ Table Growth" },
            TextFont = { font_size = 12 },
            TextColor = { color = colors.text_warn },
        }):with_parent(alerts_header)
        
        for i = 1, math.min(5, #state.scan_growth_alerts) do
            local alert = state.scan_growth_alerts[i]
            local row = spawn({
                Node = {
                    width = {Percent = 100},
                    height = {Px = ROW_HEIGHT},
                    flex_direction = "Row",
                    justify_content = "SpaceBetween",
                    align_items = "Center",
                },
                BackgroundColor = { color = colors.row_bg },
            }):with_parent(state.scroll_container):id()
            table.insert(state.content_entities, row)
            
            -- Truncate path
            local display_path = alert.path
            if #display_path > 18 then
                display_path = "..." .. display_path:sub(-15)
            end
            
            spawn({
                Text = { text = display_path },
                TextFont = { font_size = 10 },
                TextColor = { color = colors.text },
            }):with_parent(row)
            
            spawn({
                Text = { text = string.format("+%d", alert.growth) },
                TextFont = { font_size = 10 },
                TextColor = { color = colors.text_bad },
            }):with_parent(row)
        end
    end
    
    -- Section: Largest Tables
    local largest = Profiler.get_largest_tables(5)
    if #largest > 0 then
        local tables_header = spawn({
            Node = {
                width = {Percent = 100},
                margin = {top = {Px = 8}, bottom = {Px = 4}},
            },
        }):with_parent(state.scroll_container):id()
        table.insert(state.content_entities, tables_header)
        
        spawn({
            Text = { text = "Largest Tables" },
            TextFont = { font_size = 12 },
            TextColor = { color = colors.text_dim },
        }):with_parent(tables_header)
        
        for i, tbl in ipairs(largest) do
            local row = spawn({
                Node = {
                    width = {Percent = 100},
                    height = {Px = ROW_HEIGHT},
                    flex_direction = "Row",
                    justify_content = "SpaceBetween",
                    align_items = "Center",
                },
                BackgroundColor = { color = (i % 2 == 0) and colors.row_alt or colors.row_bg },
            }):with_parent(state.scroll_container):id()
            table.insert(state.content_entities, row)
            
            local display_path = tbl.path
            if #display_path > 20 then
                display_path = "..." .. display_path:sub(-17)
            end
            
            spawn({
                Text = { text = display_path },
                TextFont = { font_size = 10 },
                TextColor = { color = colors.text },
            }):with_parent(row)
            
            spawn({
                Text = { text = tostring(tbl.size) },
                TextFont = { font_size = 10 },
                TextColor = { color = tbl.size > 500 and colors.text_warn or colors.text_dim },
            }):with_parent(row)
        end
    end
end

--------------------------------------------------------------------------------
-- Public API
--------------------------------------------------------------------------------

--- Enable profiling
function Profiler.enable()
    state.enabled = true
    state.scan_enabled = true
    print("[PROFILER] Enabled")
end

--- Disable profiling
function Profiler.disable()
    state.enabled = false
    state.scan_enabled = false
    print("[PROFILER] Disabled")
end

--- Toggle profiling
function Profiler.toggle()
    if state.enabled then
        Profiler.disable()
        if state.ui_visible then
            Profiler.hide_ui()
        end
    else
        Profiler.enable()
        Profiler:show()
    end
end

--- Show the profiler UI
--- @param left_offset number|nil Optional left offset
--- @param parent_entity number|nil Optional parent entity
function Profiler:show(left_offset, parent_entity)
    if state.ui_visible then return state.panel_entity end
    state.ui_visible = true
    
    -- Ensure enabled when shown
    if not state.enabled then
        Profiler.enable()
    end
    
    self:spawn_panel(left_offset, parent_entity)
    update_ui_content()
    
    return state.panel_entity
end

--- Alias for backward compatibility
function Profiler.show_ui()
    Profiler:show()
end

--- Hide the profiler UI
function Profiler.hide_ui()
    if not state.ui_visible then return end
    state.ui_visible = false
    despawn_ui()
end

--- Toggle UI visibility
function Profiler.toggle_ui()
    if state.ui_visible then
        Profiler.hide_ui()
    else
        Profiler:show()
    end
end

--- Clear all recorded data
function Profiler.clear()
    state.systems = {}
    state.scan_snapshots = {}
    state.scan_growth_alerts = {}
    state.scan_results = {}
    state.frame_times = {}
    print("[PROFILER] Data cleared")
end

--- Print a text report to console
function Profiler.report()
    print("=== PROFILER REPORT ===")
    print(string.format("Lua Memory: %.1f KB", state.lua_memory_kb))
    print("")
    
    print("-- System Timings (avg) --")
    local sorted = {}
    for name, data in pairs(state.systems) do
        table.insert(sorted, {name = name, data = data})
    end
    table.sort(sorted, function(a, b) return a.data.avg_ms > b.data.avg_ms end)
    
    for i, sys in ipairs(sorted) do
        print(string.format("  %s: avg=%.2fms max=%.2fms count=%d",
            sys.name, sys.data.avg_ms, sys.data.max_ms, sys.data.count))
    end
    
    print("")
    print("-- Growth Alerts --")
    for _, alert in ipairs(state.scan_growth_alerts) do
        print(string.format("  %s: %d -> %d (+%.0f%%)",
            alert.path, alert.old_size, alert.new_size, alert.growth_percent))
    end
    
    print("========================")
end

--- Check if profiler is enabled
function Profiler.is_enabled()
    return state.enabled
end

--- Check if UI is visible
function Profiler.is_ui_visible()
    return state.ui_visible
end

--- Get state (for external inspection)
function Profiler.get_state()
    return state
end

--------------------------------------------------------------------------------
-- Update System
--------------------------------------------------------------------------------

local function is_key_just_pressed(world, key_code)
    local input = world:get_resource("ButtonInput<KeyCode>")
    if input and input.just_pressed then
        for _, pressed_key in ipairs(input.just_pressed) do
            for k, _ in pairs(pressed_key) do
                if k == key_code then
                    return true
                end
            end
        end
    end
    return false
end

local function profiler_update_system(world)
    if not state.enabled then return end
    
    -- Update memory usage
    state.lua_memory_kb = collectgarbage("count")
    
    -- Track frame time
    local dt = world:delta_time() * 1000  -- ms
    table.insert(state.frame_times, dt)
    while #state.frame_times > state.frame_time_max do
        table.remove(state.frame_times, 1)
    end

    -- Calculate FPS from frame times
    if #state.frame_times > 0 then
        local total_time = 0
        for _, frame_time in ipairs(state.frame_times) do
            total_time = total_time + frame_time
        end
        local avg_frame_time = total_time / #state.frame_times
        state.current_fps = avg_frame_time > 0 and math.floor(1000 / avg_frame_time + 0.5) or 0
    end
    
    -- Fetch Rust-side system timing data (more accurate than Lua os.clock)
    -- System names are now formatted as "Schedule:script_name.lua"
    local rust_stats = world:profiler_stats()
    if rust_stats then
        -- Update parallel status
        state.parallel_enabled = rust_stats.parallel_enabled or false
        
        -- Count unique state_ids and update systems
        local unique_states = {}
        if rust_stats.systems then
            for system_name, timing in pairs(rust_stats.systems) do
                local sid = timing.state_id or 0
                unique_states[sid] = true
                state.systems[system_name] = {
                    count = timing.count or 0,
                    total_ms = timing.total_ms or 0,
                    max_ms = timing.max_ms or 0,
                    avg_ms = timing.avg_ms or 0,
                    last_ms = timing.last_ms or 0,
                    state_id = sid,
                }
            end
        end
        
        -- Count unique states
        local count = 0
        for _ in pairs(unique_states) do count = count + 1 end
        state.state_count = count
    end
    
    -- Fetch query timing data
    if rust_stats and rust_stats.queries then
        for signature, timing in pairs(rust_stats.queries) do
            state.queries[signature] = {
                count = timing.count or 0,
                total_ms = timing.total_ms or 0,
                max_ms = timing.max_ms or 0,
                avg_ms = timing.avg_ms or 0,
                last_ms = timing.last_ms or 0,
                last_result_count = timing.last_result_count or 0,
            }
        end
    end
    
    -- Periodic table scanning
    if state.scan_enabled then
        state.scan_frame_counter = state.scan_frame_counter + 1
        
        if state.scan_coroutine then
            -- Continue existing scan
            resume_scan()
        elseif state.scan_frame_counter % state.scan_interval_frames == 0 then
            -- Start new scan
            start_global_scan()
        end
    end
    
    -- Update UI content periodically (every 30 frames to avoid overhead)
    if state.ui_visible and state.scan_frame_counter % 30 == 0 then
        update_ui_content()
    end
end

-- Register update system
register_system("Update", profiler_update_system)

return Profiler
