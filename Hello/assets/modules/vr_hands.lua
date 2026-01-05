-- VR Hand Mesh Module
-- Loads glTF hand meshes and poses them using XR hand tracking data
-- Directly copies transforms from OpenXR HandBone entities to mesh bones
--
-- Usage:
--   local VrHands = require("modules/vr_hands.lua")
--   VrHands.init()  -- Load hand meshes
--   register_system("Update", function(world)
--       VrHands.update(world)  -- Pose hands each frame
--   end)

local VrHands = {}

-- Configuration
local HAND_MODEL_PATH = "vr_hand/scene.gltf#Scene0"
local HAND_SCALE = 0.13  -- Scale to match XR coordinate space

-- State
local initialized = false
local left_hand_root = nil   -- Entity ID of left hand scene root
local right_hand_root = nil  -- Entity ID of right hand scene root

-- Rotation offsets to align hand mesh with XR tracking
-- Computed lazily on first update when world is available
local LEFT_HAND_OFFSET = nil
local RIGHT_HAND_OFFSET = nil

-- Bone name mapping: OpenXR HandBone enum -> glTF joint name
local BONE_NAME_MAP = {
    -- Palm and Wrist
    ["Palm"] = "HANDPALM_joint_02",
    ["Wrist"] = "Root_joint_01",
    
    -- Thumb
    ["ThumbMetacarpal"] = "THUMB_BASE_joint_019",
    ["ThumbProximal"] = "THUMB_MID_joint_020",
    ["ThumbDistal"] = "THUMB_TOP_joint_021",
    ["ThumbTip"] = "THUMB_UP_TOP_joint_022",
    
    -- Index finger
    ["IndexMetacarpal"] = "INDEX_BASE_joint_03",
    ["IndexProximal"] = "INDEX_MID_joint_04",
    ["IndexIntermediate"] = "INDEX_TOP_joint_05",
    ["IndexDistal"] = "INDEX_UP_TOP_joint_00",
    ["IndexTip"] = "INDEX_UP_TOP_joint_00",
    
    -- Middle finger
    ["MiddleMetacarpal"] = "MIDDLE_F_BASE_joint_07",
    ["MiddleProximal"] = "MIDDLE_F_MID_joint_08",
    ["MiddleIntermediate"] = "MIDDLE_F_TOP_joint_09",
    ["MiddleDistal"] = "MIDDLE_F_UP_TOP_joint_010",
    ["MiddleTip"] = "MIDDLE_F_UP_TOP_joint_010",
    
    -- Ring finger
    ["RingMetacarpal"] = "RING_BASE_joint_011",
    ["RingProximal"] = "RING_MID_joint_012",
    ["RingIntermediate"] = "RING_TOP_joint_013",
    ["RingDistal"] = "RING_UP_TOP_joint_014",
    ["RingTip"] = "RING_UP_TOP_joint_014",
    
    -- Little/Pinky finger
    ["LittleMetacarpal"] = "PINK_BASE_joint_015",
    ["LittleProximal"] = "PINK_MID_joint_016",
    ["LittleIntermediate"] = "PINK_TOP_joint_017",
    ["LittleDistal"] = "PINK_UP_TOP_joint_018",
    ["LittleTip"] = "PINK_UP_TOP_joint_018",
}

-- Cache of mesh bone entity IDs by hand and glTF name
-- Structure: mesh_bone_cache["left"]["THUMB_BASE_joint_019"] = entity_id
-- Structure: mesh_bone_cache["right"]["THUMB_BASE_joint_019"] = entity_id
local mesh_bone_cache = { left = {}, right = {} }
local bone_cache_built = false

-- Initial rotation caches for delta-based finger tracking
-- Store palm-relative (local) rotations to isolate finger movement from hand movement
local xr_bone_initial_local_rotations = {}  -- "hand_boneName" -> initial palm-relative rotation
local xr_palm_initial_rotations = {}        -- "left"/"right" -> initial palm global rotation
local mesh_bone_initial_rotations = {}      -- Entity ID -> initial local rotation
local xr_initial_rotations_cached = false

