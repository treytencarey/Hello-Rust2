-- Asset File Browser
-- Fixed-width left panel with infinite scrolling folder tree
--
-- Usage:
--   local FileBrowser = require("scripts/ui/file_browser.lua")
--   local browser = FileBrowser.new()
--   browser:show()

local FileBrowser = {}
FileBrowser.__index = FileBrowser

-- Constants
local PANEL_WIDTH = 280
local ROW_HEIGHT = 26
local INDENT_SIZE = 16
local PAGE_SIZE = 50

--- Create a new file browser
function FileBrowser.new()
    local self = setmetatable({}, FileBrowser)
    
    -- UI state
    self.is_visible = false
    self.panel_entity = nil
    self.scroll_container = nil
    self.entities = {}
    self.row_entities = {}  -- Track row entities for despawning
    
    -- Data state
    self.current_path = ""  -- Root
    self.folders = {}  -- path -> { expanded, loading, items, has_more, offset }
    self.selected_paths = {}  -- Table used as set: path -> true for selected items
    
    -- Drag drop state
    self.drag_target = nil  -- Current drop target folder path
    self.drag_target_entity = nil  -- Entity of current drop target (for unhighlighting)
    self.dragging_item = nil  -- Item being dragged
    self.drag_ghost = nil  -- Visual ghost following mouse during drag
    self.drag_folder_handled = false  -- Flag to prevent scroll/panel from overriding folder target
    self.panel_hovered = false  -- Track if mouse is over panel (for external drops)
    self.drop_overlay = nil  -- Drop overlay entity
    self.hover_folder = ""  -- Currently hovered folder (for external drop targeting)
    self.pending_external_drops = {}  -- Queue of external files to process in Update
    self.ctrl_held = false  -- Track Ctrl key state via keyboard events
    
    -- Pending move (for confirmation modal)
    self.pending_move = nil  -- { source = item, target_folder = path }
    
    -- Context menu
    self.context_menu_entity = nil
    self.context_menu_path = nil
    self.context_menu_handled = false  -- Flag to prevent event bubbling double-spawn
    
    -- Dirty flag for deferred rendering (prevents duplicate render_tree calls)
    self.needs_render = false
    
    -- Colors
    self.colors = {
        bg = {r = 0.12, g = 0.12, b = 0.14, a = 1.0},
        header_bg = {r = 0.15, g = 0.15, b = 0.18, a = 1.0},
        row_hover = {r = 0.2, g = 0.2, b = 0.25, a = 1.0},
        row_selected = {r = 0.25, g = 0.35, b = 0.5, a = 1.0},
        text = {r = 0.85, g = 0.85, b = 0.85, a = 1.0},
        text_dim = {r = 0.55, g = 0.55, b = 0.55, a = 1.0},
        folder = {r = 0.9, g = 0.75, b = 0.4, a = 1.0},
        file = {r = 0.6, g = 0.7, b = 0.8, a = 1.0},
        drop_zone = {r = 0.2, g = 0.4, b = 0.6, a = 0.3},
        context_bg = {r = 0.18, g = 0.18, b = 0.22, a = 0.98},
        context_hover = {r = 0.25, g = 0.25, b = 0.3, a = 1.0},
        danger = {r = 0.8, g = 0.3, b = 0.3, a = 1.0},
    }
    
    -- Rename dialog state
    self.rename_dialog = nil
    self.rename_path = nil
    self.rename_new_name = nil
    
    -- Internal drag state
    self.dragging_item = nil
    self.drag_preview = nil
    
    -- Register Update system for all event handling (needs world access)
    local browser = self  -- Capture self for closure
    register_system("Update", function(world)
        -- Main update for event handling
        browser:update(world)
        -- Deferred operations
        browser:process_pending_rename(world)
        browser:process_pending_create_file(world)
        browser:process_pending_create_folder(world)
        browser:process_pending_external_drops(world)
    end)
    
    return self
end

--- Show the file browser
--- @param left_offset Optional left offset in pixels (for sidebar integration)
--- @param parent_entity number|nil Optional parent entity ID for VR integration
--- @return panel_entity The panel container entity ID
function FileBrowser:show(left_offset, parent_entity)
    if self.is_visible then return self.panel_entity end
    
    self.left_offset = left_offset or 0
    self.parent_entity = parent_entity
    self:spawn_panel()
    self:load_folder("")  -- Load root
    
    self.is_visible = true
    return self.panel_entity
end

--- Hide the file browser
function FileBrowser:hide()
    if not self.is_visible then return end
    
    self:destroy_all()
    self.is_visible = false
end

--- Spawn the main panel
function FileBrowser:spawn_panel()
    -- Main panel container (always absolute, left_offset determines position)
    local left_val = self.left_offset or 0
    local node_config = {
        position_type = "Absolute",
        left = {Px = left_val},
        top = {Px = 0},
        bottom = {Px = 0},
        width = {Px = PANEL_WIDTH},
        flex_direction = "Column",
    }
    
    local panel = spawn({
        Node = node_config,
        BackgroundColor = { color = self.colors.bg },
        GlobalZIndex = { value = 100 },
        -- Component to identify this as a sidebar panel (for escape key handling)
        SidebarPanel = { script = "scripts/ui/file_browser.lua" },
    })
        :observe("Pointer<Over>", function(entity, event)
            self.panel_hovered = true
            -- When hovering over panel background (not a specific item), target root
            self.hover_folder = ""
        end)
        :observe("Pointer<Out>", function(entity, event)
            self.panel_hovered = false
        end)
        :observe("Pointer<DragOver>", function(entity, event)
            -- When dragging over panel (but not a folder), set root as target
            -- Only if folder didn't already handle it (prevents bubbling override)
            if self.dragging_items and not self.drag_folder_handled then
                -- Unhighlight previous folder target
                if self.drag_target_entity then
                    self.drag_target_entity:set({ BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0} } })
                    self.drag_target_entity = nil
                end
                self.drag_target = ""  -- Root directory
            end
            -- Reset flag at end of bubbling chain (panel is last)
            self.drag_folder_handled = false
        end)
    
    -- Apply parent if provided (for VR integration)
    if self.parent_entity then
        panel = panel:with_parent(self.parent_entity)
    end
    
    self.panel_entity = panel:id()
    table.insert(self.entities, self.panel_entity)
    
    -- Header (child of panel)
    local header = spawn({
        Node = {
            width = {Percent = 100},
            height = {Px = 36},
            flex_direction = "Row",
            align_items = "Center",
            justify_content = "SpaceBetween",
            padding = { left = {Px = 12}, right = {Px = 8} },
        },
        BackgroundColor = { color = self.colors.header_bg },
    }):with_parent(self.panel_entity):id()
    table.insert(self.entities, header)
    
    -- Header title (child of header)
    local title = spawn({
        Text = { text = "[Assets]" },
        TextFont = { font_size = 14 },
        TextColor = { color = self.colors.text },
    }):with_parent(header):id()
    table.insert(self.entities, title)
    
    -- Button row (right side of header)
    local button_row = spawn({
        Node = {
            flex_direction = "Row",
            column_gap = {Px = 4},
            align_items = "Center",
        },
    }):with_parent(header):id()
    table.insert(self.entities, button_row)
    
    -- Upload button
    local upload_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 26},
            height = {Px = 26},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = {r = 0.2, g = 0.2, b = 0.22, a = 1.0} },
        BorderRadius = { 
            top_left = {Px = 4}, top_right = {Px = 4}, 
            bottom_left = {Px = 4}, bottom_right = {Px = 4} 
        },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self:show_file_picker()
        end)
        :id()
    table.insert(self.entities, upload_btn)
    
    -- Upload icon (↑)
    spawn({
        Text = { text = "U" },
        TextFont = { font_size = 14 },
        TextColor = { color = self.colors.text_dim },
    }):with_parent(upload_btn)
    
    -- Refresh button (child of button_row)
    local refresh_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 26},
            height = {Px = 26},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = {r = 0.2, g = 0.2, b = 0.22, a = 1.0} },
        BorderRadius = { 
            top_left = {Px = 4}, top_right = {Px = 4}, 
            bottom_left = {Px = 4}, bottom_right = {Px = 4} 
        },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self:refresh()
        end)
        :id()
    table.insert(self.entities, refresh_btn)
    
    -- Refresh icon (child of button)
    local refresh_icon = spawn({
        Text = { text = "R" },
        TextFont = { font_size = 14 },
        TextColor = { color = self.colors.text_dim },
    }):with_parent(refresh_btn):id()
    table.insert(self.entities, refresh_icon)
    
    -- Scroll container (child of panel)
    -- Store scroll_offset for manual scrolling
    self.scroll_offset = 0
    
    self.scroll_container = spawn({
        Node = {
            display = "Flex",
            width = {Percent = 100},
            flex_grow = 1,
            flex_direction = "Column",
            align_items = "FlexStart",
            -- Bevy 0.17: overflow uses OverflowAxis for x/y, Scroll enables scrolling
            overflow = { x = "Visible", y = "Scroll" },
            padding = { top = {Px = 4}, bottom = {Px = 4} },
        },
        -- ScrollPosition controls the scroll offset
        ScrollPosition = { offset = {x = 0, y = 0} },
    })
        :with_parent(self.panel_entity)
        :observe("Pointer<Scroll>", function(entity, event)
            -- Handle scroll wheel events
            local scroll = event.event
            self.scroll_offset = self.scroll_offset - scroll.y * 2
            if self.scroll_offset < 0 then
                self.scroll_offset = 0
            end
            -- Update ScrollPosition
            entity:set({ ScrollPosition = { offset = {x = 0, y = self.scroll_offset} } })
        end)
        :observe("Pointer<DragOver>", function(entity, event)
            -- Scroll container as root drop target
            -- Only set root target if folder didn't already handle it (prevents bubbling override)
            if self.dragging_items and not self.drag_folder_handled then
                -- Unhighlight any folder target
                if self.drag_target_entity then
                    self.drag_target_entity:set({ BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0} } })
                    self.drag_target_entity = nil
                end
                self.drag_target = ""  -- Empty string = root directory
            end
            -- NOTE: Don't reset flag here - panel DragOver fires AFTER this due to bubbling
        end)
        :observe("Pointer<DragEnd>", function(entity, event)
            -- Handle ALL drops (both folder and root targets)
            
            -- Cleanup drag ghost
            if self.drag_ghost_id then
                despawn(self.drag_ghost_id)
                self.drag_ghost = nil
                self.drag_ghost_id = nil
            end
            
            -- Clear highlight on target
            if self.drag_target_entity then
                self.drag_target_entity:set({ BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0} } })
                self.drag_target_entity = nil
            end
            
            if self.dragging_items and #self.dragging_items > 0 then
                local target = self.drag_target
                
                if target then
                    -- Move all dragging items to target folder
                    for _, item in ipairs(self.dragging_items) do
                        -- Get parent folder from path
                        local source_folder = item.path:match("(.+)/[^/]+$") or ""
                        
                        -- Don't move if already in target folder
                        if target ~= source_folder then
                            local item_name = item.path:match("([^/]+)$") or item.path
                            local new_path = (target == "") and item_name or (target .. "/" .. item_name)
                            
                            -- Queue the move (use first item for confirmation, move all)
                            if not self.pending_moves then
                                self.pending_moves = {}
                            end
                            table.insert(self.pending_moves, { 
                                source_path = item.path, 
                                target_folder = target, 
                                new_path = new_path 
                            })
                        end
                    end
                    
                    -- Show confirmation if there are moves
                    if self.pending_moves and #self.pending_moves > 0 then
                        self:show_move_multiple_confirmation()
                    end
                end
            end
            -- Clear drag state
            self.dragging_items = nil
            self.drag_target = nil
        end)
        :observe("Pointer<Click>", function(entity, event)
            -- Right-click on scroll container for root context menu
            -- Only if row didn't already handle it (prevent bubble double-spawn)
            if self.context_menu_handled then
                self.context_menu_handled = false  -- Reset for next click
                return
            end
            local click = event.event
            local is_right_click = click and click.button and click.button.variant == "Secondary"
            if is_right_click then
                -- Get position from event.pointer_location.position (not click.pointer_location)
                local x = (event.pointer_location and event.pointer_location.position and event.pointer_location.position.x) or 100
                local y = (event.pointer_location and event.pointer_location.position and event.pointer_location.position.y) or 100
                -- Create a fake root item for context menu
                local root_item = { name = "", path = "", is_directory = true }
                self:show_context_menu(root_item, x, y)
            end
        end)
        :id()
    table.insert(self.entities, self.scroll_container)
