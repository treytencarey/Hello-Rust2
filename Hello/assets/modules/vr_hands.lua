-- VR Hand Mesh Module v2
-- Modular hand tracking with configurable bone mappings
--
-- Usage:
--   local VrHands = require("modules/vr_hands.lua")
--   
--   local hand = VrHands.create({
--       asset_path = "vr_hand2/LeftHand.gltf#Scene0",
--       hand = "left",  -- "left" or "right"
--       scale = 1.0,    -- optional
--       bone_map = VrHands.DEFAULT_BONE_MAPS.left,  -- optional, asset-specific
--   })
--   
--   register_system("Update", function(world) hand:update(world) end)
--   hand:cleanup()

local VrHands = {}
VrHands.__index = VrHands

--------------------------------------------------------------------------------
-- Default Bone Mapping (for vr_hand2/RightHand.gltf model)
-- Maps OpenXR HandBone enum values to mesh bone names
-- Same mesh is used for both hands (left is mirrored via negative scale)
--------------------------------------------------------------------------------

VrHands.DEFAULT_BONE_MAP = {
    -- Wrist/Palm (used for root positioning)
    Wrist = "hands:b_r_hand",
    Palm = "hands:b_r_grip",
    
    -- Thumb
    ThumbMetacarpal = "hands:b_r_thumb1",
    ThumbProximal = "hands:b_r_thumb2",
    ThumbDistal = "hands:b_r_thumb3",
    ThumbTip = "hands:b_r_thumb_ignore",
    
    -- Index
    IndexMetacarpal = "hands:b_r_index1",
    IndexProximal = "hands:b_r_index2",
    IndexIntermediate = "hands:b_r_index3",
    IndexDistal = "hands:b_r_index_ignore",
    IndexTip = "hands:b_r_index_ignore",
    
    -- Middle
    MiddleMetacarpal = "hands:b_r_middle1",
    MiddleProximal = "hands:b_r_middle2",
    MiddleIntermediate = "hands:b_r_middle3",
    MiddleDistal = "hands:b_r_middle_ignore",
    MiddleTip = "hands:b_r_middle_ignore",
    
    -- Ring
    RingMetacarpal = "hands:b_r_ring1",
    RingProximal = "hands:b_r_ring2",
    RingIntermediate = "hands:b_r_ring3",
    RingDistal = "hands:b_r_ring_ignore",
    RingTip = "hands:b_r_ring_ignore",
    
    -- Little/Pinky
    LittleMetacarpal = "hands:b_r_pinky0",
    LittleProximal = "hands:b_r_pinky1",
    LittleIntermediate = "hands:b_r_pinky2",
    LittleDistal = "hands:b_r_pinky3",
    LittleTip = "hands:b_r_pinky_ignore",
}

--------------------------------------------------------------------------------
-- XR Bone Parent Hierarchy (for computing local rotations)
--------------------------------------------------------------------------------

VrHands.XR_BONE_PARENTS = {
    -- Wrist is child of Palm
    Wrist = "Palm",
    
    -- Thumb chain
    ThumbMetacarpal = "Wrist",
    ThumbProximal = "ThumbMetacarpal",
    ThumbDistal = "ThumbProximal",
    ThumbTip = "ThumbDistal",
    
    -- Index chain
    IndexMetacarpal = "Wrist",
    IndexProximal = "IndexMetacarpal",
    IndexIntermediate = "IndexProximal",
    IndexDistal = "IndexIntermediate",
    IndexTip = "IndexDistal",
    
    -- Middle chain
    MiddleMetacarpal = "Wrist",
    MiddleProximal = "MiddleMetacarpal",
    MiddleIntermediate = "MiddleProximal",
    MiddleDistal = "MiddleIntermediate",
    MiddleTip = "MiddleDistal",
    
    -- Ring chain
    RingMetacarpal = "Wrist",
    RingProximal = "RingMetacarpal",
    RingIntermediate = "RingProximal",
    RingDistal = "RingIntermediate",
    RingTip = "RingDistal",
    
    -- Little/Pinky chain
    LittleMetacarpal = "Wrist",
    LittleProximal = "LittleMetacarpal",
    LittleIntermediate = "LittleProximal",
    LittleDistal = "LittleIntermediate",
    LittleTip = "LittleDistal",
}