--- Initialize hand meshes
function VrHands.init()
    if initialized then return end
    
    print("[VR_HANDS] Initializing hand mesh system...")
    
    local left_handle = load_asset(HAND_MODEL_PATH)
    local right_handle = load_asset(HAND_MODEL_PATH)
    
    -- Spawn left hand (mirrored on X axis)
    local left_entity = spawn({
        SceneRoot = { id = left_handle },
        Transform = {
            translation = { x = 0, y = 0, z = 0 },
            rotation = { x = 0, y = 0, z = 0, w = 1 },
            scale = { x = -HAND_SCALE, y = HAND_SCALE, z = HAND_SCALE }
        },
        VrHandMesh = { hand = "left" }
    })
    left_hand_root = left_entity:id()
    print("[VR_HANDS] Spawned left hand mesh:", left_hand_root)
    
    -- Spawn right hand
    local right_entity = spawn({
        SceneRoot = { id = right_handle },
        Transform = {
            translation = { x = 0, y = 0, z = 0 },
            rotation = { x = 0, y = 0, z = 0, w = 1 },
            scale = { x = HAND_SCALE, y = HAND_SCALE, z = HAND_SCALE }
        },
        VrHandMesh = { hand = "right" }
    })
    right_hand_root = right_entity:id()
    print("[VR_HANDS] Spawned right hand mesh:", right_hand_root)
    
    initialized = true
    print("[VR_HANDS] Hand mesh system initialized")
end