end

--- Move an item locally (update folder lists immediately)
function FileBrowser:move_item_locally(item, target_folder)
    -- Find and remove from source folder
    local source_folder = item.path:match("(.+)/[^/]+$") or ""
    local source_state = self.folders[source_folder]
    if source_state and source_state.items then
        for i, existing in ipairs(source_state.items) do
            if existing.path == item.path then
                table.remove(source_state.items, i)
                break
            end
        end
    end
    
    -- Add to target folder
    local target_state = self.folders[target_folder]
    if target_state and target_state.items then
        local new_path = target_folder ~= "" and (target_folder .. "/" .. item.name) or item.name
        table.insert(target_state.items, {
            name = item.name,
            path = new_path,
            is_directory = item.is_directory,
            size = item.size or 0
        })
    end
    
    -- Defer re-render to avoid despawning entities during event handlers
    -- (The update loop will call render_tree next frame)
    self.needs_render = true
end

--- Load a folder's contents from server
function FileBrowser:load_folder(path, offset)
    offset = offset or 0
    
    -- Initialize folder state if needed
    if not self.folders[path] then
        self.folders[path] = {
            expanded = (path == ""),  -- Root always expanded
            loading = false,
            items = {},
            has_more = true,
            offset = 0,
        }
    end
    
    local folder = self.folders[path]
    if folder.loading then return end
    
    folder.loading = true
    folder.offset = offset
    
    -- Request directory listing from server
    local request_id = list_server_directory(path, offset, PAGE_SIZE)
    
    -- Store request for response matching
    folder.request_id = request_id
end

--- Handle directory listing response
function FileBrowser:on_directory_listing(event)
    local folder = self.folders[event.path]
    if not folder then return end
    
    folder.loading = false
    
    -- Option<String> is serialized as {None = true} or {Some = "error message"}
    if event.error and event.error.Some then
        print("Directory listing error: " .. tostring(event.error.Some))
        return
    end
    
    -- Append or replace items
    if event.offset == 0 then
        folder.items = {}
    end
    
    for _, file in ipairs(event.files) do
        table.insert(folder.items, file)
    end
    
    folder.has_more = event.has_more
    folder.total_count = event.total_count
    
    -- Mark for deferred rendering (prevents duplicate render_tree calls)
    self.needs_render = true
end

--- Render the folder tree
function FileBrowser:render_tree()
    -- Clear existing rows
    self:clear_tree()
    
    -- Recursively render from root
    self:render_folder("", 0)
end

--- Clear tree rows
function FileBrowser:clear_tree()
    -- Clear the list FIRST to prevent double-despawn if called multiple times
    local entities_to_despawn = self.row_entities or {}
    self.row_entities = {}
    
    -- Now despawn the entities
    for _, entity in ipairs(entities_to_despawn) do
        despawn(entity)
    end
end

--- Render a folder and its children
function FileBrowser:render_folder(path, depth)
    local folder = self.folders[path]
    if not folder then return end
    
    for _, item in ipairs(folder.items) do
        self:render_row(item, depth)
        
        -- If it's an expanded folder, render its children
        if item.is_directory then
            local child_folder = self.folders[item.path]
            if child_folder and child_folder.expanded then
                self:render_folder(item.path, depth + 1)
            end
        end
    end
    
    -- Show "Load more" if has_more
    if folder.has_more and not folder.loading then
        self:render_load_more(path, depth)
    end
end