--------------------------------------------------------------------------------
-- Factory Function
--------------------------------------------------------------------------------

--- Create a new VR hand tracker instance
--- @param config table { asset_path, hand, scale?, bone_map?, rotation_offset? }
--- @return table Hand tracker instance
function VrHands.create(config)
    assert(config.asset_path, "asset_path is required")
    assert(config.hand == "left" or config.hand == "right", "hand must be 'left' or 'right'")
    
    local self = setmetatable({}, VrHands)
    
    -- Configuration
    self.hand = config.hand
    self.asset_path = config.asset_path
    self.scale = config.scale or 1.0
    self.bone_map = config.bone_map or VrHands.DEFAULT_BONE_MAP
    
    -- Controller grip offset: compensates for mesh rest pose vs XR neutral pose
    -- These values are hand-specific because mirroring affects rotations differently
    local default_grip_offset
    if config.hand == "right" then
        default_grip_offset = { x = -25, y = -45, z = 225 }
    else
        -- Left hand offset (mirrored) - adjust these values as needed
        default_grip_offset = { x = -25, y = 45, z = -225 }  -- Y and Z negated for mirror
    end
    self.grip_offset_degrees = config.grip_offset or default_grip_offset
    self.grip_offset_quat = nil  -- Will be computed on first update
    
    -- State
    self.root_entity = nil
    self.mesh_bones = {}  -- bone_name -> entity_id
    self.bones_cached = false
    
    -- Reference pose state (for rotation delta approach)
    self.xr_reference_rotations = {}   -- HandBone -> palm-relative quaternion
    self.mesh_initial_rotations = {}   -- entity_id -> initial local quaternion
    self.reference_captured = false
    
    -- Mesh offset compensation (for GLTF meshes not at origin)
    self.mesh_wrist_bone_id = nil      -- The wrist/hand bone entity we use for offset
    self.mesh_offset_captured = false
    self.mesh_initial_offset = nil     -- Vec3: offset from root to wrist bone in mesh local space
    self.mesh_initial_rotation = nil   -- Quat: initial rotation of wrist bone in world space
    
    -- XR initial rotation (for rotation delta mapping)
    self.xr_initial_rotation = nil     -- Quat: XR hand rotation when first captured
    self.xr_initial_inv = nil          -- Quat: inverse of XR initial (cached for perf)
    self.xr_rotation_captured = false
    
    -- Debug flags
    self._init_logged = false
    self._bones_logged = false
    
    -- Spawn the hand mesh
    self:_spawn()
    
    return self
end

--------------------------------------------------------------------------------
-- Internal: Spawn Hand Mesh
--------------------------------------------------------------------------------

function VrHands:_spawn()
    local handle = load_asset(self.asset_path)
    
    -- Determine scale (left hand may need mirroring)
    local scale_x = self.scale
    local scale_y = self.scale
    local scale_z = self.scale
    
    -- For left hand, mirror on X axis
    if self.hand == "left" then
        scale_x = -self.scale
    end
    
    local entity = spawn({
        SceneRoot = { id = handle },
        Transform = {
            translation = { x = 0, y = 0, z = 0 },
            rotation = { x = 0, y = 0, z = 0, w = 1 },
            scale = { x = scale_x, y = scale_y, z = scale_z }
        },
        VrHandMesh = { hand = self.hand }
    })
    
    self.root_entity = entity:id()
    print(string.format("[VR_HANDS] Spawned %s hand mesh: %s", self.hand, tostring(self.root_entity)))
end

--------------------------------------------------------------------------------
-- Internal: Parse Bevy entity debug format "indexVversion" to numeric bits
-- Example: "22v9" -> (9 << 32) | 22 = 38654705686
--------------------------------------------------------------------------------

