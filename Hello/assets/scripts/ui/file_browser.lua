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
    self.selected_path = nil
    
    -- Drag drop state
    self.drag_target = nil  -- Current drop target folder
    self.dragging_item = nil  -- Item being dragged
    self.panel_hovered = false  -- Track if mouse is over panel (for external drops)
    self.drop_overlay = nil  -- Drop overlay entity
    
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
    
    -- Register Update system for deferred operations (needs world access)
    local browser = self  -- Capture self for closure
    register_system("Update", function(world)
        browser:process_pending_rename(world)
        browser:process_pending_create_file(world)
    end)
    
    return self
end

--- Show the file browser
function FileBrowser:show()
    if self.is_visible then return end
    
    self:spawn_panel()
    self:load_folder("")  -- Load root
    
    self.is_visible = true
end

--- Hide the file browser
function FileBrowser:hide()
    if not self.is_visible then return end
    
    self:destroy_all()
    self.is_visible = false
end

--- Spawn the main panel
function FileBrowser:spawn_panel()
    -- Main panel container (fixed left)
    self.panel_entity = spawn({
        Node = {
            position_type = "Absolute",
            left = {Px = 0},
            top = {Px = 0},
            bottom = {Px = 0},
            width = {Px = PANEL_WIDTH},
            flex_direction = "Column",
        },
        BackgroundColor = { color = self.colors.bg },
        GlobalZIndex = { value = 100 },
    })
        :observe("Pointer<Over>", function(entity, event)
            self.panel_hovered = true
        end)
        :observe("Pointer<Out>", function(entity, event)
            self.panel_hovered = false
        end)
        :id()
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
    
    -- Refresh button (child of header)
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
        :with_parent(header)
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
            print("Scroll y: " .. tostring(scroll.y) .. " current offset: " .. tostring(self.scroll_offset))
            self.scroll_offset = self.scroll_offset - scroll.y * 2
            if self.scroll_offset < 0 then
                self.scroll_offset = 0
            end
            -- Update ScrollPosition
            entity:set({ ScrollPosition = { offset = {x = 0, y = self.scroll_offset} } })
        end)
        :observe("Pointer<DragOver>", function(entity, event)
            -- Scroll container as root drop target
            -- Only set root target if no folder target is set (prevents bubbling from overwriting)
            if self.dragging_item and (self.drag_target == nil or self.drag_target == "") then
                print("DragOver scroll container (root)")
                self.drag_target = ""  -- Empty string = root directory
            end
        end)
        :observe("Pointer<DragEnd>", function(entity, event)
            -- Handle ALL drops (both folder and root targets)
            print("DragEnd - drag_target: " .. tostring(self.drag_target))
            if self.dragging_item then
                local source = self.dragging_item
                local target = self.drag_target
                
                if target == "" then
                    -- Drop to root directory
                    if source.path:find("/") then  -- Only move if not already in root
                        print("Moving " .. source.path .. " to root directory")
                        self:move_item_locally(source, "")
                        rename_asset(source.path, source.name)
                    end
                elseif target and target ~= source.path then
                    -- Drop to folder
                    print("Moving " .. source.path .. " to folder " .. target)
                    local new_path = target .. "/" .. source.name
                    self:move_item_locally(source, target)
                    rename_asset(source.path, new_path)
                end
            end
            -- Clear drag state
            self.dragging_item = nil
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
                local x = (click.pointer_location and click.pointer_location.position and click.pointer_location.position.x) or 100
                local y = (click.pointer_location and click.pointer_location.position and click.pointer_location.position.y) or 100
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
    
    -- Re-render
    self:render_tree()
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
    local is_selected = self.selected_path == item.path
    
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
    })
        :with_parent(self.scroll_container)
        :observe("Pointer<Click>", function(entity, event)
            -- Check for right-click (Secondary button) via reflected event data
            local click = event.event
            local is_right_click = click and click.button and click.button.variant == "Secondary"
            
            if is_right_click then
                -- Right-click: only select the item, don't toggle folder
                -- (toggling would trigger render_tree which despawns this row mid-click)
                self.selected_path = item.path
                -- Get position for context menu
                local x = (click.pointer_location and click.pointer_location.position and click.pointer_location.position.x) or 100
                local y = (click.pointer_location and click.pointer_location.position and click.pointer_location.position.y) or 100
                -- Set flag to prevent scroll container from also showing menu (event bubbles)
                self.context_menu_handled = true
                self:on_row_right_click(item, x, y)
            else
                self:on_row_click(item)
            end
        end)
        :observe("Pointer<Over>", function(entity, event)
            -- Subtle hover highlight (only if not selected)
            if self.selected_path ~= item.path then
                entity:set({ BackgroundColor = { color = {r = 0.18, g = 0.18, b = 0.22, a = 1.0} } })
            end
        end)
        :observe("Pointer<Out>", function(entity, event)
            -- Restore background (only if not selected and not drag target)
            if self.selected_path ~= item.path and self.drag_target ~= item.path then
                entity:set({ BackgroundColor = { color = {r = 0, g = 0, b = 0, a = 0} } })
            end
        end)
        :observe("Pointer<DragStart>", function(entity, event)
            -- Start dragging this item
            self.dragging_item = item
            print("Started dragging: " .. item.path)
        end)
        :observe("Pointer<DragOver>", function(entity, event)
            -- Highlight as drop target if this is a folder and not the same item
            if item.is_directory and self.dragging_item and self.dragging_item.path ~= item.path then
                print("DragOver folder: " .. item.path)
                self.drag_target = item.path
                entity:set({ BackgroundColor = { color = {r = 0.2, g = 0.3, b = 0.4, a = 1.0} } })
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
    if not item.is_directory and item.size > 0 then
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
function FileBrowser:on_row_click(item)
    self.selected_path = item.path
    
    if item.is_directory then
        -- toggle_folder sets needs_render flag
        self:toggle_folder(item.path)
    else
        -- For files, mark for re-render to update selection highlighting
        self.needs_render = true
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
        elseif e.HoveredFileCancelled then
            print("HoveredFileCancelled - hiding overlay")
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
        local target_folder = self.selected_path or ""  -- Use selected folder or root
        
        -- Only drop into folders, not files
        if self.selected_path then
            local folder_state = self.folders[self.selected_path]
            if not folder_state then
                -- Selected item is a file, use its parent folder
                target_folder = self.selected_path:match("(.+)/[^/]+$") or ""
            end
        end
        -- Extract filename from source path
        -- Strip surrounding quotes if present (Windows sometimes adds them)
        source_path = source_path:gsub('^"', ''):gsub('"$', '')
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
            
            -- Add to local folder list immediately
            local folder_state = self.folders[target_folder]
            if folder_state and folder_state.items then
                table.insert(folder_state.items, {
                    name = filename,
                    path = relative_dest,
                    is_directory = false,
                    size = 0  -- Unknown size for now
                })
            end
            
            -- Re-render tree
            self:render_tree()
            
            -- Also upload to server if in network mode
            if upload_asset then
                upload_asset(relative_dest, relative_dest, true)
            end
        else
            print("Failed to copy file: " .. tostring(err))
        end
    end
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
    
    -- Only show rename/delete if we have an item with a real path (not root)
    if item and item.path and item.path ~= "" then
        -- Rename option
        self:add_context_menu_item("Rename", function()
            self:close_context_menu()
            self:show_rename_dialog(item)
        end)
        
        -- Delete option
        self:add_context_menu_item("Delete", function()
            self:close_context_menu()
            self:show_delete_confirmation(item)
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
    print("Found " .. #entities .. " entities with LuaTextInputValue")
    if #entities > 0 then
        local entity = entities[1]
        local value = entity:get("LuaTextInputValue")
        print("Got LuaTextInputValue: " .. tostring(value))
        if value then
            print("Value has text: " .. tostring(value.text))
            if value.text then
                new_name = value.text
                print("Read text from input: '" .. new_name .. "'")
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
    print("Found " .. #entities .. " entities with LuaTextInputValue for file creation")
    if #entities > 0 then
        local entity = entities[1]
        local value = entity:get("LuaTextInputValue")
        if value and value.text then
            file_name = value.text
            print("Read filename from input: '" .. file_name .. "'")
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

--- Handle right click on row (called from render_row)
function FileBrowser:on_row_right_click(item, x, y)
    print("Right click on row: " .. item.path)
    self:show_context_menu(item, x, y)
end

return FileBrowser