--- Render a single row
function FileBrowser:render_row(item, depth)
    local indent = depth * INDENT_SIZE + 8
    local is_selected = self.selected_paths[item.path] ~= nil
    
    local row = spawn({
        Button = {},
        Node = {
            display = "Flex",
            width = {Percent = 100},
            height = {Px = ROW_HEIGHT},
            flex_direction = "Row",
            align_items = "Center",
            padding = { left = {Px = indent}, right = {Px = 8} },
        },
        BackgroundColor = { color = is_selected and self.colors.row_selected or {r = 0, g = 0, b = 0, a = 0} },
        -- Custom Lua component for querying rows in Update system
        LuaRowData = { path = item.path, is_directory = item.is_directory or false },
    })
        :with_parent(self.scroll_container)
        :observe("Pointer<Click>", function(entity, event)
            -- Check for right-click (Secondary button) via reflected event data
            local click = event.event
            local is_right_click = click and click.button and click.button.variant == "Secondary"
            
            -- Use tracked Ctrl state from keyboard events
            local ctrl_held = self.ctrl_held
            
            if is_right_click then
                -- Right-click: add to selection if not already selected, show context menu
                if not self.selected_paths[item.path] then
                    -- Clear selection and select only this item
                    self.selected_paths = {}
                    self.selected_paths[item.path] = true
                end
                -- Get position for context menu from event.pointer_location.position
                local x = 100
                local y = 100
                if event.pointer_location and event.pointer_location.position then
                    x = event.pointer_location.position.x or 100
                    y = event.pointer_location.position.y or 100
                end
                -- Set flag to prevent scroll container from also showing menu (event bubbles)
                self.context_menu_handled = true
                self:on_row_right_click(item, x, y)
            else
                self:on_row_click(item, ctrl_held)
            end
        end)
        :observe("Pointer<Over>", function(entity, event)
            -- Subtle hover highlight (only if not selected)
            if not self.selected_paths[item.path] then
                entity:set({ BackgroundColor = { color = {r = 0.18, g = 0.18, b = 0.22, a = 1.0} } })
            end
            -- Track hovered item for external drop targeting
            if item.is_directory then
                self.hover_folder = item.path
            else
                -- Non-directory: use parent folder
                self.hover_folder = item.path:match("(.+)/[^/]+$") or ""
            end
        end)
        :observe("Pointer<Out>", function(entity, event)
            -- Restore background (only if not selected and not drag target)
            if not self.selected_paths[item.path] and self.drag_target ~= item.path then
                entity:set({ BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0} } })
            end
        end)
        :observe("Pointer<DragStart>", function(entity, event)
            -- If dragging a selected item, drag ALL selected items
            -- Otherwise, drag just this item
            if self.selected_paths[item.path] then
                -- Collect all selected items
                self.dragging_items = {}
                for path, _ in pairs(self.selected_paths) do
                    table.insert(self.dragging_items, { path = path })
                end
            else
                -- Dragging unselected item - just drag this one
                self.dragging_items = { item }
            end
            
            -- Create visual ghost that follows mouse
            local start_x = 100
            local start_y = 100
            if event.pointer_location and event.pointer_location.position then
                start_x = event.pointer_location.position.x or 100
                start_y = event.pointer_location.position.y or 100
            end
            
            -- Ghost text shows count if multiple items
            local ghost_text = item.name
            if #self.dragging_items > 1 then
                ghost_text = "Moving " .. #self.dragging_items .. " items"
            end
            
            -- Store both entity object (for :set()) and ID (for despawn/with_parent)
            local ghost_entity = spawn({
                Node = {
                    position_type = "Absolute",
                    left = {Px = start_x + 10},
                    top = {Px = start_y},
                    padding = { left = {Px = 8}, right = {Px = 8}, top = {Px = 4}, bottom = {Px = 4} },
                },
                BackgroundColor = { color = {r = 0.25, g = 0.35, b = 0.5, a = 0.9} },
                BorderRadius = { top_left = {Px = 4}, top_right = {Px = 4}, bottom_left = {Px = 4}, bottom_right = {Px = 4} },
                GlobalZIndex = { value = 600 },
            })
            self.drag_ghost = ghost_entity  -- Store entity object for :set()
            self.drag_ghost_id = ghost_entity:id()  -- Store ID for despawn/with_parent
            
            spawn({
                Text = { text = ghost_text },
                TextFont = { font_size = 12 },
                TextColor = { color = {r = 1, g = 1, b = 1, a = 1} },
            }):with_parent(self.drag_ghost_id)
        end)
        :observe("Pointer<Drag>", function(entity, event)
            -- Update ghost position to follow mouse
            if self.drag_ghost and event.pointer_location and event.pointer_location.position then
                local x = (event.pointer_location.position.x or 100) + 10
                local y = event.pointer_location.position.y or 100
                self.drag_ghost:set({
                    Node = { left = {Px = x}, top = {Px = y} }
                })
            end
        end)
        :observe("Pointer<DragOver>", function(entity, event)
            -- Handle as drop target if dragging and not the same item
            if self.dragging_items and #self.dragging_items > 0 then
                local target_folder
                if item.is_directory then
                    -- Directory: drop into this folder
                    target_folder = item.path
                else
                    -- Non-directory: drop into this file's parent folder
                    target_folder = item.path:match("(.+)/[^/]+$") or ""
                end
                
                -- Only update if target changed
                if self.drag_target ~= target_folder then
                    -- Unhighlight previous target
                    if self.drag_target_entity then
                        self.drag_target_entity:set({ BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0} } })
                    end
                    
                    -- Highlight new target (only for directories)
                    self.drag_target = target_folder
                    if item.is_directory then
                        self.drag_target_entity = entity
                        entity:set({ BackgroundColor = { color = {r = 0.2, g = 0.3, b = 0.4, a = 1.0} } })
                    else
                        self.drag_target_entity = nil  -- Don't highlight files
                    end
                end
                self.drag_folder_handled = true  -- Prevent scroll/panel from overriding with root
            end
        end)
        -- Note: DragEnd is handled by scroll_container for all drops
        :id()
    table.insert(self.row_entities, row)
    
    -- Expand arrow for directories
    if item.is_directory then
        local folder = self.folders[item.path]
        local is_expanded = folder and folder.expanded
        
        local arrow = spawn({
            Button = {},
            Node = {
                width = {Px = 16},
                height = {Px = 16},
                justify_content = "Center",
                align_items = "Center",
                margin = { right = {Px = 4} },
            },
        })
            :with_parent(row)
            :observe("Pointer<Click>", function(entity, event)
                self:toggle_folder(item.path)
            end)
            :id()
        -- Don't track child arrow - will be despawned with parent row
        
        local arrow_text = spawn({
            Text = { text = is_expanded and "▼" or "▶" },
            TextFont = { font_size = 10 },
            TextColor = { color = self.colors.text_dim },
        }):with_parent(arrow):id()
        -- Don't track child text - will be despawned with parent row
    else
        -- Spacer for files
        local spacer = spawn({
            Node = { width = {Px = 20}, height = {Px = 1} },
        }):with_parent(row):id()
        -- Don't track spacer child - will be despawned with parent row
    end
    
    -- Icon (placeholder - will be image later)
    local icon_text = item.is_directory and "+" or "-"
    local icon = spawn({
        Text = { text = icon_text },
        TextFont = { font_size = 14 },
        TextColor = { color = item.is_directory and self.colors.folder or self.colors.file },
        Node = { margin = { right = {Px = 6} } },
    }):with_parent(row):id()
    -- Don't track icon child - will be despawned with parent row
    
    -- Name container with clipping for long names
    -- min_width = 0 is required for overflow clipping to work with flex_shrink
    local name_container = spawn({
        Node = {
            flex_grow = 1,
            flex_shrink = 1,
            min_width = {Px = 0},  -- Required for overflow clipping in flex
            -- Bevy 0.17: overflow uses separate x/y axes
            overflow = { x = "Clip", y = "Visible" },
            margin = { right = {Px = 8} },
        },
    }):with_parent(row):id()
    
    spawn({
        Text = { text = item.name },
        TextFont = { font_size = 12 },
        TextColor = { color = self.colors.text },
    }):with_parent(name_container)
    -- Don't track name child - will be despawned with parent row
    
    -- Size for files (flex_shrink = 0 to always show)
    if not item.is_directory and item.size and item.size > 0 then
        local size_text = self:format_size(item.size)
        spawn({
            Text = { text = size_text },
            TextFont = { font_size = 11 },
            TextColor = { color = self.colors.text_dim },
            Node = { flex_shrink = 0 },
        }):with_parent(row)
        -- Don't track size child - will be despawned with parent row
    end
end

--- Render "Load more" button
function FileBrowser:render_load_more(path, depth)
    local indent = depth * INDENT_SIZE + 28
    
    local btn = spawn({
        Button = {},
        Node = {
            display = "Flex",
            width = {Percent = 100},
            height = {Px = ROW_HEIGHT},
            flex_direction = "Row",
            align_items = "Center",
            justify_content = "Center",
            padding = { left = {Px = indent} },
        },
    })
        :with_parent(self.scroll_container)
        :observe("Pointer<Click>", function(entity, event)
            local folder = self.folders[path]
            if folder then
                self:load_folder(path, folder.offset + PAGE_SIZE)
            end
        end)
        :id()
    table.insert(self.row_entities, btn)
    
    local text = spawn({
        Text = { text = "Load more..." },
        TextFont = { font_size = 12 },
        TextColor = { color = {r = 0.5, g = 0.7, b = 0.9, a = 1.0} },
    }):with_parent(btn):id()
    -- Don't track load_more text child - will be despawned with parent btn
end

--- Handle row click
function FileBrowser:on_row_click(item, ctrl_held)
    if ctrl_held then
        -- Ctrl+click: toggle item in selection
        if self.selected_paths[item.path] then
            self.selected_paths[item.path] = nil
        else
            self.selected_paths[item.path] = true
        end
        -- Don't toggle folder on Ctrl+click, just add/remove from selection
        self.needs_render = true
    else
        -- Regular click: clear selection and select only this item
        self.selected_paths = {}
        self.selected_paths[item.path] = true
        
        if item.is_directory then
            -- toggle_folder sets needs_render flag
            self:toggle_folder(item.path)
        else
            -- For files, mark for re-render to update selection highlighting
            self.needs_render = true
        end
    end
end

--- Toggle folder expansion
function FileBrowser:toggle_folder(path)
    local folder = self.folders[path]
    
    if not folder then
        -- First time expanding - load contents
        self:load_folder(path)
        self.folders[path].expanded = true
    else
        folder.expanded = not folder.expanded
        
        if folder.expanded and #folder.items == 0 then
            self:load_folder(path)
        end
    end
    
    -- Mark for deferred rendering
    self.needs_render = true
end

--- Refresh the file browser
function FileBrowser:refresh()
    -- Clear and reload all expanded folders
    for path, folder in pairs(self.folders) do
        if folder.expanded or path == "" then
            folder.items = {}
            folder.has_more = true
            self:load_folder(path, 0)
        end
    end
end

--- Update (called each frame)
function FileBrowser:update(world)
    if not self.is_visible then return end
    
    -- Track keyboard modifier state via keyboard events
    local keyboard_events = world:read_events("bevy_input::keyboard::KeyboardInput")
    for _, e in ipairs(keyboard_events) do
        -- Check for Ctrl key (ControlLeft or ControlRight)
        local key = e.key_code
        if key and (key.ControlLeft or key.ControlRight) then
            local state = e.state
            if state and state.Pressed then
                self.ctrl_held = true
            elseif state and state.Released then
                self.ctrl_held = false
            end
        end
    end
    
    -- Process directory listing events
    local events = world:read_events("hello::asset_events::AssetDirectoryListingEvent")
    for _, e in ipairs(events) do
        self:on_directory_listing(e)
    end
    
    -- Handle FileDragAndDrop events
    local drop_events = world:read_events("bevy_window::event::FileDragAndDrop")
    for _, e in ipairs(drop_events) do
        if e.DroppedFile then
            -- Process drop (always for external files - they go to root/selected folder)
            self:on_file_drop(e)
            self:hide_drop_overlay()
        elseif e.HoveredFile then
            -- Always show overlay for external file drags
            -- (Pointer<Over> doesn't work for OS file drags, so we can't check panel_hovered)
            self:show_drop_overlay()
        elseif e.HoveredFileCanceled then
            -- Note: Bevy 0.11+ uses "HoveredFileCanceled" (American spelling with one 'l')
            self:hide_drop_overlay()
        end
    end
    
    -- Deferred rendering - only render once per frame, after all events processed
    if self.needs_render then
        self.needs_render = false
        self:render_tree()
    end
end