function VrHands._parse_entity_debug(str)
    if type(str) ~= "string" then
        return nil
    end
    
    local index_str, ver_str = str:match("^(%d+)v(%d+)$")
    if not index_str or not ver_str then
        return nil
    end
    
    local index = tonumber(index_str)
    local version = tonumber(ver_str)
    if not index or not version then
        return nil
    end
    
    -- Bevy Entity bits = (version << 32) | index
    -- Since Lua uses 64-bit floats, this should work for reasonable entity counts
    return (version * 4294967296) + index  -- 4294967296 = 2^32
end

--------------------------------------------------------------------------------
-- Internal: Check if entity is descendant of our root (walks up ChildOf chain)
--------------------------------------------------------------------------------

function VrHands:_is_descendant_of(world, entity_id, max_depth)
    max_depth = max_depth or 50
    local current_id = entity_id
    
    for _ = 1, max_depth do
        if current_id == world:get_entity(self.root_entity):id() then
            return true
        end
        
        local entity = world:get_entity(current_id)
        if not entity then
            return false
        end
        
        local child_of = entity:get("ChildOf")
        if not child_of then
            return false  -- No parent, reached top
        end
        
        current_id = child_of._0
    end
    
    return false
end

--------------------------------------------------------------------------------
-- Internal: Build Mesh Bone Cache (only for bones belonging to this instance)
--------------------------------------------------------------------------------

function VrHands:_build_bone_cache(world)
    if self.bones_cached then return true end
    
    local named_entities = world:query({"Name", "Transform", "ChildOf"}, nil)
    if not named_entities or #named_entities == 0 then
        return false
    end
    
    -- Build reverse lookup: mesh_bone_name -> HandBone
    local bone_name_to_xr = {}
    for xr_bone, mesh_name in pairs(self.bone_map) do
        bone_name_to_xr[mesh_name] = xr_bone
    end
    
    local found_count = 0
    
    for _, entity in ipairs(named_entities) do
        -- Only process entities that are descendants of our hand mesh root
        if not self:_is_descendant_of(world, entity:id()) then
            goto continue_entity
        end

        local name_component = entity:get("Name")
        if name_component then
            local bone_name = nil
            if type(name_component) == "string" then
                bone_name = name_component
            elseif type(name_component) == "table" then
                bone_name = name_component.name
            end
            
            if bone_name then
                bone_name = bone_name:gsub('^"', ''):gsub('"$', '')
                
                -- Check if this bone is in our mapping
                if bone_name_to_xr[bone_name] then
                    local entity_id = entity:id()
                    
                    -- Store by mesh bone name
                    if not self.mesh_bones[bone_name] then
                        self.mesh_bones[bone_name] = entity_id
                        
                        -- Cache initial rotation
                        local transform = entity:get("Transform")
                        if transform and transform.rotation then
                            self.mesh_initial_rotations[entity_id] = {
                                x = transform.rotation.x or 0,
                                y = transform.rotation.y or 0,
                                z = transform.rotation.z or 0,
                                w = transform.rotation.w or 1
                            }
                        end
                        
                        found_count = found_count + 1
                    end
                end
            end
        end
        
        ::continue_entity::
    end
    
    if found_count > 0 then
        if not self._bones_logged then
            print(string.format("[VR_HANDS] %s hand: cached %d mesh bones", self.hand, found_count))
            self._bones_logged = true
        end
        self.bones_cached = true
        
        -- Store wrist bone ID for offset calculation
        local wrist_bone_name = self.bone_map["Wrist"]
        if wrist_bone_name and self.mesh_bones[wrist_bone_name] then
            self.mesh_wrist_bone_id = self.mesh_bones[wrist_bone_name]
        end
        
        return true
    end
    
    return false
end

--------------------------------------------------------------------------------
-- Internal: Capture Mesh Offset (compensate for GLTF mesh not at origin)
--------------------------------------------------------------------------------

