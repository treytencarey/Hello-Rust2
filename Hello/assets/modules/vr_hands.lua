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

-- Helper: Extract translation from GlobalTransform using component method
-- world must be passed in since this is a helper function
local function extract_translation(world, entity_id)
    return world:call_component_method(entity_id, "GlobalTransform", "translation")
end

-- Helper: Extract rotation from GlobalTransform using component method
local function extract_rotation(world, entity_id)
    return world:call_component_method(entity_id, "GlobalTransform", "rotation")
end

-- Helper: Quaternion multiplication (q1 * q2)
-- Kept in Lua as we need to combine rotations without entity context
local function quat_multiply(q1, q2)
    if not q1 or not q2 then return nil end
    return {
        x = q1.w * q2.x + q1.x * q2.w + q1.y * q2.z - q1.z * q2.y,
        y = q1.w * q2.y - q1.x * q2.z + q1.y * q2.w + q1.z * q2.x,
        z = q1.w * q2.z + q1.x * q2.y - q1.y * q2.x + q1.z * q2.w,
        w = q1.w * q2.w - q1.x * q2.x - q1.y * q2.y - q1.z * q2.z
    }
end

-- Helper: Create quaternion from Euler angles (degrees) - XYZ order
-- Uses math directly since these are computed at module load time (no world context)
local function quat_from_euler(x_deg, y_deg, z_deg)
    local x, y, z = math.rad(x_deg), math.rad(y_deg), math.rad(z_deg)
    local cx, sx = math.cos(x * 0.5), math.sin(x * 0.5)
    local cy, sy = math.cos(y * 0.5), math.sin(y * 0.5)
    local cz, sz = math.cos(z * 0.5), math.sin(z * 0.5)
    
    -- XYZ extrinsic rotation order (equivalent to ZYX intrinsic)
    return {
        x = sx * cy * cz + cx * sy * sz,
        y = cx * sy * cz - sx * cy * sz,
        z = cx * cy * sz + sx * sy * cz,
        w = cx * cy * cz - sx * sy * sz
    }
end

-- Rotation offsets to align hand mesh with XR tracking
local LEFT_HAND_OFFSET = quat_from_euler(-70, 10, 160)
local RIGHT_HAND_OFFSET = quat_from_euler(-70, -10, 210)

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

-- Cache of mesh bone entity IDs by glTF name
local mesh_bone_cache = {}
local bone_cache_built = false

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

--- Build cache of mesh bone entities by name
local function build_mesh_bone_cache(world)
    if bone_cache_built then return end
    
    local named_entities = world:query({"Name"}, nil)
    if not named_entities or #named_entities == 0 then
        return
    end
    
    print(string.format("[VR_HANDS] Building mesh bone cache from %d named entities...", #named_entities))
    
    for _, entity in ipairs(named_entities) do
        local name_component = entity:get("Name")
        if name_component then
            local bone_name = nil
            if type(name_component) == "string" then
                bone_name = name_component
            elseif type(name_component) == "table" then
                bone_name = name_component.name
            end
            
            -- Strip embedded quotes
            if bone_name then
                bone_name = bone_name:gsub('^"', ''):gsub('"$', '')
            end
            
            -- Check if this is a bone we care about
            if bone_name then
                for xr_bone_name, gltf_name in pairs(BONE_NAME_MAP) do
                    if bone_name == gltf_name then
                        mesh_bone_cache[gltf_name] = mesh_bone_cache[gltf_name] or {}
                        table.insert(mesh_bone_cache[gltf_name], entity:id())
                    end
                end
            end
        end
    end
    
    local count = 0
    for _ in pairs(mesh_bone_cache) do count = count + 1 end
    print(string.format("[VR_HANDS] Mesh bone cache built: %d unique bone types", count))
    
    if count > 0 then
        bone_cache_built = true
    end
end

--- Update hand mesh poses from XR hand tracking
function VrHands.update(world)
    if not initialized then return end
    
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
            local translation = extract_translation(world, left_palm_entity_id)
            local rotation = extract_rotation(world, left_palm_entity_id)
            if translation then
                -- Apply rotation offset to align mesh with XR tracking
                local final_rotation = quat_multiply(rotation, LEFT_HAND_OFFSET) or LEFT_HAND_OFFSET
                
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
            local translation = extract_translation(world, right_palm_entity_id)
            local rotation = extract_rotation(world, right_palm_entity_id)
            if translation then
                -- Apply rotation offset to align mesh with XR tracking
                local final_rotation = quat_multiply(rotation, RIGHT_HAND_OFFSET) or RIGHT_HAND_OFFSET
                
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
    
    -- Update finger bone rotations for both hands
    local function update_finger_bones(hand_bones, hand_label, rotation_offset)
        if not hand_bones or #hand_bones == 0 then return end
        
        for _, bone_entity in ipairs(hand_bones) do
            local hand_bone = bone_entity:get("HandBone")
            local global_transform = bone_entity:get("GlobalTransform")
            local flags = bone_entity:get("XrSpaceLocationFlags")
            
            -- Skip if not tracked
            if flags and (flags.position_tracked == false or flags.rotation_tracked == false) then
                goto continue_bone
            end
            
            -- Skip Palm and Wrist - they're handled by root positioning
            if hand_bone == "Palm" or hand_bone == "Wrist" then
                goto continue_bone
            end
            
            -- Find matching mesh bone name
            local gltf_bone_name = BONE_NAME_MAP[hand_bone]
            if gltf_bone_name and mesh_bone_cache[gltf_bone_name] then
                local rotation = extract_rotation(world, bone_entity:id())
                if rotation then
                    -- Apply the same offset as the hand root to align bone rotations
                    local final_rotation = quat_multiply(rotation, rotation_offset)
                    
                    -- Update mesh bones with this name
                    for _, mesh_bone_id in ipairs(mesh_bone_cache[gltf_bone_name]) do
                        local mesh_bone = world:get_entity(mesh_bone_id)
                        if mesh_bone then
                            mesh_bone:set({
                                Transform = {
                                    rotation = final_rotation
                                }
                            })
                        end
                    end
                end
            end
            
            ::continue_bone::
        end
    end
    
    -- DISABLED: Finger tracking causes visual issues - needs proper local rotation calculation
    -- update_finger_bones(left_bones, "left", LEFT_HAND_OFFSET)
    -- update_finger_bones(right_bones, "right", RIGHT_HAND_OFFSET)
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
    mesh_bone_cache = {}
    bone_cache_built = false
    initialized = false
    print("[VR_HANDS] Cleaned up hand meshes")
end

return VrHands