--- Show drop overlay for external file drag
function FileBrowser:show_drop_overlay()
    if self.drop_overlay then return end  -- Already showing
    
    self.drop_overlay = spawn({
        Node = {
            position_type = "Absolute",
            left = {Px = 0}, right = {Px = 0},
            top = {Px = 0}, bottom = {Px = 0},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0.7} },
        GlobalZIndex = { value = 500 },
        -- Allow pointer events to pass through to elements below
        -- so hover_folder tracking still works during external drags
        PickingBehavior = { should_block_lower = false },
    }):with_parent(self.panel_entity):id()
    
    spawn({
        Text = { text = "Drop files here" },
        TextFont = { font_size = 24 },
        TextColor = { color = {r = 1, g = 1, b = 1, a = 0.9} },
    }):with_parent(self.drop_overlay)
end

--- Hide drop overlay
function FileBrowser:hide_drop_overlay()
    if self.drop_overlay then
        despawn(self.drop_overlay)
        self.drop_overlay = nil
    end
end

--- Handle file drop
function FileBrowser:on_file_drop(event)
    -- FileDragAndDrop can be DroppedFile, HoveredFile, or HoveredFileCancelled
    -- We handle DroppedFile
    if event.DroppedFile then
        local source_path = event.DroppedFile.path_buf
        
        -- Queue the drop for processing in Update system
        -- where we can query for the currently hovered folder via Interaction component
        local source_path_clean = source_path:gsub('^"', ''):gsub('"$', '')
        table.insert(self.pending_external_drops, {
            source_path = source_path_clean,
        })
    end
end

--- Process pending external drops (called from Update system with world access)
function FileBrowser:process_pending_external_drops(world)
    if #self.pending_external_drops == 0 then return end
    
    -- Query for hovered folder row using Interaction component
    -- First, find all rows with Interaction = Hovered
    local target_folder = ""
    
    -- Query entities with our row marker and Interaction component
    local rows = world:query({"LuaRowData", "Interaction"}, nil)
    for _, row_entity in ipairs(rows) do
        local interaction = row_entity:get("Interaction")
        local row_data = row_entity:get("LuaRowData")
        -- Check for Hovered state (table format: {Hovered = true} or {None = true})
        if interaction and interaction.Hovered then
            if row_data then
                if row_data.is_directory then
                    target_folder = row_data.path or ""
                else
                    -- Non-directory: use parent folder
                    target_folder = (row_data.path or ""):match("(.+)/[^/]+$") or ""
                end
                break
            end
        end
    end
    
    -- Process all queued drops
    for _, drop in ipairs(self.pending_external_drops) do
        local source_path = drop.source_path
        local filename = source_path:match("([^/\\]+)$") or "DroppedFile"
        
        -- Build destination path in assets folder
        local relative_dest = target_folder ~= "" and (target_folder .. "/" .. filename) or filename
        local full_dest = "assets/" .. relative_dest
        
        print("Copying external file: " .. source_path .. " -> " .. full_dest)
        
        -- Copy file to assets folder
        local ok, err = pcall(function()
            copy_file(source_path, full_dest)
        end)
        
        if ok then
            print("File copied successfully: " .. relative_dest)
            
            -- Add to local folder list immediately (only if not already present)
            local folder_state = self.folders[target_folder]
            if folder_state and folder_state.items then
                -- Check if file already exists to prevent duplicates
                local already_exists = false
                for _, existing in ipairs(folder_state.items) do
                    if existing.path == relative_dest then
                        already_exists = true
                        break
                    end
                end
                if not already_exists then
                    table.insert(folder_state.items, {
                        name = filename,
                        path = relative_dest,
                        is_directory = false,
                        size = 0  -- Unknown size for now
                    })
                end
            end
            
            -- Use deferred rendering to avoid despawn issues
            self.needs_render = true
            
            -- Also upload to server if in network mode
            if upload_asset then
                upload_asset(relative_dest, relative_dest, true)
            end
        else
            print("Failed to copy file: " .. tostring(err))
        end
    end
    
    -- Clear the queue
    self.pending_external_drops = {}
end

--- Format file size
function FileBrowser:format_size(bytes)
    if bytes < 1024 then
        return bytes .. " B"
    elseif bytes < 1024 * 1024 then
        return string.format("%.1f KB", bytes / 1024)
    else
        return string.format("%.1f MB", bytes / (1024 * 1024))
    end
end

--- Destroy all UI
function FileBrowser:destroy_all()
    self:clear_tree()
    self:close_context_menu()
    self:close_rename_dialog()
    
    for _, entity in ipairs(self.entities) do
        despawn(entity)
    end
    
    self.entities = {}
    self.panel_entity = nil
    self.scroll_container = nil
end

-- =============================================================================
-- Context Menu
-- =============================================================================

--- Show context menu at position
--- item can be nil for root context menu (e.g., right-click on empty area)
function FileBrowser:show_context_menu(item, x, y)
    self:close_context_menu()
    
    self.context_menu_item = item
    local menu_width = 160
    
    -- Invisible backdrop to catch click-outside
    self.context_menu_backdrop = spawn({
        Button = {},
        Node = {
            position_type = "Absolute",
            left = {Px = 0}, right = {Px = 0},
            top = {Px = 0}, bottom = {Px = 0},
        },
        BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0.01} },  -- Nearly invisible
        GlobalZIndex = { value = 499 },
    })
        :observe("Pointer<Click>", function(entity, event)
            self:close_context_menu()
        end)
        :id()
    
    -- Context menu container (with Button to block events from reaching backdrop)
    self.context_menu_entity = spawn({
        Button = {},  -- Makes this pickable and blocks events from reaching backdrop
        Node = {
            position_type = "Absolute",
            left = {Px = x},
            top = {Px = y},
            width = {Px = menu_width},
            flex_direction = "Column",
            padding = { top = {Px = 4}, bottom = {Px = 4} },
        },
        BackgroundColor = { color = self.colors.context_bg },
        BorderRadius = { 
            top_left = {Px = 6}, top_right = {Px = 6}, 
            bottom_left = {Px = 6}, bottom_right = {Px = 6} 
        },
        GlobalZIndex = { value = 500 },
    }):id()
    
    -- Determine target folder for new file
    local target_folder = ""
    if item then
        if item.is_directory then
            target_folder = item.path
        else
            -- Get parent folder from file path
            target_folder = item.path:match("(.*/)")
            if target_folder then
                target_folder = target_folder:sub(1, -2)  -- Remove trailing slash
            else
                target_folder = ""  -- Root folder
            end
        end
    end
    
    -- New File option (always available)
    self:add_context_menu_item("+ New File", function()
        self:close_context_menu()
        self:show_new_file_dialog(target_folder)
    end)
    
    -- New Folder option (always available)
    self:add_context_menu_item("+ New Folder", function()
        self:close_context_menu()
        self:show_new_folder_dialog(target_folder)
    end)
    
    -- Only show rename/delete if we have an item with a real path (not root)
    if item and item.path and item.path ~= "" then
        -- Count selected items
        local selected_count = 0
        for _ in pairs(self.selected_paths) do
            selected_count = selected_count + 1
        end
        
        -- Rename option - only show for single selection
        if selected_count <= 1 then
            self:add_context_menu_item("Rename", function()
                self:close_context_menu()
                self:show_rename_dialog(item)
            end)
        end
        
        -- Delete option - deletes all selected items
        local delete_text = selected_count > 1 and ("Delete " .. selected_count .. " items") or "Delete"
        self:add_context_menu_item(delete_text, function()
            self:close_context_menu()
            if selected_count > 1 then
                -- Delete all selected items
                self:show_delete_multiple_confirmation()
            else
                self:show_delete_confirmation(item)
            end
        end, true)  -- is_danger
    end
end

--- Add item to context menu
function FileBrowser:add_context_menu_item(text, on_click, is_danger)
    local text_color = is_danger and self.colors.danger or self.colors.text
    
    local item = spawn({
        Button = {},
        Node = {
            width = {Percent = 100},
            height = {Px = 28},
            flex_direction = "Row",
            align_items = "Center",
            padding = { left = {Px = 12}, right = {Px = 12} },
        },
    })
        :with_parent(self.context_menu_entity)
        :observe("Pointer<Click>", function(entity, event)
            on_click()
        end)
        :id()
    
    local label = spawn({
        Text = { text = text },
        TextFont = { font_size = 12 },
        TextColor = { color = text_color },
    }):with_parent(item):id()
end

--- Close context menu
function FileBrowser:close_context_menu()
    if self.context_menu_backdrop then
        despawn(self.context_menu_backdrop)
        self.context_menu_backdrop = nil
    end
    if self.context_menu_entity then
        despawn(self.context_menu_entity)
        self.context_menu_entity = nil
    end
    self.context_menu_item = nil
end

-- =============================================================================
-- Rename Dialog
-- =============================================================================