function VrHands:_capture_mesh_offset(world)
    if self.mesh_offset_captured then return true end
    if not self.mesh_wrist_bone_id then return false end
    
    -- Get the wrist bone's GlobalTransform
    local wrist_translation = world:call_component_method(self.mesh_wrist_bone_id, "GlobalTransform", "translation")
    local wrist_rotation = world:call_component_method(self.mesh_wrist_bone_id, "GlobalTransform", "rotation")
    
    if not wrist_translation or not wrist_rotation then
        return false
    end
    
    -- Get root entity's current GlobalTransform
    local root_translation = world:call_component_method(self.root_entity, "GlobalTransform", "translation")
    local root_rotation = world:call_component_method(self.root_entity, "GlobalTransform", "rotation")
    
    if not root_translation or not root_rotation then
        return false
    end
    
    -- The position offset is how far the wrist bone is from the scene root in world space
    -- We'll use this to compute where to place the root so the wrist ends up at the XR position
    self.mesh_initial_offset = {
        x = wrist_translation.x - root_translation.x,
        y = wrist_translation.y - root_translation.y,
        z = wrist_translation.z - root_translation.z
    }
    
    -- Store the root entity's rotation as coordinate system compensation
    -- This is the Armature rotation which has the Z-up to Y-up conversion
    self.mesh_initial_rotation = root_rotation
    
    self.mesh_offset_captured = true
    
    print(string.format("[VR_HANDS] %s hand: captured mesh offset pos=(%.2f, %.2f, %.2f) rot=(%.2f, %.2f, %.2f, %.2f)",
        self.hand,
        self.mesh_initial_offset.x,
        self.mesh_initial_offset.y,
        self.mesh_initial_offset.z,
        wrist_rotation.x, wrist_rotation.y, wrist_rotation.z, wrist_rotation.w))
    
    return true
end

--------------------------------------------------------------------------------
-- Internal: Capture XR Initial Rotation (for rotation delta mapping)
--------------------------------------------------------------------------------

function VrHands:_capture_xr_initial_rotation(world, xr_rotation)
    if self.xr_rotation_captured then return true end
    if not xr_rotation then return false end
    if not self.mesh_initial_rotation then return false end  -- Need mesh rotation first
    
    self.xr_initial_rotation = xr_rotation
    self.xr_initial_inv = world:call_static_method("Quat", "inverse", xr_rotation)
    self.xr_rotation_captured = true
    
    print(string.format("[VR_HANDS] %s hand: captured XR initial rotation (%.2f, %.2f, %.2f, %.2f)",
        self.hand,
        xr_rotation.x, xr_rotation.y, xr_rotation.z, xr_rotation.w))
    
    return true
end

--------------------------------------------------------------------------------
-- Internal: Capture Reference Pose (XR local rotations at rest)
--------------------------------------------------------------------------------

function VrHands:_capture_reference(world, xr_bones)
    if self.reference_captured then return true end
    if not xr_bones or #xr_bones == 0 then return false end
    
    -- Build lookup table first
    local xr_bone_lookup = {}
    for _, bone_entity in ipairs(xr_bones) do
        local hand_bone = bone_entity:get("HandBone")
        if hand_bone then
            xr_bone_lookup[hand_bone] = bone_entity
        end
    end
    
    local cached_count = 0
    
    for _, bone_entity in ipairs(xr_bones) do
        local hand_bone = bone_entity:get("HandBone")
        local flags = bone_entity:get("XrSpaceLocationFlags")
        
        if flags and (flags.position_tracked == false or flags.rotation_tracked == false) then
            goto continue_cache
        end
        
        -- Skip Palm and Wrist (not finger bones)
        if hand_bone == "Palm" or hand_bone == "Wrist" then
            goto continue_cache
        end
        
        -- Find parent bone
        local parent_bone_name = VrHands.XR_BONE_PARENTS[hand_bone]
        if not parent_bone_name then
            goto continue_cache
        end
        
        local parent_entity = xr_bone_lookup[parent_bone_name]
        if not parent_entity then
            goto continue_cache
        end
        
        -- Get world rotations
        local parent_world_rot = world:call_component_method(parent_entity:id(), "GlobalTransform", "rotation")
        local child_world_rot = world:call_component_method(bone_entity:id(), "GlobalTransform", "rotation")
        
        if parent_world_rot and child_world_rot then
            -- Compute local rotation: local = inv(parent) * child
            local inv_parent = world:call_static_method("Quat", "inverse", parent_world_rot)
            local local_rot = world:call_static_method("Quat", "mul_quat", inv_parent, child_world_rot)
            
            -- Store the XR reference LOCAL rotation
            self.xr_reference_rotations[hand_bone] = local_rot
            cached_count = cached_count + 1
        end
        
        ::continue_cache::
    end
    
    if cached_count > 0 then
        print(string.format("[VR_HANDS] %s hand: captured %d XR reference LOCAL rotations", self.hand, cached_count))
        self.reference_captured = true
        return true
    end
    
    return false