--- Build cache of mesh bone entities by sorting IDs (Left spawned first -> Lower IDs)
local function build_mesh_bone_cache(world)
    if bone_cache_built then return end
    
    local named_entities = world:query({"Name", "Transform"}, nil)
    if not named_entities or #named_entities == 0 then
        return
    end
    
    print(string.format("[VR_HANDS] Building mesh bone cache from %d named entities...", #named_entities))
    
    -- Temporary storage: bone_name -> list of {id, transform}
    local found_bones = {}
    
    for _, entity in ipairs(named_entities) do
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
                
                -- Check if this is a bone we care about
                for xr_bone_name, gltf_name in pairs(BONE_NAME_MAP) do
                    if bone_name == gltf_name then
                        local entity_id = entity:id()
                        found_bones[gltf_name] = found_bones[gltf_name] or {}
                        
                        -- Store ID and initial rotation
                        local transform = entity:get("Transform")
                        local rotation = (transform and transform.rotation) and {
                            x = transform.rotation.x or 0,
                            y = transform.rotation.y or 0,
                            z = transform.rotation.z or 0,
                            w = transform.rotation.w or 1
                        } or { x = 0, y = 0, z = 0, w = 1 }
                        
                        table.insert(found_bones[gltf_name], { id = entity_id, rot = rotation })
                        break
                    end
                end
            end
        end
    end
    
    -- Process found bones: Sort by ID and assign to Left/Right
    mesh_bone_cache = { left = {}, right = {} }
    local left_count = 0
    local right_count = 0
    
    for gltf_name, instances in pairs(found_bones) do
        -- Sort by Entity ID (heuristic: Left spawned first -> lower ID)
        -- Note: Entity IDs are userdata or numbers, assuming comparable
        table.sort(instances, function(a, b) 
            -- helper to handle userdata ID comparison if needed, though Lua handles simple types
            return tostring(a.id) < tostring(b.id) 
        end)
        
        -- First instance -> Left
        if instances[1] then
            local data = instances[1]
            mesh_bone_cache.left[gltf_name] = data.id
            mesh_bone_initial_rotations[data.id] = data.rot
            left_count = left_count + 1
        end
        
        -- Second instance -> Right
        if instances[2] then
            local data = instances[2]
            mesh_bone_cache.right[gltf_name] = data.id
            mesh_bone_initial_rotations[data.id] = data.rot
            right_count = right_count + 1
        end
        
        if #instances > 2 then
            print(string.format("[VR_HANDS] Warning: Found %d instances of bone %s (expected 2)", #instances, gltf_name))
        end
    end
    
    print(string.format("[VR_HANDS] Mesh bone cache built: left=%d right=%d bones", left_count, right_count))
    
    if left_count > 0 and right_count > 0 then
        bone_cache_built = true
    end
end

--- Cache initial XR bone rotations relative to palm (rest pose)
local function cache_xr_initial_rotations(world, hand_bones, hand_label)
    if not hand_bones or #hand_bones == 0 then return false end
    
    -- First, find the palm rotation for this hand
    local palm_rotation = nil
    for _, bone_entity in ipairs(hand_bones) do
        local hand_bone = bone_entity:get("HandBone")
        local flags = bone_entity:get("XrSpaceLocationFlags")
        if flags and (flags.position_tracked == false or flags.rotation_tracked == false) then
            goto continue_palm
        end
        if hand_bone == "Palm" then
            palm_rotation = world:call_component_method(bone_entity:id(), "GlobalTransform", "rotation")
            break
        end
        ::continue_palm::
    end
    
    if not palm_rotation then
        return false  -- Can't cache without palm reference
    end
    
    -- Cache palm rotation and inverse for this hand
    xr_palm_initial_rotations[hand_label] = palm_rotation
    
    -- Now cache each bone's rotation relative to palm
    local inv_palm = world:call_static_method("Quat", "inverse", palm_rotation)
    local cached_count = 0
    
    for _, bone_entity in ipairs(hand_bones) do
        local hand_bone = bone_entity:get("HandBone")
        local flags = bone_entity:get("XrSpaceLocationFlags")
        
        -- Only cache if tracked
        if flags and (flags.position_tracked == false or flags.rotation_tracked == false) then
            goto continue_cache
        end
        
        if hand_bone and hand_bone ~= "Palm" and hand_bone ~= "Wrist" then
            local bone_rotation = world:call_component_method(bone_entity:id(), "GlobalTransform", "rotation")
            if bone_rotation then
                -- Compute rotation relative to palm: local_rot = inverse(palm) * bone
                local local_rot = world:call_static_method("Quat", "mul_quat", inv_palm, bone_rotation)
                local key = hand_label .. "_" .. hand_bone
                xr_bone_initial_local_rotations[key] = local_rot
                cached_count = cached_count + 1
            end
        end
        
        ::continue_cache::
    end
    
    return cached_count > 0
end

--- Update hand mesh poses from XR hand tracking
function VrHands.update(world)
    if not initialized then return end
    
    -- Initialize rotation offsets on first update (requires world context)
    if LEFT_HAND_OFFSET == nil then
        LEFT_HAND_OFFSET = world:call_static_method("Quat", "from_euler", "XYZ", math.rad(-90), math.rad(-135), math.rad(0))
        RIGHT_HAND_OFFSET = world:call_static_method("Quat", "from_euler", "XYZ", math.rad(-90), math.rad(135), math.rad(0))
    end
    
    -- Build mesh bone cache if not done yet
    if not bone_cache_built then
        build_mesh_bone_cache(world)
        if not bone_cache_built then
            return  -- Scene not loaded yet
        end
    end
    
    -- Query left hand bones (with LeftHand marker)
    local left_bones = world:query({"LeftHand", "HandBone", "GlobalTransform", "XrSpaceLocationFlags"}, nil)
    -- Query right hand bones (with RightHand marker)
    local right_bones = world:query({"RightHand", "HandBone", "GlobalTransform", "XrSpaceLocationFlags"}, nil)
    
    -- Fallback: Query all HandBone entities if specific hand queries fail
    local all_bones = nil
    if (not left_bones or #left_bones == 0) and (not right_bones or #right_bones == 0) then
        all_bones = world:query({"HandBone", "GlobalTransform", "XrSpaceLocationFlags"}, nil)
        if not all_bones or #all_bones == 0 then
            if not VrHands._no_bones_printed then
                print("[VR_HANDS] No XR HandBone entities found - hand tracking may not be active")
                VrHands._no_bones_printed = true
            end
            return
        end
    end
    
    if not VrHands._bones_found_printed then
        local left_count = left_bones and #left_bones or 0
        local right_count = right_bones and #right_bones or 0
        local all_count = all_bones and #all_bones or 0
        print(string.format("[VR_HANDS] Found bones - left:%d right:%d all:%d", left_count, right_count, all_count))
        VrHands._bones_found_printed = true
    end
    
    -- Find Palm entity IDs for each hand (we'll extract transforms via call_component_method)
    local left_palm_entity_id = nil
    local right_palm_entity_id = nil
    
    -- Process left hand - find Palm for root positioning
    if left_bones and #left_bones > 0 then
        for _, bone_entity in ipairs(left_bones) do
            local hand_bone = bone_entity:get("HandBone")
            local flags = bone_entity:get("XrSpaceLocationFlags")
            
            -- Check tracking flags
            if flags and (flags.position_tracked == false or flags.rotation_tracked == false) then
                goto continue_left
            end
            
            if hand_bone == "Palm" or hand_bone == "Wrist" then
                left_palm_entity_id = bone_entity:id()
                -- Prefer Palm, but accept Wrist as fallback
                if hand_bone == "Palm" then
                    break
                end
            end
            
            ::continue_left::
        end
    end
    
    -- Process right hand - find Palm for root positioning  
    if right_bones and #right_bones > 0 then
        for _, bone_entity in ipairs(right_bones) do
            local hand_bone = bone_entity:get("HandBone")
            local flags = bone_entity:get("XrSpaceLocationFlags")
            
            -- Check tracking flags
            if flags and (flags.position_tracked == false or flags.rotation_tracked == false) then
                goto continue_right
            end
            
            if hand_bone == "Palm" or hand_bone == "Wrist" then
                right_palm_entity_id = bone_entity:id()
                -- Prefer Palm, but accept Wrist as fallback
                if hand_bone == "Palm" then
                    break
                end
            end
            
            ::continue_right::
        end
    end
    
    -- Fallback: Use all_bones if specific hand queries failed
    if all_bones and #all_bones > 0 and not left_palm_entity_id and not right_palm_entity_id then
        -- Just use first Palm we find to test
        for _, bone_entity in ipairs(all_bones) do
            local hand_bone = bone_entity:get("HandBone")
            if hand_bone == "Palm" then
                local entity_id = bone_entity:id()
                if not left_palm_entity_id then
                    left_palm_entity_id = entity_id
                elseif not right_palm_entity_id then
                    right_palm_entity_id = entity_id
                end
                if left_palm_entity_id and right_palm_entity_id then
                    break
                end
            end
        end
    end
    
    -- Update left hand mesh root transform
    if left_palm_entity_id and left_hand_root then
        local left_mesh = world:get_entity(left_hand_root)
        if left_mesh then
            local translation = world:call_component_method(left_palm_entity_id, "GlobalTransform", "translation")
            local rotation = world:call_component_method(left_palm_entity_id, "GlobalTransform", "rotation")
            if translation then
                -- Apply rotation offset to align mesh with XR tracking
                local final_rotation = world:call_static_method("Quat", "mul_quat", rotation, LEFT_HAND_OFFSET)
                
                if not VrHands._left_palm_printed then
                    local rot_str = final_rotation and string.format("%.2f,%.2f,%.2f,%.2f", 
                        final_rotation.x or 0, final_rotation.y or 0, final_rotation.z or 0, final_rotation.w or 0) or "nil"
                    print(string.format("[VR_HANDS] Left palm at: %.2f, %.2f, %.2f rot: %s", 
                        translation.x or 0, translation.y or 0, translation.z or 0, rot_str))
                    VrHands._left_palm_printed = true
                end
                left_mesh:set({
                    Transform = {
                        translation = translation,
                        rotation = final_rotation,
                        scale = { x = -HAND_SCALE, y = HAND_SCALE, z = HAND_SCALE }  -- Mirrored
                    }
                })
            end
        end
    end



    
    -- Update right hand mesh root transform
    if right_palm_entity_id and right_hand_root then
        local right_mesh = world:get_entity(right_hand_root)
        if right_mesh then
            local translation = world:call_component_method(right_palm_entity_id, "GlobalTransform", "translation")
            local rotation = world:call_component_method(right_palm_entity_id, "GlobalTransform", "rotation")
            if translation then
                -- Apply rotation offset to align mesh with XR tracking
                local final_rotation = world:call_static_method("Quat", "mul_quat", rotation, RIGHT_HAND_OFFSET)
                
                right_mesh:set({
                    Transform = {
                        translation = translation,
                        rotation = final_rotation,
                        scale = { x = HAND_SCALE, y = HAND_SCALE, z = HAND_SCALE }
                    }
                })
            end
        end
    end
    
    -- Cache initial XR bone rotations on first frame with tracking
    if not xr_initial_rotations_cached then
        local left_cached = cache_xr_initial_rotations(world, left_bones, "left")
        local right_cached = cache_xr_initial_rotations(world, right_bones, "right")
        if left_cached or right_cached then
            local count = 0
            for _ in pairs(xr_bone_initial_local_rotations) do count = count + 1 end
            print(string.format("[VR_HANDS] Cached %d initial XR bone local rotations (palm-relative)", count))
            xr_initial_rotations_cached = true
        end
    end
    
    -- Update finger bone rotations using palm-relative rotation deltas
    local function update_finger_bones(hand_bones, hand_label)
        if not hand_bones or #hand_bones == 0 then return end
        if not xr_initial_rotations_cached then return end
        
        -- Find current palm rotation for this hand
        local current_palm_rot = nil
        for _, bone_entity in ipairs(hand_bones) do
            local hand_bone = bone_entity:get("HandBone")
            if hand_bone == "Palm" then
                current_palm_rot = world:call_component_method(bone_entity:id(), "GlobalTransform", "rotation")
                break
            end
        end
        
        if not current_palm_rot then return end
        
        local inv_current_palm = world:call_static_method("Quat", "inverse", current_palm_rot)
        
        for _, bone_entity in ipairs(hand_bones) do
            local hand_bone = bone_entity:get("HandBone")
            local flags = bone_entity:get("XrSpaceLocationFlags")
            
            -- Skip if not tracked
            if flags and (flags.position_tracked == false or flags.rotation_tracked == false) then
                goto continue_bone
            end
            
            -- Skip Palm and Wrist - they're handled by root positioning
            if hand_bone == "Palm" or hand_bone == "Wrist" then
                goto continue_bone
            end
            
            -- Get initial palm-relative rotation for this bone
            local key = hand_label .. "_" .. hand_bone
            local initial_local_rot = xr_bone_initial_local_rotations[key]
            if not initial_local_rot then
                goto continue_bone
            end
            
            -- Get current bone rotation and compute current palm-relative rotation
            local current_bone_rot = world:call_component_method(bone_entity:id(), "GlobalTransform", "rotation")
            if not current_bone_rot then
                goto continue_bone
            end
            
            -- Current local rotation = inverse(current_palm) * current_bone
            local current_local_rot = world:call_static_method("Quat", "mul_quat", inv_current_palm, current_bone_rot)
            
            -- Compute rotation delta in palm-local space
            -- This isolates finger movement from hand movement
            -- delta = inverse(initial_local) * current_local
            local inv_initial_local = world:call_static_method("Quat", "inverse", initial_local_rot)
            local delta = world:call_static_method("Quat", "mul_quat", inv_initial_local, current_local_rot)
            
            -- Find matching mesh bone for THIS hand and apply delta to its initial rotation
            local gltf_bone_name = BONE_NAME_MAP[hand_bone]
            if gltf_bone_name then
                local mesh_bone_id = mesh_bone_cache[hand_label][gltf_bone_name]
                if mesh_bone_id then
                    local initial_mesh_rot = mesh_bone_initial_rotations[mesh_bone_id]
                    if initial_mesh_rot then
                        -- Apply delta to mesh bone's initial rotation
                        -- new_rotation = initial_mesh_rotation * delta
                        local new_rotation = world:call_static_method("Quat", "mul_quat", initial_mesh_rot, delta)
                        
                        local mesh_bone = world:get_entity(mesh_bone_id)
                        if mesh_bone then
                            mesh_bone:set({
                                Transform = {
                                    rotation = new_rotation
                                }
                            })
                        end
                    end
                end
            end
            
            ::continue_bone::
        end
    end
    
    -- Update finger bones for both hands
    -- update_finger_bones(left_bones, "left")
    -- update_finger_bones(right_bones, "right")
end




--- Cleanup hand mesh entities

function VrHands.cleanup()
    if left_hand_root then
        despawn(left_hand_root)
        left_hand_root = nil
    end
    if right_hand_root then
        despawn(right_hand_root)
        right_hand_root = nil
    end
    mesh_bone_cache = { left = {}, right = {} }
    mesh_bone_initial_rotations = {}
    xr_bone_initial_local_rotations = {}
    xr_palm_initial_rotations = {}
    bone_cache_built = false
    xr_initial_rotations_cached = false
    initialized = false
    print("[VR_HANDS] Cleaned up hand meshes")
end

return VrHands