--- Show rename dialog
function FileBrowser:show_rename_dialog(item)
    self:close_rename_dialog()
    
    self.rename_path = item.path
    self.rename_new_name = item.name
    
    local dialog_width = 300
    local dialog_height = 140
    
    -- Dialog backdrop (click to close)
    local backdrop = spawn({
        Button = {},
        Node = {
            position_type = "Absolute",
            left = {Px = 0}, right = {Px = 0},
            top = {Px = 0}, bottom = {Px = 0},
        },
        BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0.5} },
        GlobalZIndex = { value = 600 },
    })
        :observe("Pointer<Click>", function(entity, event)
            self:close_rename_dialog()
        end)
        :id()
    
    -- Dialog container (NOT a child of backdrop to prevent click bubbling)
    self.rename_dialog = spawn({
        Node = {
            position_type = "Absolute",
            left = {Percent = 50},
            top = {Percent = 50},
            width = {Px = dialog_width},
            height = {Px = dialog_height},
            margin = { left = {Px = -dialog_width/2}, top = {Px = -dialog_height/2} },
            flex_direction = "Column",
            padding = { left = {Px = 16}, right = {Px = 16}, top = {Px = 12}, bottom = {Px = 12} },
        },
        BackgroundColor = { color = self.colors.context_bg },
        BorderRadius = { 
            top_left = {Px = 8}, top_right = {Px = 8}, 
            bottom_left = {Px = 8}, bottom_right = {Px = 8} 
        },
        GlobalZIndex = { value = 601 },
    }):id()  -- No with_parent - independent of backdrop
    
    -- Title
    spawn({
        Text = { text = "Rename" },
        TextFont = { font_size = 14 },
        TextColor = { color = self.colors.text },
        Node = { margin = { bottom = {Px = 12} } },
    }):with_parent(self.rename_dialog)
    
    -- Current name (for reference)
    spawn({
        Text = { text = "Current: " .. item.name },
        TextFont = { font_size = 11 },
        TextColor = { color = self.colors.text_dim },
        Node = { margin = { bottom = {Px = 8} } },
    }):with_parent(self.rename_dialog)
    
    -- Text input field with initial value pre-filled
    self.rename_item = item  -- Store for do_rename
    self.rename_input_entity = spawn({
        LuaTextInput = { 
            initial_value = item.name,  -- Pre-fill with current name
            auto_focus = true 
        },
        Node = {
            width = {Percent = 100},
            height = {Px = 28},
            padding = { left = {Px = 8}, right = {Px = 8} },
            align_items = "Center",
            margin = { bottom = {Px = 12} },
        },
        BackgroundColor = { color = {r = 0.1, g = 0.1, b = 0.12, a = 1.0} },
        BorderRadius = { 
            top_left = {Px = 4}, top_right = {Px = 4}, 
            bottom_left = {Px = 4}, bottom_right = {Px = 4} 
        },
        TextColor = { color = self.colors.text },
        TextFont = { font_size = 12 },
    }):with_parent(self.rename_dialog):id()
    
    -- Button row
    local btn_row = spawn({
        Node = {
            width = {Percent = 100},
            flex_direction = "Row",
            justify_content = "FlexEnd",
        },
    }):with_parent(self.rename_dialog):id()
    
    -- Cancel button
    local cancel_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 70},
            height = {Px = 28},
            justify_content = "Center",
            align_items = "Center",
            margin = { right = {Px = 8} },
        },
        BackgroundColor = { color = {r = 0.25, g = 0.25, b = 0.28, a = 1.0} },
        BorderRadius = { 
            top_left = {Px = 4}, top_right = {Px = 4}, 
            bottom_left = {Px = 4}, bottom_right = {Px = 4} 
        },
    })
        :with_parent(btn_row)
        :observe("Pointer<Click>", function(entity, event)
            self:close_rename_dialog()
        end)
        :id()
    
    spawn({
        Text = { text = "Cancel" },
        TextFont = { font_size = 12 },
        TextColor = { color = self.colors.text },
    }):with_parent(cancel_btn)
    
    -- Rename button - gets text from TextInputBuffer
    local rename_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 70},
            height = {Px = 28},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = {r = 0.2, g = 0.45, b = 0.7, a = 1.0} },
        BorderRadius = { 
            top_left = {Px = 4}, top_right = {Px = 4}, 
            bottom_left = {Px = 4}, bottom_right = {Px = 4} 
        },
    })
        :with_parent(btn_row)
        :observe("Pointer<Click>", function(entity, event)
            self:do_rename(item)
        end)
        :id()
    
    spawn({
        Text = { text = "Rename" },
        TextFont = { font_size = 12 },
        TextColor = { color = self.colors.text },
    }):with_parent(rename_btn)
    
    -- Store backdrop for cleanup
    self.rename_backdrop = backdrop
end

--- Close rename dialog
function FileBrowser:close_rename_dialog()
    if self.rename_backdrop then
        despawn(self.rename_backdrop)
        self.rename_backdrop = nil
    end
    if self.rename_dialog then
        despawn(self.rename_dialog)
        self.rename_dialog = nil
    end
    self.rename_path = nil
    self.rename_input_entity = nil
end

--- Execute the rename operation (schedules rename, called from observer callback)
function FileBrowser:do_rename(item)
    -- Set pending rename flag - actual rename happens in update system where world is available
    -- DON'T close dialog yet - we need the entity to still exist so we can read its value!
    self.pending_rename = {
        item = item,
        input_entity = self.rename_input_entity
    }
    -- Dialog closes in process_pending_rename AFTER reading the text
end

--- Process pending rename (called from update system with world access)
function FileBrowser:process_pending_rename(world)
    if not self.pending_rename then return end
    
    local pending = self.pending_rename
    self.pending_rename = nil
    
    local new_name = nil
    
    -- Read text from the input entity's LuaTextInputValue component
    -- Just get the first one - there should only be one active text input at a time
    local entities = world:query({"LuaTextInputValue"}, nil)
    if #entities > 0 then
        local entity = entities[1]
        local value = entity:get("LuaTextInputValue")
        if value then
            if value.text then
                new_name = value.text
            end
        end
    end
    
    -- Close dialog AFTER reading the text (so entity still exists for query)
    self:close_rename_dialog()
    
    if new_name and new_name ~= "" and new_name ~= pending.item.name then
        print("Renaming " .. pending.item.path .. " to " .. new_name)
        rename_asset(pending.item.path, new_name)
        
        -- Update local folder list immediately (avoid waiting for server refresh)
        -- Find the parent folder of the item
        local parent_folder = pending.item.path:match("(.+)/[^/]+$") or ""
        local folder_state = self.folders[parent_folder]
        if folder_state and folder_state.items then
            -- Find and update the item in the list
            for i, item in ipairs(folder_state.items) do
                if item.path == pending.item.path then
                    -- Update name and path with new name
                    local new_path = parent_folder ~= "" and (parent_folder .. "/" .. new_name) or new_name
                    item.name = new_name
                    item.path = new_path
                    break
                end
            end
        end
        
        -- Re-render the tree immediately
        self:render_tree()
    else
        print("Rename cancelled or no change (new_name: '" .. tostring(new_name) .. "', original: '" .. pending.item.name .. "')")
    end
end

-- =============================================================================
-- Delete Confirmation Dialog
-- =============================================================================

--- Show delete confirmation dialog
function FileBrowser:show_delete_confirmation(item)
    self:close_delete_dialog()
    
    self.delete_item = item
    local dialog_width = 300
    
    -- Dialog backdrop
    self.delete_backdrop = spawn({
        Button = {},
        Node = {
            position_type = "Absolute",
            left = {Px = 0}, right = {Px = 0},
            top = {Px = 0}, bottom = {Px = 0},
        },
        BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0.5} },
        GlobalZIndex = { value = 700 },
    })
        :observe("Pointer<Click>", function(entity, event)
            self:close_delete_dialog()
        end)
        :id()
    
    -- Dialog container (NOT a child of backdrop to prevent click bubbling)
    self.delete_dialog = spawn({
        Node = {
            position_type = "Absolute",
            left = {Percent = 50},
            top = {Percent = 50},
            width = {Px = dialog_width},
            margin = { left = {Px = -dialog_width/2}, top = {Px = -75} },
            flex_direction = "Column",
            padding = { top = {Px = 16}, bottom = {Px = 16}, left = {Px = 16}, right = {Px = 16} },
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.bg },
        BorderRadius = { 
            top_left = {Px = 8}, top_right = {Px = 8}, 
            bottom_left = {Px = 8}, bottom_right = {Px = 8} 
        },
        GlobalZIndex = { value = 701 },
    }):id()
    
    local dialog = self.delete_dialog
    
    -- Title
    spawn({
        Text = { text = "Delete " .. (item.is_directory and "Folder" or "File") .. "?" },
        TextFont = { font_size = 16 },
        TextColor = { color = self.colors.text },
    }):with_parent(dialog)
    
    -- Item name
    spawn({
        Text = { text = item.name },
        TextFont = { font_size = 12 },
        TextColor = { color = self.colors.text_dim },
        Node = { margin = { top = {Px = 8}, bottom = {Px = 16} } },
    }):with_parent(dialog)
    
    -- Button row
    local button_row = spawn({
        Node = {
            flex_direction = "Row",
            justify_content = "Center",
            column_gap = {Px = 12},
        },
    }):with_parent(dialog):id()
    
    -- Cancel button
    local cancel_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 80},
            height = {Px = 32},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.row_hover },
        BorderRadius = { top_left = {Px = 4}, top_right = {Px = 4}, bottom_left = {Px = 4}, bottom_right = {Px = 4} },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self:close_delete_dialog()
        end)
        :id()
    
    spawn({
        Text = { text = "Cancel" },
        TextFont = { font_size = 12 },
        TextColor = { color = self.colors.text },
    }):with_parent(cancel_btn)
    
    -- Delete button
    local delete_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 80},
            height = {Px = 32},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.danger },
        BorderRadius = { top_left = {Px = 4}, top_right = {Px = 4}, bottom_left = {Px = 4}, bottom_right = {Px = 4} },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self:do_delete(item)
        end)
        :id()
    
    spawn({
        Text = { text = "Delete" },
        TextFont = { font_size = 12 },
        TextColor = { color = {r = 1, g = 1, b = 1, a = 1} },
    }):with_parent(delete_btn)