end

--------------------------------------------------------------------------------
-- Internal: Update Finger Bones
--------------------------------------------------------------------------------

function VrHands:_update_finger_bones(world, xr_bones)
    if not xr_bones or #xr_bones == 0 then return end
    
    -- Build a lookup table: HandBone name -> entity
    local xr_bone_lookup = {}
    for _, bone_entity in ipairs(xr_bones) do
        local hand_bone = bone_entity:get("HandBone")
        if hand_bone then
            xr_bone_lookup[hand_bone] = bone_entity
        end
    end
    
    -- Process each finger bone
    for _, bone_entity in ipairs(xr_bones) do
        local hand_bone = bone_entity:get("HandBone")
        local flags = bone_entity:get("XrSpaceLocationFlags")
        
        -- Skip untracked bones
        if flags and (flags.position_tracked == false or flags.rotation_tracked == false) then
            goto continue_update
        end
        
        -- Skip Palm and Wrist (handled by root positioning)
        if hand_bone == "Palm" or hand_bone == "Wrist" then
            goto continue_update
        end
        
        -- Find this bone's parent in the XR hierarchy
        local parent_bone_name = VrHands.XR_BONE_PARENTS[hand_bone]
        if not parent_bone_name then
            goto continue_update
        end
        
        local parent_entity = xr_bone_lookup[parent_bone_name]
        if not parent_entity then
            goto continue_update
        end
        
        -- Get mesh bone for this XR bone
        local mesh_bone_name = self.bone_map[hand_bone]
        if not mesh_bone_name then
            goto continue_update
        end
        
        local mesh_bone_id = self.mesh_bones[mesh_bone_name]
        if not mesh_bone_id then
            goto continue_update
        end
        
        -- Get XR parent and child world rotations
        local parent_world_rot = world:call_component_method(parent_entity:id(), "GlobalTransform", "rotation")
        local child_world_rot = world:call_component_method(bone_entity:id(), "GlobalTransform", "rotation")
        
        if not parent_world_rot or not child_world_rot then
            goto continue_update
        end
        
        -- Compute local rotation: local = inv(parent) * child
        local inv_parent = world:call_static_method("Quat", "inverse", parent_world_rot)
        local xr_current_local = world:call_static_method("Quat", "mul_quat", inv_parent, child_world_rot)
        
        -- Get XR reference local rotation (captured at rest)
        local xr_ref_local = self.xr_reference_rotations[hand_bone]
        if not xr_ref_local then
            goto continue_update  -- Can't compute delta without reference
        end
        
        -- Compute rotation delta from reference: delta = current * inv(ref)
        local inv_ref = world:call_static_method("Quat", "inverse", xr_ref_local)
        local xr_delta = world:call_static_method("Quat", "mul_quat", xr_current_local, inv_ref)
        
        -- Get mesh bone's initial rotation
        local mesh_initial_rot = self.mesh_initial_rotations[mesh_bone_id]
        if not mesh_initial_rot then
            goto continue_update
        end
        
        -- Transform XR delta using grip offset (same transform used for wrist)
        local transformed_delta = xr_delta
        
        if self.grip_offset_quat then
            -- Apply grip offset: transformed = grip_offset * delta * inv(grip_offset)
            local inv_grip = world:call_static_method("Quat", "inverse", self.grip_offset_quat)
            local temp = world:call_static_method("Quat", "mul_quat", self.grip_offset_quat, xr_delta)
            transformed_delta = world:call_static_method("Quat", "mul_quat", temp, inv_grip)
        end
        
        -- For left hand: negate Y and Z to mirror the rotation for X-scale flip
        if self.hand == "left" then
            transformed_delta = {
                x = transformed_delta.x,
                y = -transformed_delta.y,
                z = -transformed_delta.z,
                w = transformed_delta.w
            }
        end
        
        -- Apply: final = mesh_initial * transformed_delta
        local final_rotation = world:call_static_method("Quat", "mul_quat", 
            mesh_initial_rot, transformed_delta)
        
        -- Apply to mesh bone
        local mesh_bone = world:get_entity(mesh_bone_id)
        if mesh_bone then
            mesh_bone:set({
                Transform = {
                    rotation = final_rotation
                }
            })
        end
        
        ::continue_update::
    end