end

--- Close delete dialog
function FileBrowser:close_delete_dialog()
    if self.delete_backdrop then
        despawn(self.delete_backdrop)
        self.delete_backdrop = nil
    end
    if self.delete_dialog then
        despawn(self.delete_dialog)
        self.delete_dialog = nil
    end
    self.delete_item = nil
end

--- Actually delete the item
function FileBrowser:do_delete(item)
    self:close_delete_dialog()
    
    -- CRITICAL SAFETY CHECK: Never delete empty path (root directory)!
    if not item.path or item.path == "" then
        print("ERROR: Attempted to delete root directory - operation blocked!")
        return
    end
    
    print("Deleting: " .. item.path)
    delete_asset(item.path)
    
    -- Remove item from local folder list immediately for instant UI update
    -- Find the parent folder
    local parent_path = item.path:match("(.*/)")
    if parent_path then
        parent_path = parent_path:sub(1, -2)  -- Remove trailing slash
    else
        parent_path = ""  -- Root folder
    end
    
    local folder = self.folders[parent_path]
    if folder and folder.items then
        for i, existing_item in ipairs(folder.items) do
            if existing_item.path == item.path then
                table.remove(folder.items, i)
                break
            end
        end
    end
    
    -- Mark for immediate re-render
    self.needs_render = true
end

--- Show delete confirmation for multiple items
function FileBrowser:show_delete_multiple_confirmation()
    self:close_delete_dialog()
    
    -- Count selected items
    local count = 0
    for _ in pairs(self.selected_paths) do
        count = count + 1
    end
    
    if count == 0 then return end
    
    local dialog_width = 300
    
    -- Dialog backdrop
    self.delete_backdrop = spawn({
        Button = {},
        Node = {
            position_type = "Absolute",
            left = {Px = 0}, right = {Px = 0},
            top = {Px = 0}, bottom = {Px = 0},
        },
        BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0.5} },
        GlobalZIndex = { value = 700 },
    })
        :observe("Pointer<Click>", function(entity, event)
            self:close_delete_dialog()
        end)
        :id()
    
    -- Dialog container
    self.delete_dialog = spawn({
        Node = {
            position_type = "Absolute",
            left = {Percent = 50},
            top = {Percent = 50},
            width = {Px = dialog_width},
            margin = { left = {Px = -dialog_width/2}, top = {Px = -75} },
            flex_direction = "Column",
            padding = { top = {Px = 16}, bottom = {Px = 16}, left = {Px = 16}, right = {Px = 16} },
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.bg },
        BorderRadius = { 
            top_left = {Px = 8}, top_right = {Px = 8}, 
            bottom_left = {Px = 8}, bottom_right = {Px = 8} 
        },
        GlobalZIndex = { value = 701 },
    }):id()
    
    local dialog = self.delete_dialog
    
    -- Title
    spawn({
        Text = { text = "Delete " .. count .. " items?" },
        TextFont = { font_size = 16 },
        TextColor = { color = self.colors.text },
    }):with_parent(dialog)
    
    -- Warning text
    spawn({
        Text = { text = "This action cannot be undone." },
        TextFont = { font_size = 12 },
        TextColor = { color = self.colors.danger },
        Node = { margin = { top = {Px = 8}, bottom = {Px = 16} } },
    }):with_parent(dialog)
    
    -- Button row
    local button_row = spawn({
        Node = {
            flex_direction = "Row",
            justify_content = "Center",
            column_gap = {Px = 12},
        },
    }):with_parent(dialog):id()
    
    -- Cancel button
    local cancel_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 80},
            height = {Px = 32},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.row_hover },
        BorderRadius = { top_left = {Px = 4}, top_right = {Px = 4}, bottom_left = {Px = 4}, bottom_right = {Px = 4} },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self:close_delete_dialog()
        end)
        :id()
    
    spawn({
        Text = { text = "Cancel" },
        TextFont = { font_size = 12 },
        TextColor = { color = self.colors.text },
    }):with_parent(cancel_btn)
    
    -- Delete button
    local delete_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 80},
            height = {Px = 32},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.danger },
        BorderRadius = { top_left = {Px = 4}, top_right = {Px = 4}, bottom_left = {Px = 4}, bottom_right = {Px = 4} },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self:do_delete_multiple()
        end)
        :id()
    
    spawn({
        Text = { text = "Delete" },
        TextFont = { font_size = 12 },
        TextColor = { color = {r = 1, g = 1, b = 1, a = 1} },
    }):with_parent(delete_btn)
end

--- Delete all selected items
function FileBrowser:do_delete_multiple()
    self:close_delete_dialog()
    
    for path, _ in pairs(self.selected_paths) do
        -- CRITICAL SAFETY CHECK: Never delete empty path (root directory)!
        if path and path ~= "" then
            print("Deleting: " .. path)
            delete_asset(path)
            
            -- Remove item from local folder list
            local parent_path = path:match("(.*/)")
            if parent_path then
                parent_path = parent_path:sub(1, -2)
            else
                parent_path = ""
            end
            
            local folder = self.folders[parent_path]
            if folder and folder.items then
                for i, existing_item in ipairs(folder.items) do
                    if existing_item.path == path then
                        table.remove(folder.items, i)
                        break
                    end
                end
            end
        end
    end
    
    -- Clear selection and re-render
    self.selected_paths = {}
    self.needs_render = true
end

-- =============================================================================
-- Move Confirmation Dialog
-- =============================================================================

--- Show move confirmation dialog
function FileBrowser:show_move_confirmation()
    self:close_move_dialog()
    
    local move = self.pending_move
    if not move then 
        print("ERROR: No pending move!")
        return 
    end
    
    local dialog_width = 320
    
    -- Dialog backdrop (full screen semi-transparent overlay)
    self.move_backdrop = spawn({
        Button = {},
        Node = {
            position_type = "Absolute",
            left = {Px = 0}, right = {Px = 0},
            top = {Px = 0}, bottom = {Px = 0},
            width = {Percent = 100},
            height = {Percent = 100},
        },
        BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0.5} },
        GlobalZIndex = { value = 900 },
    })
        :observe("Pointer<Click>", function(entity, event)
            self.pending_move = nil  -- Clear on cancel
            self:close_move_dialog()
        end)
        :id()
    
    -- Dialog container
    self.move_dialog = spawn({
        Node = {
            position_type = "Absolute",
            left = {Percent = 50},
            top = {Percent = 50},
            width = {Px = dialog_width},
            margin = { left = {Px = -dialog_width/2}, top = {Px = -80} },
            flex_direction = "Column",
            padding = { top = {Px = 16}, bottom = {Px = 16}, left = {Px = 16}, right = {Px = 16} },
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.bg },
        BorderRadius = { 
            top_left = {Px = 8}, top_right = {Px = 8}, 
            bottom_left = {Px = 8}, bottom_right = {Px = 8} 
        },
        GlobalZIndex = { value = 901 },
    }):id()
    
    local dialog = self.move_dialog
    
    -- Title
    spawn({
        Text = { text = "Move Item?" },
        TextFont = { font_size = 16 },
        TextColor = { color = self.colors.text },
    }):with_parent(dialog)
    
    -- Source path
    spawn({
        Text = { text = "From: " .. move.source.path },
        TextFont = { font_size = 11 },
        TextColor = { color = self.colors.text_dim },
        Node = { margin = { top = {Px = 8} } },
    }):with_parent(dialog)
    
    -- Target path
    local target_display = move.target_folder == "" and "(root)" or move.target_folder
    spawn({
        Text = { text = "To: " .. target_display },
        TextFont = { font_size = 11 },
        TextColor = { color = self.colors.text_dim },
        Node = { margin = { bottom = {Px = 16} } },
    }):with_parent(dialog)
    
    -- Button row
    local button_row = spawn({
        Node = {
            flex_direction = "Row",
            column_gap = {Px = 12},
        },
    }):with_parent(dialog):id()
    
    -- Cancel button
    local cancel_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 80},
            height = {Px = 32},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.button_bg },
        BorderRadius = { top_left = {Px = 4}, top_right = {Px = 4}, bottom_left = {Px = 4}, bottom_right = {Px = 4} },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self:close_move_dialog()
        end)
        :id()
    
    spawn({
        Text = { text = "Cancel" },
        TextFont = { font_size = 12 },
        TextColor = { color = self.colors.text },
    }):with_parent(cancel_btn)
    
    -- Move button
    local move_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 80},
            height = {Px = 32},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = {r = 0.2, g = 0.5, b = 0.3, a = 1.0} },
        BorderRadius = { top_left = {Px = 4}, top_right = {Px = 4}, bottom_left = {Px = 4}, bottom_right = {Px = 4} },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self:do_move()
        end)
        :id()
    
    spawn({
        Text = { text = "Move" },
        TextFont = { font_size = 12 },
        TextColor = { color = {r = 1, g = 1, b = 1, a = 1} },
    }):with_parent(move_btn)
end

--- Close move dialog
function FileBrowser:close_move_dialog()
    if self.move_backdrop then
        despawn(self.move_backdrop)
        self.move_backdrop = nil
    end
    if self.move_dialog then
        despawn(self.move_dialog)
        self.move_dialog = nil
    end
    -- NOTE: Don't clear pending_move here - it's cleared after do_move or by user
end

--- Execute the pending move
function FileBrowser:do_move()
    local move = self.pending_move
    if not move then
        self:close_move_dialog()
        return
    end
    
    self:close_move_dialog()
    self.pending_move = nil  -- Clear after move
    
    print("Moving " .. move.source.path .. " to " .. (move.target_folder == "" and "root" or move.target_folder))
    self:move_item_locally(move.source, move.target_folder)
    rename_asset(move.source.path, move.new_path)