end

--------------------------------------------------------------------------------
-- Public: Update (call each frame)
--------------------------------------------------------------------------------

function VrHands:update(world)
    if not self.root_entity then return end
    
    -- Build bone cache if not done
    if not self.bones_cached then
        if not self:_build_bone_cache(world) then
            return  -- Scene not loaded yet
        end
    end
    
    -- Query XR hand bones for this hand
    local xr_bones = nil
    if self.hand == "left" then
        xr_bones = world:query({"LeftHand", "HandBone", "GlobalTransform", "XrSpaceLocationFlags"}, nil)
    else
        xr_bones = world:query({"RightHand", "HandBone", "GlobalTransform", "XrSpaceLocationFlags"}, nil)
    end
    
    if not xr_bones or #xr_bones == 0 then
        -- Fallback: try generic HandBone query
        local all_bones = world:query({"HandBone", "GlobalTransform", "XrSpaceLocationFlags"}, nil)
        if all_bones and #all_bones > 0 then
            xr_bones = all_bones  -- Use all bones as fallback
        else
            if not self._init_logged then
                print(string.format("[VR_HANDS] %s hand: waiting for XR hand tracking...", self.hand))
                self._init_logged = true
            end
            return
        end
    end
    
    -- Find Palm entity for root positioning
    local palm_entity_id = nil
    for _, bone_entity in ipairs(xr_bones) do
        local hand_bone = bone_entity:get("HandBone")
        local flags = bone_entity:get("XrSpaceLocationFlags")
        
        if flags and (flags.position_tracked == false or flags.rotation_tracked == false) then
            goto continue_palm
        end
        
        if hand_bone == "Wrist" then
            palm_entity_id = bone_entity:id()
        end

        if self.hand == "right" and (hand_bone == "Palm" or hand_bone == "Wrist") then
            local rotation = world:call_component_method(bone_entity:id(), "GlobalTransform", "rotation")
            -- print(string.format("[VR_HANDS] %s hand: (%s) rotation: %s", self.hand, hand_bone, rotation.x))
        end
        
        ::continue_palm::
    end
    
    -- Capture mesh offset if not done (needs scene to be loaded)
    if not self.mesh_offset_captured then
        self:_capture_mesh_offset(world)
    end
    
    -- Update root transform
    if palm_entity_id then
        local root_mesh = world:get_entity(self.root_entity)
        if root_mesh then
            local translation = world:call_component_method(palm_entity_id, "GlobalTransform", "translation")
            local rotation = world:call_component_method(palm_entity_id, "GlobalTransform", "rotation")
            
            if translation and rotation then
                -- Compute grip offset quaternion if not yet done
                if not self.grip_offset_quat and self.grip_offset_degrees then
                    self.grip_offset_quat = world:call_static_method("Quat", "from_euler", 
                        "XYZ",
                        math.rad(self.grip_offset_degrees.x),
                        math.rad(self.grip_offset_degrees.y),
                        math.rad(self.grip_offset_degrees.z))
                    print(string.format("[VR_HANDS] %s hand: grip offset (%.0f, %.0f, %.0f) degrees",
                        self.hand,
                        self.grip_offset_degrees.x,
                        self.grip_offset_degrees.y,
                        self.grip_offset_degrees.z))
                end
                
                -- Apply grip offset to XR rotation first (compensate for controller angle)
                local adjusted_rotation = rotation
                if self.grip_offset_quat then
                    adjusted_rotation = world:call_static_method("Quat", "mul_quat", 
                        rotation, self.grip_offset_quat)
                end
                
                -- Then apply mesh coordinate system compensation
                local final_rotation = adjusted_rotation
                if self.mesh_offset_captured and self.mesh_initial_rotation then
                    final_rotation = world:call_static_method("Quat", "mul_quat", 
                        adjusted_rotation, self.mesh_initial_rotation)
                end
                
                -- Determine scale
                local scale_x = self.scale
                if self.hand == "left" then
                    scale_x = -self.scale
                end
                
                -- Compute adjusted translation to compensate for mesh offset
                local adjusted_translation = translation
                if self.mesh_offset_captured and self.mesh_initial_offset then
                    -- Rotate the offset by the final rotation to transform it to world space
                    local rotated_offset = world:call_static_method("Quat", "mul_vec3", final_rotation, {
                        x = self.mesh_initial_offset.x * math.abs(scale_x),
                        y = self.mesh_initial_offset.y * self.scale,
                        z = self.mesh_initial_offset.z * self.scale
                    })
                    
                    if rotated_offset then
                        adjusted_translation = {
                            x = translation.x - rotated_offset.x,
                            y = translation.y - rotated_offset.y,
                            z = translation.z - rotated_offset.z
                        }
                    end
                end
                
                root_mesh:set({
                    Transform = {
                        translation = adjusted_translation,
                        rotation = final_rotation,
                        scale = { x = scale_x, y = self.scale, z = self.scale }
                    }
                })
            end
        end
    end
    
    -- Capture reference pose if not done
    if not self.reference_captured then
        self:_capture_reference(world, xr_bones)
    end
    
    -- Update finger bones
    self:_update_finger_bones(world, xr_bones)