end

--- Show move confirmation for multiple items
function FileBrowser:show_move_multiple_confirmation()
    self:close_move_dialog()
    
    if not self.pending_moves or #self.pending_moves == 0 then
        print("ERROR: No pending moves!")
        return
    end
    
    local count = #self.pending_moves
    local target = self.pending_moves[1].target_folder
    local dialog_width = 320
    
    -- Dialog backdrop
    self.move_backdrop = spawn({
        Button = {},
        Node = {
            position_type = "Absolute",
            left = {Px = 0}, right = {Px = 0},
            top = {Px = 0}, bottom = {Px = 0},
            width = {Percent = 100},
            height = {Percent = 100},
        },
        BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0.5} },
        GlobalZIndex = { value = 900 },
    })
        :observe("Pointer<Click>", function(entity, event)
            self.pending_moves = nil
            self:close_move_dialog()
        end)
        :id()
    
    -- Dialog container
    self.move_dialog = spawn({
        Node = {
            position_type = "Absolute",
            left = {Percent = 50},
            top = {Percent = 50},
            width = {Px = dialog_width},
            margin = { left = {Px = -dialog_width/2}, top = {Px = -80} },
            flex_direction = "Column",
            padding = { top = {Px = 16}, bottom = {Px = 16}, left = {Px = 16}, right = {Px = 16} },
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.bg },
        BorderRadius = { 
            top_left = {Px = 8}, top_right = {Px = 8}, 
            bottom_left = {Px = 8}, bottom_right = {Px = 8} 
        },
        GlobalZIndex = { value = 901 },
    }):id()
    
    local dialog = self.move_dialog
    
    -- Title
    spawn({
        Text = { text = "Move " .. count .. " items?" },
        TextFont = { font_size = 16 },
        TextColor = { color = self.colors.text },
    }):with_parent(dialog)
    
    -- Target folder text
    spawn({
        Text = { text = "To: " .. (target == "" and "/" or target) },
        TextFont = { font_size = 12 },
        TextColor = { color = self.colors.text_dim },
        Node = { margin = { top = {Px = 8}, bottom = {Px = 16} } },
    }):with_parent(dialog)
    
    -- Button row
    local button_row = spawn({
        Node = {
            flex_direction = "Row",
            justify_content = "Center",
            column_gap = {Px = 12},
        },
    }):with_parent(dialog):id()
    
    -- Cancel button
    local cancel_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 80},
            height = {Px = 32},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.row_hover },
        BorderRadius = { top_left = {Px = 4}, top_right = {Px = 4}, bottom_left = {Px = 4}, bottom_right = {Px = 4} },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self.pending_moves = nil
            self:close_move_dialog()
        end)
        :id()
    
    spawn({
        Text = { text = "Cancel" },
        TextFont = { font_size = 12 },
        TextColor = { color = self.colors.text },
    }):with_parent(cancel_btn)
    
    -- Move button
    local move_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 80},
            height = {Px = 32},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = {r = 0.2, g = 0.5, b = 0.3, a = 1.0} },
        BorderRadius = { top_left = {Px = 4}, top_right = {Px = 4}, bottom_left = {Px = 4}, bottom_right = {Px = 4} },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self:close_move_dialog()
            self:do_pending_moves()
        end)
        :id()
    
    spawn({
        Text = { text = "Move" },
        TextFont = { font_size = 12 },
        TextColor = { color = {r = 1, g = 1, b = 1, a = 1} },
    }):with_parent(move_btn)
end

--- Execute pending batch moves (for multi-item drag-drop)
function FileBrowser:do_pending_moves()
    if not self.pending_moves or #self.pending_moves == 0 then return end
    
    for _, move in ipairs(self.pending_moves) do
        print("Moving " .. move.source_path .. " to " .. move.new_path)
        -- Use rename_asset to move the file
        rename_asset(move.source_path, move.new_path)
        
        -- Update local folder lists for instant UI update
        -- Remove from source folder
        local source_folder = move.source_path:match("(.+)/[^/]+$") or ""
        local source_state = self.folders[source_folder]
        if source_state and source_state.items then
            for i, item in ipairs(source_state.items) do
                if item.path == move.source_path then
                    table.remove(source_state.items, i)
                    break
                end
            end
        end
        
        -- Add to target folder
        local target_state = self.folders[move.target_folder]
        if target_state and target_state.items then
            local item_name = move.source_path:match("([^/]+)$") or move.source_path
            local is_dir = self.folders[move.source_path] ~= nil
            table.insert(target_state.items, {
                name = item_name,
                path = move.new_path,
                is_directory = is_dir,
                size = 0
            })
        end
    end
    
    -- Clear selection and re-render
    self.selected_paths = {}
    self.pending_moves = nil
    self.needs_render = true
end

-- =============================================================================
-- New File Dialog
-- =============================================================================

--- Show new file dialog
function FileBrowser:show_new_file_dialog(target_folder)
    self:close_new_file_dialog()
    
    self.new_file_folder = target_folder
    self.new_file_name = "new_file.txt"
    local dialog_width = 300
    
    -- Dialog backdrop
    self.new_file_backdrop = spawn({
        Button = {},
        Node = {
            position_type = "Absolute",
            left = {Px = 0}, right = {Px = 0},
            top = {Px = 0}, bottom = {Px = 0},
        },
        BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0.5} },
        GlobalZIndex = { value = 700 },
    })
        :observe("Pointer<Click>", function(entity, event)
            self:close_new_file_dialog()
        end)
        :id()
    
    -- Dialog container (NOT a child of backdrop to prevent click bubbling)
    self.new_file_dialog = spawn({
        Node = {
            position_type = "Absolute",
            left = {Percent = 50},
            top = {Percent = 50},
            width = {Px = dialog_width},
            margin = { left = {Px = -dialog_width/2}, top = {Px = -80} },
            flex_direction = "Column",
            padding = { top = {Px = 16}, bottom = {Px = 16}, left = {Px = 16}, right = {Px = 16} },
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.bg },
        BorderRadius = { 
            top_left = {Px = 8}, top_right = {Px = 8}, 
            bottom_left = {Px = 8}, bottom_right = {Px = 8} 
        },
        GlobalZIndex = { value = 701 },
    }):id()
    
    local dialog = self.new_file_dialog
    
    -- Title
    spawn({
        Text = { text = "Create New File" },
        TextFont = { font_size = 16 },
        TextColor = { color = self.colors.text },
    }):with_parent(dialog)
    
    -- Folder location
    local folder_label = target_folder ~= "" and ("in " .. target_folder) or "in root folder"
    spawn({
        Text = { text = folder_label },
        TextFont = { font_size = 10 },
        TextColor = { color = self.colors.text_dim },
        Node = { margin = { top = {Px = 4}, bottom = {Px = 12} } },
    }):with_parent(dialog)
    
    -- File name input
    self.new_file_input_entity = spawn({
        LuaTextInput = { 
            initial_value = "new_file.txt",  -- Default filename
            auto_focus = true 
        },
        Node = {
            width = {Percent = 100},
            height = {Px = 28},
            padding = { left = {Px = 8}, right = {Px = 8} },
            align_items = "Center",
            margin = { bottom = {Px = 12} },
        },
        BackgroundColor = { color = {r = 0.1, g = 0.1, b = 0.12, a = 1.0} },
        BorderRadius = { 
            top_left = {Px = 4}, top_right = {Px = 4}, 
            bottom_left = {Px = 4}, bottom_right = {Px = 4} 
        },
        TextColor = { color = self.colors.text },
        TextFont = { font_size = 12 },
    }):with_parent(dialog):id()
    
    -- Button row
    local button_row = spawn({
        Node = {
            flex_direction = "Row",
            justify_content = "Center",
            column_gap = {Px = 12},
        },
    }):with_parent(dialog):id()
    
    -- Cancel button  
    local cancel_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 80},
            height = {Px = 32},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.row_hover },
        BorderRadius = { top_left = {Px = 4}, top_right = {Px = 4}, bottom_left = {Px = 4}, bottom_right = {Px = 4} },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self:close_new_file_dialog()
        end)
        :id()
    
    spawn({
        Text = { text = "Cancel" },
        TextFont = { font_size = 12 },
        TextColor = { color = self.colors.text },
    }):with_parent(cancel_btn)
    
    -- Create button
    local create_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 80},
            height = {Px = 32},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = {r = 0.3, g = 0.6, b = 0.3, a = 1.0} },
        BorderRadius = { top_left = {Px = 4}, top_right = {Px = 4}, bottom_left = {Px = 4}, bottom_right = {Px = 4} },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self:do_create_file()
        end)
        :id()
    
    spawn({
        Text = { text = "Create" },
        TextFont = { font_size = 12 },
        TextColor = { color = {r = 1, g = 1, b = 1, a = 1} },
    }):with_parent(create_btn)
end

--- Close new file dialog
function FileBrowser:close_new_file_dialog()
    if self.new_file_backdrop then
        despawn(self.new_file_backdrop)
        self.new_file_backdrop = nil
    end
    if self.new_file_dialog then
        despawn(self.new_file_dialog)
        self.new_file_dialog = nil
    end
    self.new_file_folder = nil
    self.new_file_name = nil
    self.new_file_input_entity = nil
end

--- Schedule file creation (called from observer callback)
function FileBrowser:do_create_file()
    -- Set pending flag - actual creation happens in update system where world is available
    self.pending_create_file = {
        folder = self.new_file_folder or "",
        input_entity = self.new_file_input_entity
    }
    -- Dialog closes in process_pending_create_file AFTER reading the text
end

--- Process pending file creation (called from update system with world access)
function FileBrowser:process_pending_create_file(world)
    if not self.pending_create_file then return end
    
    local pending = self.pending_create_file
    self.pending_create_file = nil
    
    local file_name = nil
    
    -- Read text from the input entity's LuaTextInputValue component
    -- Just get the first one - there should only be one active text input at a time
    local entities = world:query({"LuaTextInputValue"}, nil)
    if #entities > 0 then
        local entity = entities[1]
        local value = entity:get("LuaTextInputValue")
        if value and value.text then
            file_name = value.text
        end
    end
    
    -- Close dialog AFTER reading text
    self:close_new_file_dialog()
    
    if file_name and file_name ~= "" then
        local relative_path = pending.folder ~= "" and (pending.folder .. "/" .. file_name) or file_name
        local full_path = "assets/" .. relative_path
        print("Creating file: " .. full_path)
        
        -- Write empty file to filesystem
        write_file_bytes(full_path, "")
        
        -- Add file to local folder list immediately (avoid waiting for server refresh)
        local folder_state = self.folders[pending.folder]
        if folder_state and folder_state.items then
            -- Insert new file at end of items list
            table.insert(folder_state.items, {
                name = file_name,
                path = relative_path,
                is_directory = false,
                size = 0
            })
        end
        
        -- Re-render the tree immediately
        self:render_tree()
    else
        print("File creation cancelled - no name provided")
    end
end

-- =============================================================================
-- New Folder Dialog
-- =============================================================================

--- Show new folder dialog
function FileBrowser:show_new_folder_dialog(target_folder)
    self:close_new_folder_dialog()
    
    self.new_folder_folder = target_folder
    self.new_folder_name = "new_folder"
    local dialog_width = 300
    
    -- Dialog backdrop
    self.new_folder_backdrop = spawn({
        Button = {},
        Node = {
            position_type = "Absolute",
            left = {Px = 0}, right = {Px = 0},
            top = {Px = 0}, bottom = {Px = 0},
        },
        BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0.5} },
        GlobalZIndex = { value = 700 },
    })
        :observe("Pointer<Click>", function(entity, event)
            self:close_new_folder_dialog()
        end)
        :id()
    
    -- Dialog container
    self.new_folder_dialog = spawn({
        Node = {
            position_type = "Absolute",
            left = {Percent = 50},
            top = {Percent = 50},
            width = {Px = dialog_width},
            margin = { left = {Px = -dialog_width/2}, top = {Px = -80} },
            flex_direction = "Column",
            padding = { top = {Px = 16}, bottom = {Px = 16}, left = {Px = 16}, right = {Px = 16} },
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.bg },
        BorderRadius = { 
            top_left = {Px = 8}, top_right = {Px = 8}, 
            bottom_left = {Px = 8}, bottom_right = {Px = 8} 
        },
        GlobalZIndex = { value = 701 },
    }):id()
    
    local dialog = self.new_folder_dialog
    
    -- Title
    spawn({
        Text = { text = "Create New Folder" },
        TextFont = { font_size = 16 },
        TextColor = { color = self.colors.text },
        Node = { margin = { bottom = {Px = 12} } },
    }):with_parent(dialog)
    
    -- Folder name input (matches file dialog styling)
    self.new_folder_input_entity = spawn({
        LuaTextInput = { 
            initial_value = "new_folder",  -- Default folder name
            auto_focus = true 
        },
        Node = {
            width = {Percent = 100},
            height = {Px = 28},
            padding = { left = {Px = 8}, right = {Px = 8} },
            align_items = "Center",
            margin = { bottom = {Px = 12} },
        },
        BackgroundColor = { color = {r = 0.1, g = 0.1, b = 0.12, a = 1.0} },
        BorderRadius = { 
            top_left = {Px = 4}, top_right = {Px = 4}, 
            bottom_left = {Px = 4}, bottom_right = {Px = 4} 
        },
        TextColor = { color = self.colors.text },
        TextFont = { font_size = 12 },
    }):with_parent(dialog):id()
    
    -- Button row
    local button_row = spawn({
        Node = {
            flex_direction = "Row",
            column_gap = {Px = 12},
        },
    }):with_parent(dialog):id()
    
    -- Cancel button
    local cancel_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 80},
            height = {Px = 32},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = self.colors.button_bg },
        BorderRadius = { top_left = {Px = 4}, top_right = {Px = 4}, bottom_left = {Px = 4}, bottom_right = {Px = 4} },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self:close_new_folder_dialog()
        end)
        :id()
    
    spawn({
        Text = { text = "Cancel" },
        TextFont = { font_size = 12 },
        TextColor = { color = self.colors.text },
    }):with_parent(cancel_btn)
    
    -- Create button
    local create_btn = spawn({
        Button = {},
        Node = {
            width = {Px = 80},
            height = {Px = 32},
            justify_content = "Center",
            align_items = "Center",
        },
        BackgroundColor = { color = {r = 0.2, g = 0.5, b = 0.3, a = 1.0} },
        BorderRadius = { top_left = {Px = 4}, top_right = {Px = 4}, bottom_left = {Px = 4}, bottom_right = {Px = 4} },
    })
        :with_parent(button_row)
        :observe("Pointer<Click>", function(entity, event)
            self:do_create_folder()
        end)
        :id()
    
    spawn({
        Text = { text = "Create" },
        TextFont = { font_size = 12 },
        TextColor = { color = {r = 1, g = 1, b = 1, a = 1} },
    }):with_parent(create_btn)
end

--- Close new folder dialog
function FileBrowser:close_new_folder_dialog()
    if self.new_folder_backdrop then
        despawn(self.new_folder_backdrop)
        self.new_folder_backdrop = nil
    end
    if self.new_folder_dialog then
        despawn(self.new_folder_dialog)
        self.new_folder_dialog = nil
    end
    self.new_folder_folder = nil
    self.new_folder_input_entity = nil
end

--- Schedule folder creation (called from observer callback)
function FileBrowser:do_create_folder()
    -- Set pending flag - actual creation happens in update system where world is available
    self.pending_create_folder = {
        folder = self.new_folder_folder or "",
        input_entity = self.new_folder_input_entity
    }
    -- Dialog closes in process_pending_create_folder AFTER reading the text
end

--- Process pending folder creation (called from update system with world access)
function FileBrowser:process_pending_create_folder(world)
    if not self.pending_create_folder then return end
    
    local pending = self.pending_create_folder
    self.pending_create_folder = nil
    
    -- Get folder name from text input
    local folder_name = nil
    local entities = world:query({"LuaTextInputValue"}, nil)
    if #entities > 0 then
        local entity = entities[1]
        local value = entity:get("LuaTextInputValue")
        if value and value.text then
            folder_name = value.text
        end
    end
    
    -- Close dialog AFTER reading text
    self:close_new_folder_dialog()
    
    if folder_name and folder_name ~= "" then
        local relative_path = pending.folder ~= "" and (pending.folder .. "/" .. folder_name) or folder_name
        local full_path = "assets/" .. relative_path
        
        -- Create the directory
        create_directory(full_path)
        
        -- Add folder to local folder list immediately
        local folder_state = self.folders[pending.folder]
        if folder_state and folder_state.items then
            -- Insert new folder at beginning of items list (folders come first)
            table.insert(folder_state.items, 1, {
                name = folder_name,
                path = relative_path,
                is_directory = true,
            })
        end
        
        -- Re-render the tree immediately
        self:render_tree()
    end
end

--- Handle right click on row (called from render_row)
function FileBrowser:on_row_right_click(item, x, y)
    self:show_context_menu(item, x, y)
end

--- Show file picker dialog and upload selected files
function FileBrowser:show_file_picker()
    -- Determine target folder - use first selected folder or root
    local target_folder = ""
    -- Get first selected path
    local first_selected = nil
    for path, _ in pairs(self.selected_paths) do
        first_selected = path
        break
    end
    if first_selected then
        -- If selected is a folder, use it; if file, use its parent
        local folder_state = self.folders[first_selected]
        if folder_state then
            target_folder = first_selected
        else
            target_folder = first_selected:match("(.+)/[^/]+$") or ""
        end
    end
    
    -- Try to use native file picker (requires rfd binding)
    local ok, paths = pcall(pick_files_dialog)
    if ok and paths and #paths > 0 then
        for _, source_path in ipairs(paths) do
            local filename = source_path:match("([^/\\]+)$") or "uploaded_file"
            local relative_dest = target_folder ~= "" and (target_folder .. "/" .. filename) or filename
            local full_dest = "assets/" .. relative_dest
            
            -- Copy file to assets folder
            local copy_ok = pcall(function()
                copy_file(source_path, full_dest)
            end)
            
            if copy_ok then
                -- Add to local folder list immediately (only if not already present)
                local folder_state = self.folders[target_folder]
                if folder_state and folder_state.items then
                    -- Check if file already exists to prevent duplicates
                    local already_exists = false
                    for _, existing in ipairs(folder_state.items) do
                        if existing.path == relative_dest then
                            already_exists = true
                            break
                        end
                    end
                    if not already_exists then
                        table.insert(folder_state.items, {
                            name = filename,
                            path = relative_dest,
                            is_directory = false,
                            size = 0,  -- Size will be updated on next refresh
                        })
                    end
                end
            end
        end
        -- Use deferred rendering to avoid despawn issues
        self.needs_render = true
    else
        -- If file picker not available, print message
        print("File picker not available - please drag and drop files instead")
    end
end

return FileBrowser