end

--------------------------------------------------------------------------------
-- Public: Get Pose (for network replication)
-- Returns all data needed to replicate this hand to another client
--------------------------------------------------------------------------------

function VrHands:get_pose(world)
    if not self.root_entity then return nil end
    
    local root = world:get_entity(self.root_entity)
    if not root then return nil end
    
    local transform = root:get("Transform")
    if not transform then return nil end
    
    -- Collect bone rotations
    local bone_rotations = {}
    for bone_name, bone_entity_id in pairs(self.mesh_bones) do
        local bone = world:get_entity(bone_entity_id)
        if bone then
            local bone_transform = bone:get("Transform")
            if bone_transform and bone_transform.rotation then
                bone_rotations[bone_name] = bone_transform.rotation
            end
        end
    end
    
    return {
        hand = self.hand,
        transform = transform,
        bones = bone_rotations
    }
end

--------------------------------------------------------------------------------
-- Public: Set Pose (for receiving remote hand data)
-- Applies pose received from network
--------------------------------------------------------------------------------

function VrHands:set_pose(world, pose)
    if not self.root_entity or not pose then return end
    
    local root = world:get_entity(self.root_entity)
    if not root then return end
    
    -- Apply root transform
    if pose.transform then
        root:set({ Transform = pose.transform })
    end
    
    -- Apply bone rotations
    if pose.bones then
        for bone_name, rotation in pairs(pose.bones) do
            local bone_entity_id = self.mesh_bones[bone_name]
            if bone_entity_id then
                local bone = world:get_entity(bone_entity_id)
                if bone then
                    bone:set({ Transform = { rotation = rotation } })
                end
            end
        end
    end
end

--------------------------------------------------------------------------------
-- Public: Cleanup
--------------------------------------------------------------------------------

function VrHands:cleanup()
    if self.root_entity then
        despawn(self.root_entity)
        print(string.format("[VR_HANDS] %s hand: cleaned up", self.hand))
    end
    
    self.root_entity = nil
    self.mesh_bones = {}
    self.mesh_initial_rotations = {}
    self.xr_reference_rotations = {}
    self.bones_cached = false
    self.reference_captured = false
end

return VrHands
