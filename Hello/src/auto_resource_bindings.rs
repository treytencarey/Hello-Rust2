#[allow(unused_imports)]
use bevy::prelude::*;
pub fn register_auto_resource_bindings(registry: &bevy_lua_ecs::LuaResourceRegistry) {}
#[doc = r" Auto-discovered entity wrapper type names (for runtime TypeRegistry lookup)"]
#[doc = r" These are type names discovered by scanning bevy_* crates for:"]
#[doc = r" `pub struct TypeName(pub Entity)` with `#[derive(Component)]`"]
pub const DISCOVERED_ENTITY_WRAPPERS: &[&str] = &[
    "AccessibilityNode",
    "AnimationTarget",
    "AnimationPlayer",
    "AnimationGraphHandle",
    "AnimationTransitions",
    "ContrastAdaptiveSharpening",
    "DenoiseCas",
    "CasUniform",
    "ViewCasPipeline",
    "ViewDlssRayReconstructionTextures",
    "Fxaa",
    "CameraFxaaPipeline",
    "Smaa",
    "ViewSmaaPipelines",
    "SmaaInfoUniformOffset",
    "SmaaTextures",
    "SmaaBindGroups",
    "TemporalAntiAliasing",
    "TemporalAntiAliasHistoryTextures",
    "TemporalAntiAliasPipelineId",
    "Aabb",
    "Frustum",
    "CubemapFrusta",
    "CascadesFrusta",
    "NoCpuCulling",
    "Visibility",
    "InheritedVisibility",
    "VisibilityClass",
    "ViewVisibility",
    "NoFrustumCulling",
    "VisibleEntities",
    "VisibleMeshEntities",
    "CubemapVisibleEntities",
    "CascadesVisibleEntities",
    "ViewTransmissionTexture",
    "DeferredLightingIdDepthTexture",
    "DepthOfField",
    "DepthOfFieldUniform",
    "DepthOfFieldPipelines",
    "AuxiliaryDepthOfFieldTexture",
    "ViewDepthOfFieldBindGroupLayouts",
    "ViewDepthPyramid",
    "ViewDownsampleDepthBindGroup",
    "MotionBlur",
    "MotionBlurUniform",
    "MotionBlurPipelineId",
    "MsaaWritebackBlitPipeline",
    "OrderIndependentTransparencySettings",
    "OrderIndependentTransparencySettingsOffset",
    "OitResolvePipelineId",
    "ChromaticAberration",
    "PostProcessingPipelineId",
    "PostProcessingUniformBufferOffsets",
    "DepthPrepass",
    "NormalPrepass",
    "MotionVectorPrepass",
    "DeferredPrepass",
    "PreviousViewData",
    "PreviousViewUniformOffset",
    "ViewPrepassTextures",
    "Tonemapping",
    "ViewTonemappingPipeline",
    "DebandDither",
    "ViewUpscalingPipeline",
    "Children",
    "TiledAnimation",
    "TiledImage",
    "TiledLayer",
    "TiledLayerParallax",
    "TiledParallaxCamera",
    "TiledMapLayerZOffset",
    "TiledMapImageRepeatMargin",
    "TiledMapReference",
    "RespawnTiledMap",
    "TiledMapStorage",
    "TiledObjectVisualOf",
    "TiledObjectVisuals",
    "TiledObject",
    "TiledTile",
    "TiledTilemap",
    "RespawnTiledWorld",
    "TiledWorldChunking",
    "TiledWorldStorage",
    "FrustumCulling",
    "TilemapAnchor",
    "TilemapRenderSettings",
    "TilemapId",
    "TilemapSize",
    "TilemapTexture",
    "TilemapTileSize",
    "TilemapGridSize",
    "TilemapSpacing",
    "TilemapTextureSize",
    "TilemapType",
    "TilePos",
    "TileTextureIndex",
    "TileColor",
    "TileVisible",
    "TileFlip",
    "TilePosOld",
    "AnimatedTile",
    "Gizmo",
    "ShowLightGizmo",
    "NotShadowCaster",
    "NotShadowReceiver",
    "TransmittedShadowReceiver",
    "ShadowFilteringMethod",
    "ClusterConfig",
    "Clusters",
    "VisibleClusterableObjects",
    "ClusteredDecal",
    "CascadeShadowConfig",
    "Cascades",
    "MorphWeights",
    "MeshMorphWeights",
    "SkinnedMesh",
    "XrCamera",
    "HandSide",
    "LeftHand",
    "RightHand",
    "XrHandBoneEntities",
    "XrHandBoneRadius",
    "HandBone",
    "XrTrackingRoot",
    "XrTracker",
    "XrSpace",
    "XrVelocity",
    "XrReferenceSpace",
    "XrSpaceLocationFlags",
    "XrSpaceVelocityFlags",
    "Wireframe",
    "WireframeColor",
    "ExtractedWireframeColor",
    "NoWireframe",
    "Mesh3dWireframe",
    "MeshletMesh3d",
    "PbrDeferredLightingDepthId",
    "DeferredLightingPipeline",
    "SkipDeferredLighting",
    "Pickable",
    "PickingInteraction",
    "Hovered",
    "DirectlyHovered",
    "PointerId",
    "PointerInteraction",
    "PointerPress",
    "PointerLocation",
    "SimulationToRenderTime",
    "DefaultRapierContext",
    "RapierContextEntityLink",
    "RapierContextColliders",
    "RapierContextJoints",
    "RapierRigidBodySet",
    "NoAutomaticBatching",
    "OcclusionCulling",
    "OcclusionCullingSubview",
    "OcclusionCullingSubviewEntities",
    "Readback",
    "SyncToRenderWorld",
    "MainEntity",
    "TemporaryRenderEntity",
    "Msaa",
    "ExtractedView",
    "ColorGrading",
    "ViewUniformOffset",
    "ViewTarget",
    "NoIndirectDrawing",
    "ViewDepthTexture",
    "RenderVisibleEntities",
    "Screenshot",
    "Capturing",
    "Captured",
    "CameraRenderGraph",
    "ExtractedCamera",
    "TemporalJitter",
    "MipBias",
    "RenderEntity",
    "Hdr",
    "InteractionDisabled",
    "Pressed",
    "Checkable",
    "Checked",
    "ContentSize",
    "UiAntiAlias",
    "BoxShadowSamples",
    "UiViewTarget",
    "UiBatch",
    "UiShadowsBatch",
    "UiTextureSlicerBatch",
    "TextInputFilter",
    "TextInputBuffer",
    "TextInputPrompt",
    "TextInputStyle",
    "TextInputLayoutInfo",
    "TextInputPromptLayoutInfo",
    "TextInputContents",
    "TextInputQueue",
    "MultiClickData",
    "LuaCustomComponents",
    "LuaObserversAttached",
    "ScriptOwned",
    "LuaTextInput",
    "LuaTextInputValue",
];
#[doc = r" Register entity wrapper components at runtime using TypeRegistry"]
#[doc = r" This looks up each discovered type name in the registry and registers"]
#[doc = r" a handler if it's a valid entity wrapper component"]
#[doc = r" Also registers discovered Component types (like HandBone) as non-reflected components"]
#[doc = r" for Lua queryability when they lack #[reflect(Component)]"]
pub fn register_entity_wrappers_from_registry(
    component_registry: &mut bevy_lua_ecs::ComponentRegistry,
    type_registry: &bevy::ecs::reflect::AppTypeRegistry,
) {
    bevy_lua_ecs::register_entity_wrappers_runtime(
        component_registry,
        type_registry,
        DISCOVERED_ENTITY_WRAPPERS,
    );
    component_registry.register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledAnimation>(
        "TiledAnimation",
    );
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledImage>("TiledImage");
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledLayer>("TiledLayer");
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledLayerParallax>(
            "TiledLayerParallax",
        );
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledParallaxCamera>(
            "TiledParallaxCamera",
        );
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledMapLayerZOffset>(
            "TiledMapLayerZOffset",
        );
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledMapImageRepeatMargin>(
            "TiledMapImageRepeatMargin",
        );
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledMapReference>(
            "TiledMapReference",
        );
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::RespawnTiledMap>(
            "RespawnTiledMap",
        );
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledMapStorage>(
            "TiledMapStorage",
        );
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledObjectVisualOf>(
            "TiledObjectVisualOf",
        );
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledObjectVisuals>(
            "TiledObjectVisuals",
        );
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledObject>("TiledObject");
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledTile>("TiledTile");
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledTilemap>("TiledTilemap");
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::RespawnTiledWorld>(
            "RespawnTiledWorld",
        );
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledWorldChunking>(
            "TiledWorldChunking",
        );
    component_registry
        .register_non_reflected_component::<bevy_ecs_tiled::prelude::TiledWorldStorage>(
            "TiledWorldStorage",
        );
    component_registry
        .register_non_reflected_component::<bevy_mod_xr::camera::XrCamera>("XrCamera");
    component_registry.register_non_reflected_component::<bevy_mod_xr::hands::HandSide>("HandSide");
    component_registry.register_non_reflected_component::<bevy_mod_xr::hands::LeftHand>("LeftHand");
    component_registry
        .register_non_reflected_component::<bevy_mod_xr::hands::RightHand>("RightHand");
    component_registry.register_non_reflected_component::<bevy_mod_xr::hands::XrHandBoneEntities>(
        "XrHandBoneEntities",
    );
    component_registry.register_non_reflected_component::<bevy_mod_xr::hands::XrHandBoneRadius>(
        "XrHandBoneRadius",
    );
    component_registry.register_non_reflected_component::<bevy_mod_xr::hands::HandBone>("HandBone");
    component_registry
        .register_non_reflected_component::<bevy_mod_xr::session::XrTrackingRoot>("XrTrackingRoot");
    component_registry
        .register_non_reflected_component::<bevy_mod_xr::session::XrTracker>("XrTracker");
    component_registry.register_non_reflected_component::<bevy_mod_xr::spaces::XrSpace>("XrSpace");
    component_registry
        .register_non_reflected_component::<bevy_mod_xr::spaces::XrVelocity>("XrVelocity");
    component_registry.register_non_reflected_component::<bevy_mod_xr::spaces::XrReferenceSpace>(
        "XrReferenceSpace",
    );
    component_registry
        .register_non_reflected_component::<bevy_mod_xr::spaces::XrSpaceLocationFlags>(
            "XrSpaceLocationFlags",
        );
    component_registry
        .register_non_reflected_component::<bevy_mod_xr::spaces::XrSpaceVelocityFlags>(
            "XrSpaceVelocityFlags",
        );
    component_registry
        .register_non_reflected_component::<bevy_rapier2d::prelude::SimulationToRenderTime>(
            "SimulationToRenderTime",
        );
    component_registry
        .register_non_reflected_component::<bevy_rapier2d::prelude::DefaultRapierContext>(
            "DefaultRapierContext",
        );
    component_registry
        .register_non_reflected_component::<bevy_rapier2d::prelude::RapierContextEntityLink>(
            "RapierContextEntityLink",
        );
    component_registry
        .register_non_reflected_component::<bevy_rapier2d::prelude::RapierContextColliders>(
            "RapierContextColliders",
        );
    component_registry
        .register_non_reflected_component::<bevy_rapier2d::prelude::RapierContextJoints>(
            "RapierContextJoints",
        );
    component_registry
        .register_non_reflected_component::<bevy_rapier2d::prelude::RapierRigidBodySet>(
            "RapierRigidBodySet",
        );
    component_registry
        .register_non_reflected_component::<bevy_ui_text_input::TextInputFilter>("TextInputFilter");
    component_registry
        .register_non_reflected_component::<bevy_ui_text_input::TextInputBuffer>("TextInputBuffer");
    component_registry
        .register_non_reflected_component::<bevy_ui_text_input::TextInputPrompt>("TextInputPrompt");
    component_registry
        .register_non_reflected_component::<bevy_ui_text_input::TextInputStyle>("TextInputStyle");
    component_registry.register_non_reflected_component::<bevy_ui_text_input::TextInputLayoutInfo>(
        "TextInputLayoutInfo",
    );
    component_registry
        .register_non_reflected_component::<bevy_ui_text_input::TextInputPromptLayoutInfo>(
            "TextInputPromptLayoutInfo",
        );
    component_registry.register_non_reflected_component::<bevy_ui_text_input::TextInputContents>(
        "TextInputContents",
    );
    component_registry
        .register_non_reflected_component::<bevy_ui_text_input::TextInputQueue>("TextInputQueue");
    component_registry
        .register_non_reflected_component::<bevy_ui_text_input::edit::MultiClickData>(
            "MultiClickData",
        );
    component_registry
        .set_non_reflected_serializer(std::sync::Arc::new(serialize_non_reflected_component));
}
#[doc = r" Serialize a non-reflected component to a string using Debug trait"]
#[doc = r" This enables Lua entity:get() to return meaningful values for enum components"]
#[doc = r" like HandBone that don't implement Reflect"]
#[doc = r" "]
#[doc = r" Returns Some(debug_string) if component exists, None otherwise"]
pub fn serialize_non_reflected_component(
    entity_ref: &bevy::ecs::world::EntityRef,
    component_name: &str,
) -> Result<Option<String>, String> {
    match component_name {
        "TiledAnimation" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::TiledAnimation>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledImage" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::TiledImage>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledLayer" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::TiledLayer>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledLayerParallax" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::TiledLayerParallax>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledParallaxCamera" => {
            if let Some(component) =
                entity_ref.get::<bevy_ecs_tiled::prelude::TiledParallaxCamera>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledMapLayerZOffset" => {
            if let Some(component) =
                entity_ref.get::<bevy_ecs_tiled::prelude::TiledMapLayerZOffset>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledMapImageRepeatMargin" => {
            if let Some(component) =
                entity_ref.get::<bevy_ecs_tiled::prelude::TiledMapImageRepeatMargin>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledMapReference" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::TiledMapReference>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "RespawnTiledMap" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::RespawnTiledMap>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledMapStorage" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::TiledMapStorage>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledObjectVisualOf" => {
            if let Some(component) =
                entity_ref.get::<bevy_ecs_tiled::prelude::TiledObjectVisualOf>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledObjectVisuals" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::TiledObjectVisuals>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledObject" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::TiledObject>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledTile" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::TiledTile>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledTilemap" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::TiledTilemap>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "RespawnTiledWorld" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::RespawnTiledWorld>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledWorldChunking" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::TiledWorldChunking>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TiledWorldStorage" => {
            if let Some(component) = entity_ref.get::<bevy_ecs_tiled::prelude::TiledWorldStorage>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "XrCamera" => {
            if let Some(component) = entity_ref.get::<bevy_mod_xr::camera::XrCamera>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "HandSide" => {
            if let Some(component) = entity_ref.get::<bevy_mod_xr::hands::HandSide>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "LeftHand" => {
            if let Some(component) = entity_ref.get::<bevy_mod_xr::hands::LeftHand>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "RightHand" => {
            if let Some(component) = entity_ref.get::<bevy_mod_xr::hands::RightHand>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "XrHandBoneEntities" => {
            if entity_ref.contains::<bevy_mod_xr::hands::XrHandBoneEntities>() {
                Ok(Some("XrHandBoneEntities".to_string()))
            } else {
                Ok(None)
            }
        }
        "XrHandBoneRadius" => {
            if let Some(component) = entity_ref.get::<bevy_mod_xr::hands::XrHandBoneRadius>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "HandBone" => {
            if let Some(component) = entity_ref.get::<bevy_mod_xr::hands::HandBone>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "XrTrackingRoot" => {
            if entity_ref.contains::<bevy_mod_xr::session::XrTrackingRoot>() {
                Ok(Some("XrTrackingRoot".to_string()))
            } else {
                Ok(None)
            }
        }
        "XrTracker" => {
            if let Some(component) = entity_ref.get::<bevy_mod_xr::session::XrTracker>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "XrSpace" => {
            if let Some(component) = entity_ref.get::<bevy_mod_xr::spaces::XrSpace>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "XrVelocity" => {
            if let Some(component) = entity_ref.get::<bevy_mod_xr::spaces::XrVelocity>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "XrReferenceSpace" => {
            if let Some(component) = entity_ref.get::<bevy_mod_xr::spaces::XrReferenceSpace>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "XrSpaceLocationFlags" => {
            if let Some(component) = entity_ref.get::<bevy_mod_xr::spaces::XrSpaceLocationFlags>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "XrSpaceVelocityFlags" => {
            if let Some(component) = entity_ref.get::<bevy_mod_xr::spaces::XrSpaceVelocityFlags>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "SimulationToRenderTime" => {
            if entity_ref.contains::<bevy_rapier2d::prelude::SimulationToRenderTime>() {
                Ok(Some("SimulationToRenderTime".to_string()))
            } else {
                Ok(None)
            }
        }
        "DefaultRapierContext" => {
            if let Some(component) =
                entity_ref.get::<bevy_rapier2d::prelude::DefaultRapierContext>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "RapierContextEntityLink" => {
            if let Some(component) =
                entity_ref.get::<bevy_rapier2d::prelude::RapierContextEntityLink>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "RapierContextColliders" => {
            if let Some(component) =
                entity_ref.get::<bevy_rapier2d::prelude::RapierContextColliders>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "RapierContextJoints" => {
            if let Some(component) = entity_ref.get::<bevy_rapier2d::prelude::RapierContextJoints>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "RapierRigidBodySet" => {
            if entity_ref.contains::<bevy_rapier2d::prelude::RapierRigidBodySet>() {
                Ok(Some("RapierRigidBodySet".to_string()))
            } else {
                Ok(None)
            }
        }
        "TextInputFilter" => {
            if entity_ref.contains::<bevy_ui_text_input::TextInputFilter>() {
                Ok(Some("TextInputFilter".to_string()))
            } else {
                Ok(None)
            }
        }
        "TextInputBuffer" => {
            if let Some(component) = entity_ref.get::<bevy_ui_text_input::TextInputBuffer>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TextInputPrompt" => {
            if let Some(component) = entity_ref.get::<bevy_ui_text_input::TextInputPrompt>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TextInputStyle" => {
            if let Some(component) = entity_ref.get::<bevy_ui_text_input::TextInputStyle>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TextInputLayoutInfo" => {
            if let Some(component) = entity_ref.get::<bevy_ui_text_input::TextInputLayoutInfo>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TextInputPromptLayoutInfo" => {
            if let Some(component) =
                entity_ref.get::<bevy_ui_text_input::TextInputPromptLayoutInfo>()
            {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TextInputContents" => {
            if let Some(component) = entity_ref.get::<bevy_ui_text_input::TextInputContents>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "TextInputQueue" => {
            if let Some(component) = entity_ref.get::<bevy_ui_text_input::TextInputQueue>() {
                let debug_str = format!("{:?}", *component);
                Ok(Some(debug_str))
            } else {
                Ok(None)
            }
        }
        "MultiClickData" => {
            if entity_ref.contains::<bevy_ui_text_input::edit::MultiClickData>() {
                Ok(Some("MultiClickData".to_string()))
            } else {
                Ok(None)
            }
        }
        _ => Err(format!(
            "Unknown non-reflected component: {}",
            component_name
        )),
    }
}
pub fn register_auto_constructors(lua: &mlua::Lua) -> Result<(), mlua::Error> {
    Ok(())
}
#[doc = r" Register all discovered bitflags types with the BitflagsRegistry"]
#[doc = r" Call this in your app's Startup systems to enable generic bitflags handling"]
#[doc = r" Generated from types discovered during asset constructor parsing"]
pub fn register_auto_bitflags(registry: &bevy_lua_ecs::BitflagsRegistry) {
    registry.register(
        "TextureUsages",
        &[
            ("COPY_SRC", 1u32),
            ("COPY_DST", 2u32),
            ("TEXTURE_BINDING", 4u32),
            ("STORAGE_BINDING", 8u32),
            ("RENDER_ATTACHMENT", 16u32),
        ],
    );
}
#[doc = r" Auto-discovered asset type names (for runtime TypeRegistry lookup)"]
#[doc = r" These are type names discovered by scanning bevy_* crates for:"]
#[doc = r" `impl Asset for TypeName` or `#[derive(Asset)] struct TypeName`"]
pub const DISCOVERED_ASSET_TYPES: &[&str] = &[
    "AnimationGraph",
    "AnimationClip",
    "LoadedUntypedAsset",
    "LoadedFolder",
    "AudioSource",
    "Pitch",
    "TiledMapAsset",
    "TiledWorldAsset",
    "StandardTilemapMaterial",
    "GizmoAsset",
    "Gltf",
    "GltfMesh",
    "GltfNode",
    "GltfSkin",
    "GltfPrimitive",
    "Image",
    "TextureAtlasLayout",
    "Mesh",
    "SkinnedMeshInverseBindposes",
    "ForwardDecalMaterialExt",
    "ExtendedMaterial",
    "MeshletMesh",
    "StandardMaterial",
    "WireframeMaterial",
    "AutoExposureCompensationCurve",
    "ShaderStorageBuffer",
    "DynamicScene",
    "Scene",
    "Shader",
    "ColorMaterial",
    "Wireframe2dMaterial",
    "TilemapChunkMaterial",
    "Font",
];
#[doc = r" Register asset types at runtime using TypeRegistry"]
#[doc = r" This looks up each discovered type name in the registry and registers"]
#[doc = r" handlers for valid Asset types (handle setters, asset adders, etc.)"]
pub fn register_asset_types_from_registry(
    asset_registry: &bevy_lua_ecs::AssetRegistry,
    type_registry: &bevy::ecs::reflect::AppTypeRegistry,
) {
    bevy_lua_ecs::register_asset_types_runtime(
        asset_registry,
        type_registry,
        DISCOVERED_ASSET_TYPES,
    );
}
#[doc = r" Register typed path loaders for discovered asset types"]
#[doc = r" This uses compile-time discovered types to call the typed_path_loaders macro"]
#[doc = r" which enables proper Handle<T> loading from asset paths"]
pub fn register_auto_typed_path_loaders(
    asset_registry: &bevy_lua_ecs::AssetRegistry,
    type_registry: &bevy::ecs::reflect::AppTypeRegistry,
) {
    bevy_lua_ecs::register_typed_path_loaders!(
        asset_registry.typed_path_loaders,
        type_registry,
        bevy::animation::AnimationClip,
        bevy::audio::AudioSource,
        bevy::audio::Pitch,
        bevy_ecs_tiled::prelude::TiledMapAsset,
        bevy_ecs_tiled::prelude::TiledWorldAsset,
        bevy::gizmos::GizmoAsset,
        bevy::gltf::Gltf,
        bevy::image::Image,
        bevy::image::TextureAtlasLayout,
        bevy::mesh::Mesh,
        bevy::pbr::StandardMaterial,
        bevy::scene::DynamicScene,
        bevy::scene::Scene,
        bevy::shader::Shader,
        bevy::sprite_render::ColorMaterial,
        bevy::text::Font
    );
}
#[doc = r" Auto-discovered Handle<T> newtype wrappers"]
#[doc = r" Format: (newtype_name, inner_asset_name) - runtime will resolve via TypeRegistry"]
#[doc = r#" Examples: ("ImageRenderTarget", "Image"), ("Mesh3d", "Mesh")"#]
pub const DISCOVERED_NEWTYPE_WRAPPERS: &[(&str, &str)] = &[
    ("AnimationGraphHandle", "AnimationGraph"),
    ("LoadedFolder", "UntypedHandle"),
    ("DownsampleDepthShader", "Shader"),
    ("FullscreenShader", "Shader"),
    ("TiledMap", "TiledMapAsset"),
    ("TiledWorld", "TiledWorldAsset"),
    ("SpotLightTexture", "Image"),
    ("Mesh2d", "Mesh"),
    ("Mesh3d", "Mesh"),
    ("MeshletMesh3d", "MeshletMesh"),
    ("Mesh3dWireframe", "WireframeMaterial"),
    ("Bluenoise", "Image"),
    ("SimplifiedMesh", "Mesh"),
    (
        "ManualTextureViews",
        "ManualTextureViewHandle , ManualTextureView",
    ),
    ("SceneRoot", "Scene"),
    ("DynamicSceneRoot", "DynamicScene"),
    ("Mesh2dWireframe", "Wireframe2dMaterial"),
];
#[doc = r" Register newtype wrappers at runtime using TypeRegistry discovery"]
#[doc = r" Enables wrapping Handle<T> in newtypes like ImageRenderTarget"]
pub fn register_auto_newtype_wrappers(
    newtype_wrappers: &std::sync::Arc<
        std::sync::Mutex<std::collections::HashMap<String, bevy_lua_ecs::NewtypeWrapperCreator>>,
    >,
) {
    bevy::log::debug!(
        "[NEWTYPE_WRAPPERS] Discovered {} newtype wrappers for runtime lookup",
        DISCOVERED_NEWTYPE_WRAPPERS.len()
    );
    for (newtype_name, inner_name) in DISCOVERED_NEWTYPE_WRAPPERS {
        bevy::log::debug!(
            "[NEWTYPE_WRAPPERS]   - {} wraps Handle<{}>",
            newtype_name,
            inner_name
        );
    }
}
#[doc = r" Auto-discovered SystemParam type names and their full paths"]
#[doc = r" Format: (type_name, full_path) - for runtime lookup"]
#[doc = r#" Examples: ("MeshRayCast", "bevy::picking::mesh_picking::ray_cast::MeshRayCast")"#]
pub const DISCOVERED_SYSTEMPARAMS: &[(&str, &str)] = &[
    ("Diagnostics", "bevy::diagnostic::Diagnostics"),
    ("ComponentIdFor", "bevy::ecs::ComponentIdFor"),
    ("EventReader", "bevy::ecs::EventReader"),
    ("EventWriter", "bevy::ecs::EventWriter"),
    ("RemovedComponents", "bevy::ecs::RemovedComponents"),
    ("ParallelCommands", "bevy::ecs::ParallelCommands"),
    ("EventMutator", "bevy::ecs::EventMutator"),
    ("MessageMutator", "bevy::ecs::MessageMutator"),
    ("MessageReader", "bevy::ecs::MessageReader"),
    ("MessageWriter", "bevy::ecs::MessageWriter"),
    (
        "DirectionalNavigation",
        "bevy::input_focus::DirectionalNavigation",
    ),
    ("IsFocusedHelper", "bevy::input_focus::IsFocusedHelper"),
    ("TabNavigation", "bevy::input_focus::TabNavigation"),
    (
        "PickingMessageWriters",
        "bevy::picking::PickingMessageWriters",
    ),
    ("MeshRayCast", "bevy::picking::MeshRayCast"),
    (
        "ReadRapierContext",
        "bevy_rapier2d::prelude::ReadRapierContext",
    ),
    (
        "WriteRapierContext",
        "bevy_rapier2d::prelude::WriteRapierContext",
    ),
    ("FallbackImageMsaa", "bevy::render::FallbackImageMsaa"),
    ("TextReader", "bevy::text::TextReader"),
    ("TextWriter", "bevy::text::TextWriter"),
    ("TransformHelper", "bevy::transform::TransformHelper"),
    ("UiRootNodes", "bevy::ui::UiRootNodes"),
    ("UiChildren", "bevy::ui::UiChildren"),
    ("DefaultUiCamera", "bevy::ui::DefaultUiCamera"),
    ("UiCameraMap", "bevy::ui_render::UiCameraMap"),
];
#[doc = r" Auto-discovered SystemParam methods that use Reflect-compatible parameters"]
#[doc = r" Format: (param_type, method_name, return_type, returns_iterator)"]
pub const DISCOVERED_SYSTEMPARAM_METHODS: &[(&str, &str, &str, bool)] = &[
    ("ComponentIdFor", "get", "ComponentId", false),
    ("EventReader", "read", "EventIterator<'_,E>", false),
    (
        "EventReader",
        "read_with_id",
        "EventIteratorWithId<'_,E>",
        false,
    ),
    ("EventReader", "par_read", "EventParIter<'_,E>", true),
    ("EventReader", "len", "usize", false),
    ("EventReader", "is_empty", "bool", false),
    ("EventReader", "clear", "()", false),
    ("EventWriter", "send_default", "EventId<E>", false),
    (
        "RemovedComponents",
        "reader",
        "&ManualEventReader<RemovedComponentEntity>",
        false,
    ),
    (
        "RemovedComponents",
        "reader_mut",
        "&mutManualEventReader<RemovedComponentEntity>",
        false,
    ),
    (
        "RemovedComponents",
        "events",
        "Option<&Events<RemovedComponentEntity>>",
        false,
    ),
    (
        "RemovedComponents",
        "reader_mut_with_events",
        "Option<(&mutRemovedComponentReader<T>,&Events<RemovedComponentEntity>,)>",
        false,
    ),
    ("RemovedComponents", "read", "RemovedIter<'_>", true),
    (
        "RemovedComponents",
        "read_with_id",
        "RemovedIterWithId<'_>",
        false,
    ),
    ("RemovedComponents", "len", "usize", false),
    ("RemovedComponents", "is_empty", "bool", false),
    ("RemovedComponents", "clear", "()", false),
    ("ComponentIdFor", "get", "ComponentId", false),
    ("EventMutator", "read", "EventMutIterator<'_,E>", false),
    (
        "EventMutator",
        "read_with_id",
        "EventMutIteratorWithId<'_,E>",
        false,
    ),
    ("EventMutator", "par_read", "EventMutParIter<'_,E>", true),
    ("EventMutator", "len", "usize", false),
    ("EventMutator", "is_empty", "bool", false),
    ("EventMutator", "clear", "()", false),
    ("EventReader", "read", "EventIterator<'_,E>", false),
    (
        "EventReader",
        "read_with_id",
        "EventIteratorWithId<'_,E>",
        false,
    ),
    ("EventReader", "par_read", "EventParIter<'_,E>", true),
    ("EventReader", "len", "usize", false),
    ("EventReader", "is_empty", "bool", false),
    ("EventReader", "clear", "()", false),
    ("EventWriter", "write_default", "EventId<E>", false),
    ("EventWriter", "send_default", "EventId<E>", false),
    (
        "RemovedComponents",
        "reader",
        "&EventCursor<RemovedComponentEntity>",
        false,
    ),
    (
        "RemovedComponents",
        "reader_mut",
        "&mutEventCursor<RemovedComponentEntity>",
        false,
    ),
    (
        "RemovedComponents",
        "events",
        "Option<&Events<RemovedComponentEntity>>",
        false,
    ),
    (
        "RemovedComponents",
        "reader_mut_with_events",
        "Option<(&mutRemovedComponentReader<T>,&Events<RemovedComponentEntity>,)>",
        false,
    ),
    ("RemovedComponents", "read", "RemovedIter<'_>", true),
    (
        "RemovedComponents",
        "read_with_id",
        "RemovedIterWithId<'_>",
        false,
    ),
    ("RemovedComponents", "len", "usize", false),
    ("RemovedComponents", "is_empty", "bool", false),
    ("RemovedComponents", "clear", "()", false),
    ("ComponentIdFor", "get", "ComponentId", false),
    (
        "RemovedComponents",
        "reader",
        "&MessageCursor<RemovedComponentEntity>",
        false,
    ),
    (
        "RemovedComponents",
        "reader_mut",
        "&mutMessageCursor<RemovedComponentEntity>",
        false,
    ),
    (
        "RemovedComponents",
        "events",
        "Option<&Messages<RemovedComponentEntity>>",
        false,
    ),
    (
        "RemovedComponents",
        "messages",
        "Option<&Messages<RemovedComponentEntity>>",
        false,
    ),
    (
        "RemovedComponents",
        "reader_mut_with_messages",
        "Option<(&mutRemovedComponentReader<T>,&Messages<RemovedComponentEntity>,)>",
        false,
    ),
    (
        "RemovedComponents",
        "reader_mut_with_events",
        "Option<(&mutRemovedComponentReader<T>,&Messages<RemovedComponentEntity>,)>",
        false,
    ),
    ("RemovedComponents", "read", "RemovedIter<'_>", true),
    (
        "RemovedComponents",
        "read_with_id",
        "RemovedIterWithId<'_>",
        false,
    ),
    ("RemovedComponents", "len", "usize", false),
    ("RemovedComponents", "is_empty", "bool", false),
    ("RemovedComponents", "clear", "()", false),
    ("MessageMutator", "read", "MessageMutIterator<'_,E>", false),
    (
        "MessageMutator",
        "read_with_id",
        "MessageMutIteratorWithId<'_,E>",
        false,
    ),
    (
        "MessageMutator",
        "par_read",
        "MessageMutParIter<'_,E>",
        true,
    ),
    ("MessageMutator", "len", "usize", false),
    ("MessageMutator", "is_empty", "bool", false),
    ("MessageMutator", "clear", "()", false),
    ("MessageReader", "read", "MessageIterator<'_,E>", false),
    (
        "MessageReader",
        "read_with_id",
        "MessageIteratorWithId<'_,E>",
        false,
    ),
    ("MessageReader", "par_read", "MessageParIter<'_,E>", true),
    ("MessageReader", "len", "usize", false),
    ("MessageReader", "is_empty", "bool", false),
    ("MessageReader", "clear", "()", false),
    ("MessageWriter", "write_default", "MessageId<E>", false),
    (
        "ReadRapierContext",
        "single",
        "Result<RapierContext<'_>>",
        false,
    ),
    (
        "WriteRapierContext",
        "single",
        "Result<RapierContext<'_>>",
        false,
    ),
    (
        "WriteRapierContext",
        "single_mut",
        "Result<RapierContextMut<'_>>",
        false,
    ),
    (
        "ReadRapierContext",
        "single",
        "Result<RapierContext<'_>>",
        false,
    ),
    (
        "WriteRapierContext",
        "single",
        "Result<RapierContext<'_>>",
        false,
    ),
    (
        "WriteRapierContext",
        "single_mut",
        "Result<RapierContextMut<'_>>",
        false,
    ),
    ("UiRootNodes", "iter", "implIterator<Item=Entity>+'s", false),
    ("DefaultUiCamera", "get", "Option<Entity>", false),
    ("UiCameraMap", "get_mapper", "UiCameraMapper<'w,'s>", false),
];
#[doc = r" Dispatch a SystemParam method call from Lua"]
#[doc = r" This uses SystemState to access SystemParams from World"]
#[doc = r" Currently supports no-arg methods; parameterized methods need reflection-based arg parsing"]
pub fn dispatch_systemparam_method(
    lua: &mlua::Lua,
    world: &mut bevy::prelude::World,
    param_name: &str,
    method_name: &str,
    args: mlua::MultiValue,
) -> mlua::Result<mlua::Value> {
    match (param_name, method_name) {
        ("MeshRayCast", "cast_ray") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let typed_arg0: bevy::math::Ray3d = {
                let type_reg = type_registry
                    .get_with_short_type_path("Ray3d")
                    .or_else(|| type_registry.get_with_type_path("Ray3d"));
                let param_result: Option<Box<dyn bevy::reflect::Reflect>> = type_reg
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>())
                    .map(|rd| rd.default());
                let used_default = param_result.is_some();
                let mut param_instance = if let Some(inst) = param_result {
                    inst
                } else {
                    if let Some(arg_val) = args.front() {
                        if let mlua::Value::Table(t) = arg_val {
                            if let Some(type_registration) = type_reg {
                                if let Some(from_reflect_data) =
                                    type_registration.data::<bevy::reflect::ReflectFromReflect>()
                                {
                                    let type_info = type_registration.type_info();
                                    let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                        lua,
                                        t,
                                        type_info,
                                        &app_type_registry,
                                    )
                                    .map_err(|e| {
                                        mlua::Error::RuntimeError(format!(
                                            "Failed to build DynamicStruct for '{}': {}",
                                            "Ray3d", e
                                        ))
                                    })?;
                                    if let Some(reflected) =
                                        from_reflect_data.from_reflect(&dynamic)
                                    {
                                        args.pop_front();
                                        reflected
                                    } else {
                                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - FromReflect conversion failed. Check that all fields are provided." , "Ray3d"))) ;
                                    }
                                } else {
                                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - doesn't implement FromReflect" , "Ray3d"))) ;
                                }
                            } else {
                                return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - not found in TypeRegistry" , "Ray3d"))) ;
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Cannot construct parameter type '{}' - expected table argument",
                                "Ray3d"
                            )));
                        }
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Ray3d"))) ;
                    }
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .downcast_ref::<bevy::math::Ray3d>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Ray3d"
                        ))
                    })?
            };
            let closure_struct_1 = bevy::picking::mesh_picking::ray_cast::MeshRayCastSettings {
                filter: &|_| true,
                early_exit_test: &|_| false,
                ..Default::default()
            };
            if args.front().is_some() {
                if let Some(mlua::Value::Table(_)) = args.pop_front() {
                    bevy :: log :: debug ! ("Struct '{}' has closure fields - using permissive defaults (closure fields can't be customized from Lua)" , "MeshRayCastSettings");
                }
            }
            let mut state = bevy::ecs::system::SystemState::<
                bevy::picking::mesh_picking::ray_cast::MeshRayCast,
            >::new(world);
            let mut param = state.get_mut(world);
            let result = param.cast_ray(typed_arg0, &closure_struct_1);
            bevy_lua_ecs::reflection::result_to_lua_value(lua, &result)
        }
        _ => Err(mlua::Error::RuntimeError(format!(
            "Unknown or unsupported SystemParam method: {}::{}",
            param_name, method_name
        ))),
    }
}
#[doc = r" Dispatch a Component method call from Lua"]
#[doc = r" This directly accesses components on entities and calls their methods"]
#[doc = r" Supports Transform::looking_at, Transform::looking_to, etc."]
pub fn dispatch_component_method(
    lua: &mlua::Lua,
    world: &mut bevy::prelude::World,
    entity_id: u64,
    type_name: &str,
    method_name: &str,
    args: mlua::MultiValue,
) -> mlua::Result<mlua::Value> {
    match (type_name, method_name) {
        ("GlobalTransform", "to_matrix") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<GlobalTransform>(entity) {
                let result = comp.to_matrix();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "GlobalTransform"
                )))
            }
        }
        ("GlobalTransform", "affine") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<GlobalTransform>(entity) {
                let result = comp.affine();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "GlobalTransform"
                )))
            }
        }
        ("GlobalTransform", "compute_transform") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<GlobalTransform>(entity) {
                let result = comp.compute_transform();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "GlobalTransform"
                )))
            }
        }
        ("GlobalTransform", "to_isometry") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<GlobalTransform>(entity) {
                let result = comp.to_isometry();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "GlobalTransform"
                )))
            }
        }
        ("GlobalTransform", "reparented_to") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: GlobalTransform = {
                let reflect_default = type_registry
                    .get_with_short_type_path("GlobalTransform")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("GlobalTransform")
                        {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "GlobalTransform", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "GlobalTransform"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "GlobalTransform"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "GlobalTransform"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "GlobalTransform",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "GlobalTransform"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<GlobalTransform>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "GlobalTransform"
                        ))
                    })?
            };
            if let Some(comp) = world.get::<GlobalTransform>(entity) {
                let result = comp.reparented_to(&typed_param_0);
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "GlobalTransform"
                )))
            }
        }
        ("GlobalTransform", "to_scale_rotation_translation") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<GlobalTransform>(entity) {
                let result = comp.to_scale_rotation_translation();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "GlobalTransform"
                )))
            }
        }
        ("GlobalTransform", "translation") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<GlobalTransform>(entity) {
                let result = comp.translation();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "GlobalTransform"
                )))
            }
        }
        ("GlobalTransform", "translation_vec3a") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<GlobalTransform>(entity) {
                let result = comp.translation_vec3a();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "GlobalTransform"
                )))
            }
        }
        ("GlobalTransform", "rotation") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<GlobalTransform>(entity) {
                let result = comp.rotation();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "GlobalTransform"
                )))
            }
        }
        ("GlobalTransform", "scale") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<GlobalTransform>(entity) {
                let result = comp.scale();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "GlobalTransform"
                )))
            }
        }
        ("GlobalTransform", "radius_vec3a") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Vec3A = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3A")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3A") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3A", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3A"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3A"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3A"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3A",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3A"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3A>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3A"
                        ))
                    })?
            };
            if let Some(comp) = world.get::<GlobalTransform>(entity) {
                let result = comp.radius_vec3a(typed_param_0);
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "GlobalTransform"
                )))
            }
        }
        ("GlobalTransform", "transform_point") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            if let Some(comp) = world.get::<GlobalTransform>(entity) {
                let result = comp.transform_point(typed_param_0);
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "GlobalTransform"
                )))
            }
        }
        ("GlobalTransform", "mul_transform") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Transform = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Transform")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Transform") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Transform", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Transform"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Transform"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Transform"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Transform",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Transform"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Transform>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Transform"
                        ))
                    })?
            };
            if let Some(comp) = world.get::<GlobalTransform>(entity) {
                let result = comp.mul_transform(typed_param_0);
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "GlobalTransform"
                )))
            }
        }
        ("Transform", "looking_at") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            let typed_param_1: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                let result = comp.looking_at(typed_param_0, typed_param_1);
                *comp = result;
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "looking_to") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            let typed_param_1: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                let result = comp.looking_to(typed_param_0, typed_param_1);
                *comp = result;
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "aligned_by") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            let typed_param_1: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            let typed_param_2: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            let typed_param_3: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                let result =
                    comp.aligned_by(typed_param_0, typed_param_1, typed_param_2, typed_param_3);
                *comp = result;
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "with_translation") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                let result = comp.with_translation(typed_param_0);
                *comp = result;
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "with_rotation") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Quat = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Quat")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Quat", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Quat"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Quat",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Quat"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Quat>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Quat"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                let result = comp.with_rotation(typed_param_0);
                *comp = result;
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "with_scale") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                let result = comp.with_scale(typed_param_0);
                *comp = result;
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "to_matrix") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.to_matrix();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "compute_affine") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.compute_affine();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "local_x") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.local_x();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "left") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.left();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "right") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.right();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "local_y") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.local_y();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "up") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.up();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "down") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.down();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "local_z") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.local_z();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "forward") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.forward();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "back") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.back();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "rotate") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Quat = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Quat")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Quat", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Quat"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Quat",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Quat"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Quat>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Quat"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.rotate(typed_param_0);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "rotate_axis") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Dir3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Dir3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Dir3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Dir3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Dir3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Dir3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Dir3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Dir3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Dir3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Dir3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Dir3"
                        ))
                    })?
            };
            let typed_param_1: f32 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("f32")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("f32") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "f32", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "f32"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "f32"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "f32"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "f32",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<f32>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "f32"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.rotate_axis(typed_param_0, typed_param_1);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "rotate_x") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: f32 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("f32")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("f32") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "f32", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "f32"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "f32"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "f32"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "f32",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<f32>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "f32"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.rotate_x(typed_param_0);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "rotate_y") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: f32 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("f32")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("f32") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "f32", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "f32"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "f32"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "f32"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "f32",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<f32>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "f32"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.rotate_y(typed_param_0);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "rotate_z") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: f32 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("f32")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("f32") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "f32", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "f32"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "f32"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "f32"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "f32",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<f32>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "f32"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.rotate_z(typed_param_0);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "rotate_local") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Quat = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Quat")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Quat", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Quat"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Quat",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Quat"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Quat>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Quat"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.rotate_local(typed_param_0);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "rotate_local_axis") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Dir3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Dir3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Dir3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Dir3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Dir3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Dir3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Dir3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Dir3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Dir3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Dir3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Dir3"
                        ))
                    })?
            };
            let typed_param_1: f32 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("f32")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("f32") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "f32", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "f32"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "f32"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "f32"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "f32",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<f32>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "f32"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.rotate_local_axis(typed_param_0, typed_param_1);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "rotate_local_x") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: f32 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("f32")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("f32") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "f32", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "f32"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "f32"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "f32"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "f32",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<f32>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "f32"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.rotate_local_x(typed_param_0);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "rotate_local_y") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: f32 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("f32")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("f32") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "f32", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "f32"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "f32"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "f32"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "f32",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<f32>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "f32"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.rotate_local_y(typed_param_0);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "rotate_local_z") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: f32 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("f32")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("f32") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "f32", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "f32"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "f32"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "f32"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "f32",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<f32>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "f32"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.rotate_local_z(typed_param_0);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "translate_around") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            let typed_param_1: Quat = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Quat")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Quat", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Quat"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Quat",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Quat"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Quat>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Quat"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.translate_around(typed_param_0, typed_param_1);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "rotate_around") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            let typed_param_1: Quat = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Quat")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Quat", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Quat"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Quat",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Quat"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Quat>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Quat"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.rotate_around(typed_param_0, typed_param_1);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "look_at") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            let typed_param_1: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.look_at(typed_param_0, typed_param_1);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "look_to") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            let typed_param_1: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.look_to(typed_param_0, typed_param_1);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "align") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            let typed_param_1: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            let typed_param_2: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            let typed_param_3: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            if let Some(mut comp) = world.get_mut::<Transform>(entity) {
                comp.align(typed_param_0, typed_param_1, typed_param_2, typed_param_3);
                Ok(mlua::Value::Nil)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "mul_transform") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Transform = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Transform")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Transform") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Transform", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Transform"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Transform"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Transform"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Transform",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Transform"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Transform>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Transform"
                        ))
                    })?
            };
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.mul_transform(typed_param_0);
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "transform_point") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            let typed_param_0: Vec3 = {
                let reflect_default = type_registry
                    .get_with_short_type_path("Vec3")
                    .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                let mut param_instance: Box<dyn bevy::reflect::PartialReflect>;
                let mut used_default = false;
                if let Some(rd) = reflect_default {
                    param_instance = rd.default().into_partial_reflect();
                    used_default = true;
                } else if let Some(arg_val) = args.pop_front() {
                    if let mlua::Value::Table(ref arg_table) = arg_val {
                        if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                            if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                                let type_info = reg.type_info();
                                let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                    lua,
                                    arg_table,
                                    type_info,
                                    &app_type_registry,
                                )
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to build param '{}': {}",
                                        "Vec3", e
                                    ))
                                })?;
                                if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                    param_instance = concrete;
                                } else {
                                    return Err(mlua::Error::RuntimeError(format!(
                                        "Failed to construct parameter type '{}' via FromReflect",
                                        "Vec3"
                                    )));
                                }
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Parameter type '{}' has no FromReflect implementation",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Parameter type '{}' not found in TypeRegistry",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Parameter type '{}' expected table argument, got {:?}",
                            "Vec3",
                            arg_val.type_name()
                        )));
                    }
                } else {
                    return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                };
                if used_default {
                    if let Some(arg_val) = args.pop_front() {
                        if let mlua::Value::Table(t) = arg_val {
                            let _ = bevy_lua_ecs::lua_to_reflection(
                                lua,
                                &mlua::Value::Table(t),
                                param_instance.as_partial_reflect_mut(),
                                &app_type_registry,
                            );
                        }
                    }
                }
                param_instance
                    .try_downcast_ref::<Vec3>()
                    .cloned()
                    .ok_or_else(|| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to downcast parameter to '{}'",
                            "Vec3"
                        ))
                    })?
            };
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.transform_point(typed_param_0);
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "is_finite") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.is_finite();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        ("Transform", "to_isometry") => {
            let app_type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let type_registry = app_type_registry.read();
            let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
            let entity = bevy::prelude::Entity::from_bits(entity_id);
            if let Some(comp) = world.get::<Transform>(entity) {
                let result = comp.to_isometry();
                bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
            } else {
                Err(mlua::Error::RuntimeError(format!(
                    "Entity {:?} has no {} component",
                    entity, "Transform"
                )))
            }
        }
        _ => Err(mlua::Error::RuntimeError(format!(
            "Unknown or unsupported Component method: {}::{}",
            type_name, method_name
        ))),
    }
}
#[doc = r" Dispatch a static method call from Lua for math types (Quat, Vec3, etc.)"]
#[doc = r" These do not require entity access or World - they operate on pure data"]
#[doc = r" Methods are generated based on [package.metadata.lua_methods] static_types config"]
pub fn dispatch_static_method(
    lua: &mlua::Lua,
    world: &bevy::prelude::World,
    type_name: &str,
    method_name: &str,
    args: mlua::MultiValue,
) -> mlua::Result<mlua::Value> {
    let mut args: std::collections::VecDeque<mlua::Value> = args.into_iter().collect();
    match (type_name, method_name) {
        ("Quat", "from_xyzw") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let typed_param_2: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let typed_param_3: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result =
                Quat::from_xyzw(typed_param_0, typed_param_1, typed_param_2, typed_param_3);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "from_axis_angle") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = Quat::from_axis_angle(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "from_scaled_axis") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = Quat::from_scaled_axis(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "from_rotation_x") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = Quat::from_rotation_x(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "from_rotation_y") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = Quat::from_rotation_y(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "from_rotation_z") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = Quat::from_rotation_z(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "from_euler") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: EulerRot = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<EulerRot>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "EulerRot", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("EulerRot")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<EulerRot>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "EulerRot"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "EulerRot"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let typed_param_2: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let typed_param_3: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result =
                Quat::from_euler(typed_param_0, typed_param_1, typed_param_2, typed_param_3);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "from_rotation_arc") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = Quat::from_rotation_arc(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "from_rotation_arc_colinear") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = Quat::from_rotation_arc_colinear(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "look_to_lh") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = Quat::look_to_lh(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "look_to_rh") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = Quat::look_to_rh(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "look_at_lh") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_2: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = Quat::look_at_lh(typed_param_0, typed_param_1, typed_param_2);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "look_at_rh") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_2: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = Quat::look_at_rh(typed_param_0, typed_param_1, typed_param_2);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "to_scaled_axis") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "to_scaled_axis"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "to_scaled_axis",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.to_scaled_axis();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "xyz") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "xyz"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "xyz",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.xyz();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "conjugate") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "conjugate"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "conjugate",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.conjugate();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "inverse") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "inverse"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "inverse",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.inverse();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "dot") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "dot"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "dot",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Quat = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Quat>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Quat", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Quat")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Quat>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Quat"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Quat"))) ;
                    }
                }
            };
            let result = self_instance.dot(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "length") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "length"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "length",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.length();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "length_squared") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "length_squared"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "length_squared",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.length_squared();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "length_recip") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "length_recip"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "length_recip",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.length_recip();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "normalize") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "normalize"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "normalize",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.normalize();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "is_finite") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "is_finite"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "is_finite",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.is_finite();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "is_nan") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "is_nan"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "is_nan",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.is_nan();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "is_normalized") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "is_normalized"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "is_normalized",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.is_normalized();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "is_near_identity") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "is_near_identity"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "is_near_identity",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.is_near_identity();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "angle_between") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "angle_between"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "angle_between",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Quat = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Quat>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Quat", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Quat")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Quat>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Quat"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Quat"))) ;
                    }
                }
            };
            let result = self_instance.angle_between(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "rotate_towards") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "rotate_towards"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "rotate_towards",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Quat = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Quat>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Quat", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Quat")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Quat>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Quat"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Quat"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.rotate_towards(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "abs_diff_eq") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "abs_diff_eq"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "abs_diff_eq",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Quat = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Quat>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Quat", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Quat")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Quat>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Quat"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Quat"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.abs_diff_eq(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "lerp") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "lerp"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "lerp",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Quat = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Quat>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Quat", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Quat")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Quat>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Quat"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Quat"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.lerp(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "slerp") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "slerp"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "slerp",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Quat = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Quat>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Quat", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Quat")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Quat>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Quat"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Quat"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.slerp(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "mul_vec3") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "mul_vec3"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "mul_vec3",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.mul_vec3(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Quat", "mul_quat") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Quat = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Quat", "mul_quat"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Quat") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Quat", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Quat>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Quat"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Quat"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Quat"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Quat"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Quat",
                        "mul_quat",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Quat = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Quat>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Quat", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Quat")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Quat>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Quat"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Quat"))) ;
                    }
                }
            };
            let result = self_instance.mul_quat(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "new") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let typed_param_2: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = Vec3::new(typed_param_0, typed_param_1, typed_param_2);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "splat") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = Vec3::splat(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "with_x") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "with_x"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "with_x",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.with_x(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "with_y") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "with_y"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "with_y",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.with_y(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "with_z") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "with_z"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "with_z",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.with_z(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "dot") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "dot"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "dot",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.dot(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "dot_into_vec") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "dot_into_vec"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "dot_into_vec",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.dot_into_vec(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "cross") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "cross"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "cross",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.cross(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "min") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "min"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "min",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.min(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "max") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "max"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "max",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.max(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "clamp") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "clamp"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "clamp",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.clamp(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "min_element") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "min_element"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "min_element",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.min_element();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "max_element") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "max_element"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "max_element",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.max_element();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "min_position") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "min_position"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "min_position",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.min_position();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "max_position") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "max_position"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "max_position",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.max_position();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "element_sum") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "element_sum"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "element_sum",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.element_sum();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "element_product") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "element_product"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "element_product",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.element_product();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "abs") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "abs"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "abs",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.abs();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "signum") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "signum"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "signum",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.signum();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "copysign") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "copysign"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "copysign",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.copysign(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "is_negative_bitmask") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "is_negative_bitmask"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "is_negative_bitmask",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.is_negative_bitmask();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "is_finite") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "is_finite"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "is_finite",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.is_finite();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "is_nan") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "is_nan"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "is_nan",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.is_nan();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "length") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "length"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "length",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.length();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "length_squared") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "length_squared"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "length_squared",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.length_squared();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "length_recip") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "length_recip"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "length_recip",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.length_recip();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "distance") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "distance"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "distance",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.distance(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "distance_squared") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "distance_squared"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "distance_squared",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.distance_squared(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "div_euclid") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "div_euclid"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "div_euclid",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.div_euclid(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "rem_euclid") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "rem_euclid"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "rem_euclid",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.rem_euclid(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "normalize") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "normalize"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "normalize",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.normalize();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "normalize_or") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "normalize_or"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "normalize_or",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.normalize_or(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "normalize_or_zero") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "normalize_or_zero"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "normalize_or_zero",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.normalize_or_zero();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "is_normalized") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "is_normalized"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "is_normalized",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.is_normalized();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "project_onto") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "project_onto"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "project_onto",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.project_onto(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "reject_from") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "reject_from"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "reject_from",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.reject_from(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "project_onto_normalized") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "project_onto_normalized"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "project_onto_normalized",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.project_onto_normalized(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "reject_from_normalized") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "reject_from_normalized"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "reject_from_normalized",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.reject_from_normalized(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "round") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "round"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "round",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.round();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "floor") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "floor"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "floor",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.floor();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "ceil") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "ceil"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "ceil",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.ceil();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "trunc") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "trunc"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "trunc",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.trunc();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "fract") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "fract"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "fract",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.fract();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "fract_gl") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "fract_gl"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "fract_gl",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.fract_gl();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "exp") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "exp"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "exp",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.exp();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "powf") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "powf"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "powf",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.powf(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "recip") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "recip"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "recip",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.recip();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "lerp") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "lerp"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "lerp",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.lerp(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "move_towards") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "move_towards"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "move_towards",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.move_towards(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "midpoint") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "midpoint"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "midpoint",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.midpoint(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "abs_diff_eq") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "abs_diff_eq"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "abs_diff_eq",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.abs_diff_eq(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "clamp_length") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "clamp_length"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "clamp_length",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.clamp_length(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "clamp_length_max") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "clamp_length_max"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "clamp_length_max",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.clamp_length_max(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "clamp_length_min") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "clamp_length_min"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "clamp_length_min",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.clamp_length_min(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "mul_add") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "mul_add"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "mul_add",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.mul_add(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "reflect") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "reflect"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "reflect",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.reflect(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "refract") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "refract"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "refract",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.refract(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "angle_between") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "angle_between"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "angle_between",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let result = self_instance.angle_between(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "rotate_x") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "rotate_x"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "rotate_x",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.rotate_x(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "rotate_y") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "rotate_y"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "rotate_y",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.rotate_y(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "rotate_z") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "rotate_z"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "rotate_z",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.rotate_z(typed_param_0);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "rotate_axis") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "rotate_axis"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "rotate_axis",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.rotate_axis(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "rotate_towards") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "rotate_towards"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "rotate_towards",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.rotate_towards(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "any_orthogonal_vector") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "any_orthogonal_vector"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "any_orthogonal_vector",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.any_orthogonal_vector();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "any_orthonormal_vector") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "any_orthonormal_vector"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "any_orthonormal_vector",
                        self_arg.type_name()
                    )));
                }
            };
            let result = self_instance.any_orthonormal_vector();
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        ("Vec3", "slerp") => {
            let app_type_registry = world.resource::<bevy::ecs::reflect::AppTypeRegistry>();
            let type_registry = app_type_registry.read();
            let self_instance: Vec3 = {
                let self_arg = args.pop_front().ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "{}::{} requires a self argument",
                        "Vec3", "slerp"
                    ))
                })?;
                if let mlua::Value::Table(ref arg_table) = self_arg {
                    if let Some(reg) = type_registry.get_with_short_type_path("Vec3") {
                        if let Some(rfr) = reg.data::<bevy::reflect::ReflectFromReflect>() {
                            let type_info = reg.type_info();
                            let dynamic = bevy_lua_ecs::lua_table_to_dynamic(
                                lua,
                                arg_table,
                                type_info,
                                &app_type_registry,
                            )
                            .map_err(|e| {
                                mlua::Error::RuntimeError(format!(
                                    "Failed to build self '{}': {}",
                                    "Vec3", e
                                ))
                            })?;
                            if let Some(concrete) = rfr.from_reflect(&dynamic) {
                                *concrete.downcast::<Vec3>().map_err(|_| {
                                    mlua::Error::RuntimeError(format!(
                                        "Failed to downcast self to {}",
                                        "Vec3"
                                    ))
                                })?
                            } else {
                                return Err(mlua::Error::RuntimeError(format!(
                                    "Failed to construct self type '{}' via FromReflect",
                                    "Vec3"
                                )));
                            }
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Type '{}' has no FromReflect implementation",
                                "Vec3"
                            )));
                        }
                    } else {
                        return Err(mlua::Error::RuntimeError(format!(
                            "Type '{}' not found in TypeRegistry",
                            "Vec3"
                        )));
                    }
                } else {
                    return Err(mlua::Error::RuntimeError(format!(
                        "{}::{} self argument expected table, got {:?}",
                        "Vec3",
                        "slerp",
                        self_arg.type_name()
                    )));
                }
            };
            let typed_param_0: Vec3 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<Vec3>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "Vec3", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("Vec3")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<Vec3>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "Vec3"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "Vec3"))) ;
                    }
                }
            };
            let typed_param_1: f32 = {
                if let Some(arg_val) = args.pop_front() {
                    bevy_lua_ecs::reflection::lua_value_to_type::<f32>(
                        lua,
                        arg_val,
                        &app_type_registry,
                    )
                    .map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to convert parameter '{}': {}",
                            "f32", e
                        ))
                    })?
                } else {
                    let reflect_default = type_registry
                        .get_with_short_type_path("f32")
                        .and_then(|reg| reg.data::<bevy::prelude::ReflectDefault>());
                    if let Some(rd) = reflect_default {
                        let param_instance = rd.default();
                        *param_instance.downcast::<f32>().map_err(|_| {
                            mlua::Error::RuntimeError(format!(
                                "Failed to downcast default to '{}'",
                                "f32"
                            ))
                        })?
                    } else {
                        return Err (mlua :: Error :: RuntimeError (format ! ("Cannot construct parameter type '{}' - no argument provided and no Default" , "f32"))) ;
                    }
                }
            };
            let result = self_instance.slerp(typed_param_0, typed_param_1);
            bevy_lua_ecs::reflection::try_reflect_to_lua_value(lua, &result)
        }
        _ => Err(mlua::Error::RuntimeError(format!(
            "Unknown or unsupported static method: {}::{}",
            type_name, method_name
        ))),
    }
}
#[doc = r" Returns a Lua table of events converted via reflection"]
#[doc = r" Also supports reading Message types (uses MessageReader instead of EventReader)"]
pub fn dispatch_read_events(
    lua: &mlua::Lua,
    world: &mut bevy::prelude::World,
    event_type: &str,
) -> mlua::Result<mlua::Value> {
    let type_registry = world
        .resource::<bevy::ecs::reflect::AppTypeRegistry>()
        .clone();
    match event_type {
        "WindowResized" | "bevy_window::event::WindowResized" | "bevy::window::WindowResized" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "WindowResized");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "RequestRedraw" | "bevy_window::event::RequestRedraw" | "bevy::window::RequestRedraw" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "RequestRedraw");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "WindowCreated" | "bevy_window::event::WindowCreated" | "bevy::window::WindowCreated" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "WindowCreated");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "WindowCloseRequested"
        | "bevy_window::event::WindowCloseRequested"
        | "bevy::window::WindowCloseRequested" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "WindowCloseRequested");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "WindowClosed" | "bevy_window::event::WindowClosed" | "bevy::window::WindowClosed" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "WindowClosed");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "WindowClosing" | "bevy_window::event::WindowClosing" | "bevy::window::WindowClosing" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "WindowClosing");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "WindowDestroyed"
        | "bevy_window::event::WindowDestroyed"
        | "bevy::window::WindowDestroyed" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "WindowDestroyed");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "CursorMoved" | "bevy_window::event::CursorMoved" | "bevy::window::CursorMoved" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "CursorMoved");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "CursorEntered" | "bevy_window::event::CursorEntered" | "bevy::window::CursorEntered" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "CursorEntered");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "CursorLeft" | "bevy_window::event::CursorLeft" | "bevy::window::CursorLeft" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "CursorLeft");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "WindowFocused" | "bevy_window::event::WindowFocused" | "bevy::window::WindowFocused" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "WindowFocused");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "WindowOccluded"
        | "bevy_window::event::WindowOccluded"
        | "bevy::window::WindowOccluded" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "WindowOccluded");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "WindowScaleFactorChanged"
        | "bevy_window::event::WindowScaleFactorChanged"
        | "bevy::window::WindowScaleFactorChanged" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "WindowScaleFactorChanged");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "WindowBackendScaleFactorChanged"
        | "bevy_window::event::WindowBackendScaleFactorChanged"
        | "bevy::window::WindowBackendScaleFactorChanged" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "WindowBackendScaleFactorChanged");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "FileDragAndDrop"
        | "bevy_window::event::FileDragAndDrop"
        | "bevy::window::FileDragAndDrop" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "FileDragAndDrop");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "WindowMoved" | "bevy_window::event::WindowMoved" | "bevy::window::WindowMoved" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "WindowMoved");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "WindowThemeChanged"
        | "bevy_window::event::WindowThemeChanged"
        | "bevy::window::WindowThemeChanged" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "WindowThemeChanged");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "AppLifecycle" | "bevy_window::event::AppLifecycle" | "bevy::window::AppLifecycle" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "AppLifecycle");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "KeyboardInput"
        | "bevy_input::keyboard::KeyboardInput"
        | "bevy::input::keyboard::KeyboardInput" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "KeyboardInput");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "KeyboardFocusLost"
        | "bevy_input::keyboard::KeyboardFocusLost"
        | "bevy::input::keyboard::KeyboardFocusLost" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "KeyboardFocusLost");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "MouseButtonInput"
        | "bevy_input::mouse::MouseButtonInput"
        | "bevy::input::mouse::MouseButtonInput" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "MouseButtonInput");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "MouseMotion" | "bevy_input::mouse::MouseMotion" | "bevy::input::mouse::MouseMotion" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "MouseMotion");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "MouseWheel" | "bevy_input::mouse::MouseWheel" | "bevy::input::mouse::MouseWheel" => {
            let instance_id: u64 = lua.globals().get("__INSTANCE_ID__").unwrap_or(0);
            let accumulator = world
                .resource::<bevy_lua_ecs::LuaEventAccumulator>()
                .clone();
            let events = accumulator.drain(instance_id, "MouseWheel");
            let results = lua.create_table()?;
            let mut index = 1;
            for event_json in events {
                if let Ok(lua_value) = bevy_lua_ecs::json_to_lua_value(lua, &event_json) {
                    results.set(index, lua_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "AssetDeleteEvent"
        | "hello::asset_events::AssetDeleteEvent"
        | "hello::asset_events::AssetDeleteEvent" => {
            let mut system_state = bevy::ecs::system::SystemState::<
                bevy::prelude::MessageReader<crate::asset_events::AssetDeleteEvent>,
            >::new(world);
            let mut message_reader = system_state.get_mut(world);
            let results = lua.create_table()?;
            let mut index = 1;
            for message in message_reader.read() {
                if let Ok(message_value) = bevy_lua_ecs::reflection_to_lua(
                    lua,
                    message as &dyn bevy::reflect::PartialReflect,
                    &type_registry,
                ) {
                    results.set(index, message_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "AssetDirectoryListingEvent"
        | "hello::asset_events::AssetDirectoryListingEvent"
        | "hello::asset_events::AssetDirectoryListingEvent" => {
            let mut system_state = bevy::ecs::system::SystemState::<
                bevy::prelude::MessageReader<crate::asset_events::AssetDirectoryListingEvent>,
            >::new(world);
            let mut message_reader = system_state.get_mut(world);
            let results = lua.create_table()?;
            let mut index = 1;
            for message in message_reader.read() {
                if let Ok(message_value) = bevy_lua_ecs::reflection_to_lua(
                    lua,
                    message as &dyn bevy::reflect::PartialReflect,
                    &type_registry,
                ) {
                    results.set(index, message_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "AssetLocalNewerEvent"
        | "hello::asset_events::AssetLocalNewerEvent"
        | "hello::asset_events::AssetLocalNewerEvent" => {
            let mut system_state = bevy::ecs::system::SystemState::<
                bevy::prelude::MessageReader<crate::asset_events::AssetLocalNewerEvent>,
            >::new(world);
            let mut message_reader = system_state.get_mut(world);
            let results = lua.create_table()?;
            let mut index = 1;
            for message in message_reader.read() {
                if let Ok(message_value) = bevy_lua_ecs::reflection_to_lua(
                    lua,
                    message as &dyn bevy::reflect::PartialReflect,
                    &type_registry,
                ) {
                    results.set(index, message_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "AssetRenameEvent"
        | "hello::asset_events::AssetRenameEvent"
        | "hello::asset_events::AssetRenameEvent" => {
            let mut system_state = bevy::ecs::system::SystemState::<
                bevy::prelude::MessageReader<crate::asset_events::AssetRenameEvent>,
            >::new(world);
            let mut message_reader = system_state.get_mut(world);
            let results = lua.create_table()?;
            let mut index = 1;
            for message in message_reader.read() {
                if let Ok(message_value) = bevy_lua_ecs::reflection_to_lua(
                    lua,
                    message as &dyn bevy::reflect::PartialReflect,
                    &type_registry,
                ) {
                    results.set(index, message_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "AssetUploadProgressEvent"
        | "hello::asset_events::AssetUploadProgressEvent"
        | "hello::asset_events::AssetUploadProgressEvent" => {
            let mut system_state = bevy::ecs::system::SystemState::<
                bevy::prelude::MessageReader<crate::asset_events::AssetUploadProgressEvent>,
            >::new(world);
            let mut message_reader = system_state.get_mut(world);
            let results = lua.create_table()?;
            let mut index = 1;
            for message in message_reader.read() {
                if let Ok(message_value) = bevy_lua_ecs::reflection_to_lua(
                    lua,
                    message as &dyn bevy::reflect::PartialReflect,
                    &type_registry,
                ) {
                    results.set(index, message_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        "PointerInput"
        | "bevy_picking::pointer::PointerInput"
        | "bevy::picking::pointer::PointerInput" => {
            let mut system_state = bevy::ecs::system::SystemState::<
                bevy::prelude::MessageReader<bevy::picking::pointer::PointerInput>,
            >::new(world);
            let mut message_reader = system_state.get_mut(world);
            let results = lua.create_table()?;
            let mut index = 1;
            for message in message_reader.read() {
                if let Ok(message_value) = bevy_lua_ecs::reflection_to_lua(
                    lua,
                    message as &dyn bevy::reflect::PartialReflect,
                    &type_registry,
                ) {
                    results.set(index, message_value)?;
                    index += 1;
                }
            }
            Ok(mlua::Value::Table(results))
        }
        _ => Err(mlua::Error::RuntimeError(format!(
            "Unknown event type: '{}'. Available types include Bevy events and Message types.",
            event_type
        ))),
    }
}
#[doc = r" Dispatch write_events call for a specific event type"]
#[doc = r" Constructs the event from a Lua table using reflection and sends via EventWriter"]
pub fn dispatch_write_events(
    lua: &mlua::Lua,
    world: &mut bevy::prelude::World,
    event_type: &str,
    data: &mlua::Table,
) -> Result<(), String> {
    match event_type { "WindowResized" | "bevy_window::event::WindowResized" | "bevy::window::WindowResized" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowResized") . or_else (|| registry . get_with_type_path ("bevy_window::event::WindowResized")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowResized" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowResized as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowResized >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowResized") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowResized")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowResized")) ; } } , "RequestRedraw" | "bevy_window::event::RequestRedraw" | "bevy::window::RequestRedraw" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::RequestRedraw") . or_else (|| registry . get_with_type_path ("bevy_window::event::RequestRedraw")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::RequestRedraw" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: RequestRedraw as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: RequestRedraw >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::RequestRedraw") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::RequestRedraw")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::RequestRedraw")) ; } } , "WindowCreated" | "bevy_window::event::WindowCreated" | "bevy::window::WindowCreated" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowCreated") . or_else (|| registry . get_with_type_path ("bevy_window::event::WindowCreated")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowCreated" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowCreated as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowCreated >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowCreated") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowCreated")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowCreated")) ; } } , "WindowCloseRequested" | "bevy_window::event::WindowCloseRequested" | "bevy::window::WindowCloseRequested" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowCloseRequested") . or_else (|| registry . get_with_type_path ("bevy_window::event::WindowCloseRequested")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowCloseRequested" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowCloseRequested as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowCloseRequested >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowCloseRequested") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowCloseRequested")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowCloseRequested")) ; } } , "WindowClosed" | "bevy_window::event::WindowClosed" | "bevy::window::WindowClosed" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowClosed") . or_else (|| registry . get_with_type_path ("bevy_window::event::WindowClosed")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowClosed" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowClosed as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowClosed >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowClosed") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowClosed")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowClosed")) ; } } , "WindowClosing" | "bevy_window::event::WindowClosing" | "bevy::window::WindowClosing" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowClosing") . or_else (|| registry . get_with_type_path ("bevy_window::event::WindowClosing")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowClosing" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowClosing as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowClosing >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowClosing") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowClosing")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowClosing")) ; } } , "WindowDestroyed" | "bevy_window::event::WindowDestroyed" | "bevy::window::WindowDestroyed" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowDestroyed") . or_else (|| registry . get_with_type_path ("bevy_window::event::WindowDestroyed")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowDestroyed" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowDestroyed as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowDestroyed >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowDestroyed") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowDestroyed")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowDestroyed")) ; } } , "CursorMoved" | "bevy_window::event::CursorMoved" | "bevy::window::CursorMoved" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::CursorMoved") . or_else (|| registry . get_with_type_path ("bevy_window::event::CursorMoved")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::CursorMoved" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: CursorMoved as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: CursorMoved >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::CursorMoved") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::CursorMoved")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::CursorMoved")) ; } } , "CursorEntered" | "bevy_window::event::CursorEntered" | "bevy::window::CursorEntered" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::CursorEntered") . or_else (|| registry . get_with_type_path ("bevy_window::event::CursorEntered")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::CursorEntered" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: CursorEntered as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: CursorEntered >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::CursorEntered") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::CursorEntered")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::CursorEntered")) ; } } , "CursorLeft" | "bevy_window::event::CursorLeft" | "bevy::window::CursorLeft" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::CursorLeft") . or_else (|| registry . get_with_type_path ("bevy_window::event::CursorLeft")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::CursorLeft" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: CursorLeft as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: CursorLeft >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::CursorLeft") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::CursorLeft")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::CursorLeft")) ; } } , "WindowFocused" | "bevy_window::event::WindowFocused" | "bevy::window::WindowFocused" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowFocused") . or_else (|| registry . get_with_type_path ("bevy_window::event::WindowFocused")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowFocused" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowFocused as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowFocused >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowFocused") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowFocused")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowFocused")) ; } } , "WindowOccluded" | "bevy_window::event::WindowOccluded" | "bevy::window::WindowOccluded" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowOccluded") . or_else (|| registry . get_with_type_path ("bevy_window::event::WindowOccluded")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowOccluded" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowOccluded as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowOccluded >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowOccluded") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowOccluded")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowOccluded")) ; } } , "WindowScaleFactorChanged" | "bevy_window::event::WindowScaleFactorChanged" | "bevy::window::WindowScaleFactorChanged" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowScaleFactorChanged") . or_else (|| registry . get_with_type_path ("bevy_window::event::WindowScaleFactorChanged")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowScaleFactorChanged" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowScaleFactorChanged as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowScaleFactorChanged >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowScaleFactorChanged") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowScaleFactorChanged")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowScaleFactorChanged")) ; } } , "WindowBackendScaleFactorChanged" | "bevy_window::event::WindowBackendScaleFactorChanged" | "bevy::window::WindowBackendScaleFactorChanged" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowBackendScaleFactorChanged") . or_else (|| registry . get_with_type_path ("bevy_window::event::WindowBackendScaleFactorChanged")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowBackendScaleFactorChanged" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowBackendScaleFactorChanged as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowBackendScaleFactorChanged >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowBackendScaleFactorChanged") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowBackendScaleFactorChanged")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowBackendScaleFactorChanged")) ; } } , "FileDragAndDrop" | "bevy_window::event::FileDragAndDrop" | "bevy::window::FileDragAndDrop" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::FileDragAndDrop") . or_else (|| registry . get_with_type_path ("bevy_window::event::FileDragAndDrop")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::FileDragAndDrop" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: FileDragAndDrop as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: FileDragAndDrop >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::FileDragAndDrop") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::FileDragAndDrop")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::FileDragAndDrop")) ; } } , "WindowMoved" | "bevy_window::event::WindowMoved" | "bevy::window::WindowMoved" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowMoved") . or_else (|| registry . get_with_type_path ("bevy_window::event::WindowMoved")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowMoved" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowMoved as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowMoved >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowMoved") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowMoved")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowMoved")) ; } } , "WindowThemeChanged" | "bevy_window::event::WindowThemeChanged" | "bevy::window::WindowThemeChanged" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowThemeChanged") . or_else (|| registry . get_with_type_path ("bevy_window::event::WindowThemeChanged")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowThemeChanged" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowThemeChanged as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowThemeChanged >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowThemeChanged") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowThemeChanged")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowThemeChanged")) ; } } , "AppLifecycle" | "bevy_window::event::AppLifecycle" | "bevy::window::AppLifecycle" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::AppLifecycle") . or_else (|| registry . get_with_type_path ("bevy_window::event::AppLifecycle")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::AppLifecycle" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: AppLifecycle as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: AppLifecycle >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::AppLifecycle") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::AppLifecycle")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::AppLifecycle")) ; } } , "KeyboardInput" | "bevy_input::keyboard::KeyboardInput" | "bevy::input::keyboard::KeyboardInput" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::input::keyboard::KeyboardInput") . or_else (|| registry . get_with_type_path ("bevy_input::keyboard::KeyboardInput")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::input::keyboard::KeyboardInput" , e)) ? ; if let Some (concrete_event) = < bevy :: input :: keyboard :: KeyboardInput as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: input :: keyboard :: KeyboardInput >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::input::keyboard::KeyboardInput") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::input::keyboard::KeyboardInput")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::input::keyboard::KeyboardInput")) ; } } , "KeyboardFocusLost" | "bevy_input::keyboard::KeyboardFocusLost" | "bevy::input::keyboard::KeyboardFocusLost" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::input::keyboard::KeyboardFocusLost") . or_else (|| registry . get_with_type_path ("bevy_input::keyboard::KeyboardFocusLost")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::input::keyboard::KeyboardFocusLost" , e)) ? ; if let Some (concrete_event) = < bevy :: input :: keyboard :: KeyboardFocusLost as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: input :: keyboard :: KeyboardFocusLost >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::input::keyboard::KeyboardFocusLost") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::input::keyboard::KeyboardFocusLost")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::input::keyboard::KeyboardFocusLost")) ; } } , "MouseButtonInput" | "bevy_input::mouse::MouseButtonInput" | "bevy::input::mouse::MouseButtonInput" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::input::mouse::MouseButtonInput") . or_else (|| registry . get_with_type_path ("bevy_input::mouse::MouseButtonInput")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::input::mouse::MouseButtonInput" , e)) ? ; if let Some (concrete_event) = < bevy :: input :: mouse :: MouseButtonInput as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: input :: mouse :: MouseButtonInput >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::input::mouse::MouseButtonInput") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::input::mouse::MouseButtonInput")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::input::mouse::MouseButtonInput")) ; } } , "MouseMotion" | "bevy_input::mouse::MouseMotion" | "bevy::input::mouse::MouseMotion" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::input::mouse::MouseMotion") . or_else (|| registry . get_with_type_path ("bevy_input::mouse::MouseMotion")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::input::mouse::MouseMotion" , e)) ? ; if let Some (concrete_event) = < bevy :: input :: mouse :: MouseMotion as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: input :: mouse :: MouseMotion >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::input::mouse::MouseMotion") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::input::mouse::MouseMotion")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::input::mouse::MouseMotion")) ; } } , "MouseWheel" | "bevy_input::mouse::MouseWheel" | "bevy::input::mouse::MouseWheel" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::input::mouse::MouseWheel") . or_else (|| registry . get_with_type_path ("bevy_input::mouse::MouseWheel")) { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::input::mouse::MouseWheel" , e)) ? ; if let Some (concrete_event) = < bevy :: input :: mouse :: MouseWheel as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: input :: mouse :: MouseWheel >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::input::mouse::MouseWheel") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::input::mouse::MouseWheel")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::input::mouse::MouseWheel")) ; } } _ => Err (format ! ("Unknown event type: '{}'. Available events are discovered from bevy_window and bevy_input." , event_type)) }
}
#[doc = r" Dispatch write_message call for a specific message type"]
#[doc = r" Uses MessageWriter<T> and lua_table_to_dynamic for reflection-based construction"]
pub fn dispatch_write_message(
    lua: &mlua::Lua,
    world: &mut bevy::prelude::World,
    message_type: &str,
    data: &mlua::Table,
) -> Result<(), String> {
    match message_type {
        "AssetDeleteEvent"
        | "hello::asset_events::AssetDeleteEvent"
        | "hello::asset_events::AssetDeleteEvent" => {
            let type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let registry = type_registry.read();
            let type_registration = registry
                .get_with_type_path("hello::asset_events::AssetDeleteEvent")
                .or_else(|| registry.get_with_type_path("hello::asset_events::AssetDeleteEvent"));
            if let Some(type_registration) = type_registration {
                let type_info = type_registration.type_info();
                let asset_registry = world.get_resource::<bevy_lua_ecs::AssetRegistry>().cloned();
                let dynamic = bevy_lua_ecs::lua_table_to_dynamic_with_assets(
                    lua,
                    data,
                    type_info,
                    &type_registry,
                    asset_registry.as_ref(),
                )
                .map_err(|e| {
                    format!(
                        "Failed to build message '{}': {}",
                        "hello::asset_events::AssetDeleteEvent", e
                    )
                })?;
                use bevy::reflect::Struct;
                for i in 0..dynamic.field_len() {
                    let field_name = dynamic.name_at(i).unwrap_or("unknown");
                    let field_value = dynamic
                        .field_at(i)
                        .map(|f| {
                            format!("{} (kind: {:?})", f.reflect_type_path(), f.reflect_kind())
                        })
                        .unwrap_or("None".to_string());
                    bevy::log::debug!(
                        "[MESSAGE_CONSTRUCT] Field '{}': {}",
                        field_name,
                        field_value
                    );
                }
                if let Some(reflect_default) =
                    type_registration.data::<bevy::prelude::ReflectDefault>()
                {
                    let mut concrete_instance = reflect_default.default();
                    match concrete_instance.try_apply(&dynamic) {
                        Ok(()) => {
                            if let Ok(concrete_message) =
                                concrete_instance.take::<crate::asset_events::AssetDeleteEvent>()
                            {
                                drop(registry);
                                let mut system_state = bevy::ecs::system::SystemState::<
                                    bevy::prelude::MessageWriter<
                                        crate::asset_events::AssetDeleteEvent,
                                    >,
                                >::new(world);
                                let mut message_writer = system_state.get_mut(world);
                                message_writer.write(concrete_message);
                                bevy::log::debug!(
                                    "[MESSAGE_WRITE] Sent message via try_apply: {}",
                                    "hello::asset_events::AssetDeleteEvent"
                                );
                                return Ok(());
                            } else {
                                bevy :: log :: warn ! ("[MESSAGE_WRITE] try_apply succeeded but downcast failed for '{}'" , "hello::asset_events::AssetDeleteEvent");
                            }
                        }
                        Err(e) => {
                            bevy::log::debug!(
                                "[MESSAGE_WRITE] try_apply failed for '{}': {:?}",
                                "hello::asset_events::AssetDeleteEvent",
                                e
                            );
                        }
                    }
                }
                if let Some(reflect_from_reflect) =
                    type_registration.data::<bevy::reflect::ReflectFromReflect>()
                {
                    if let Some(concrete_value) = reflect_from_reflect.from_reflect(&dynamic) {
                        if let Ok(concrete_message) =
                            concrete_value.take::<crate::asset_events::AssetDeleteEvent>()
                        {
                            drop(registry);
                            let mut system_state = bevy::ecs::system::SystemState::<
                                bevy::prelude::MessageWriter<crate::asset_events::AssetDeleteEvent>,
                            >::new(world);
                            let mut message_writer = system_state.get_mut(world);
                            message_writer.write(concrete_message);
                            bevy::log::debug!(
                                "[MESSAGE_WRITE] Sent message via ReflectFromReflect: {}",
                                "hello::asset_events::AssetDeleteEvent"
                            );
                            return Ok(());
                        } else {
                            bevy :: log :: warn ! ("[MESSAGE_WRITE] ReflectFromReflect succeeded but downcast failed for '{}'" , "hello::asset_events::AssetDeleteEvent");
                        }
                    } else {
                        bevy :: log :: debug ! ("[MESSAGE_WRITE] ReflectFromReflect::from_reflect returned None for '{}'" , "hello::asset_events::AssetDeleteEvent");
                    }
                }
                if let Some (concrete_value) = < crate :: asset_events :: AssetDeleteEvent as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: MessageWriter < crate :: asset_events :: AssetDeleteEvent >> :: new (world) ; let mut message_writer = system_state . get_mut (world) ; message_writer . write (concrete_value) ; bevy :: log :: debug ! ("[MESSAGE_WRITE] Sent message via FromReflect trait: {}" , "hello::asset_events::AssetDeleteEvent") ; return Ok (()) ; }
                return Err (format ! ("Failed to construct message '{}' - all conversion strategies failed. This usually means a nested type doesn't implement FromReflect properly or a newtype wrapper is causing issues." , "hello::asset_events::AssetDeleteEvent")) ;
            } else {
                return Err(format!(
                    "Message type '{}' not found in TypeRegistry",
                    "hello::asset_events::AssetDeleteEvent"
                ));
            }
        }
        "AssetDirectoryListingEvent"
        | "hello::asset_events::AssetDirectoryListingEvent"
        | "hello::asset_events::AssetDirectoryListingEvent" => {
            let type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let registry = type_registry.read();
            let type_registration = registry
                .get_with_type_path("hello::asset_events::AssetDirectoryListingEvent")
                .or_else(|| {
                    registry.get_with_type_path("hello::asset_events::AssetDirectoryListingEvent")
                });
            if let Some(type_registration) = type_registration {
                let type_info = type_registration.type_info();
                let asset_registry = world.get_resource::<bevy_lua_ecs::AssetRegistry>().cloned();
                let dynamic = bevy_lua_ecs::lua_table_to_dynamic_with_assets(
                    lua,
                    data,
                    type_info,
                    &type_registry,
                    asset_registry.as_ref(),
                )
                .map_err(|e| {
                    format!(
                        "Failed to build message '{}': {}",
                        "hello::asset_events::AssetDirectoryListingEvent", e
                    )
                })?;
                use bevy::reflect::Struct;
                for i in 0..dynamic.field_len() {
                    let field_name = dynamic.name_at(i).unwrap_or("unknown");
                    let field_value = dynamic
                        .field_at(i)
                        .map(|f| {
                            format!("{} (kind: {:?})", f.reflect_type_path(), f.reflect_kind())
                        })
                        .unwrap_or("None".to_string());
                    bevy::log::debug!(
                        "[MESSAGE_CONSTRUCT] Field '{}': {}",
                        field_name,
                        field_value
                    );
                }
                if let Some(reflect_default) =
                    type_registration.data::<bevy::prelude::ReflectDefault>()
                {
                    let mut concrete_instance = reflect_default.default();
                    match concrete_instance.try_apply(&dynamic) {
                        Ok(()) => {
                            if let Ok(concrete_message) =
                                concrete_instance
                                    .take::<crate::asset_events::AssetDirectoryListingEvent>()
                            {
                                drop(registry);
                                let mut system_state = bevy::ecs::system::SystemState::<
                                    bevy::prelude::MessageWriter<
                                        crate::asset_events::AssetDirectoryListingEvent,
                                    >,
                                >::new(world);
                                let mut message_writer = system_state.get_mut(world);
                                message_writer.write(concrete_message);
                                bevy::log::debug!(
                                    "[MESSAGE_WRITE] Sent message via try_apply: {}",
                                    "hello::asset_events::AssetDirectoryListingEvent"
                                );
                                return Ok(());
                            } else {
                                bevy :: log :: warn ! ("[MESSAGE_WRITE] try_apply succeeded but downcast failed for '{}'" , "hello::asset_events::AssetDirectoryListingEvent");
                            }
                        }
                        Err(e) => {
                            bevy::log::debug!(
                                "[MESSAGE_WRITE] try_apply failed for '{}': {:?}",
                                "hello::asset_events::AssetDirectoryListingEvent",
                                e
                            );
                        }
                    }
                }
                if let Some(reflect_from_reflect) =
                    type_registration.data::<bevy::reflect::ReflectFromReflect>()
                {
                    if let Some(concrete_value) = reflect_from_reflect.from_reflect(&dynamic) {
                        if let Ok(concrete_message) =
                            concrete_value.take::<crate::asset_events::AssetDirectoryListingEvent>()
                        {
                            drop(registry);
                            let mut system_state = bevy::ecs::system::SystemState::<
                                bevy::prelude::MessageWriter<
                                    crate::asset_events::AssetDirectoryListingEvent,
                                >,
                            >::new(world);
                            let mut message_writer = system_state.get_mut(world);
                            message_writer.write(concrete_message);
                            bevy::log::debug!(
                                "[MESSAGE_WRITE] Sent message via ReflectFromReflect: {}",
                                "hello::asset_events::AssetDirectoryListingEvent"
                            );
                            return Ok(());
                        } else {
                            bevy :: log :: warn ! ("[MESSAGE_WRITE] ReflectFromReflect succeeded but downcast failed for '{}'" , "hello::asset_events::AssetDirectoryListingEvent");
                        }
                    } else {
                        bevy :: log :: debug ! ("[MESSAGE_WRITE] ReflectFromReflect::from_reflect returned None for '{}'" , "hello::asset_events::AssetDirectoryListingEvent");
                    }
                }
                if let Some (concrete_value) = < crate :: asset_events :: AssetDirectoryListingEvent as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: MessageWriter < crate :: asset_events :: AssetDirectoryListingEvent >> :: new (world) ; let mut message_writer = system_state . get_mut (world) ; message_writer . write (concrete_value) ; bevy :: log :: debug ! ("[MESSAGE_WRITE] Sent message via FromReflect trait: {}" , "hello::asset_events::AssetDirectoryListingEvent") ; return Ok (()) ; }
                return Err (format ! ("Failed to construct message '{}' - all conversion strategies failed. This usually means a nested type doesn't implement FromReflect properly or a newtype wrapper is causing issues." , "hello::asset_events::AssetDirectoryListingEvent")) ;
            } else {
                return Err(format!(
                    "Message type '{}' not found in TypeRegistry",
                    "hello::asset_events::AssetDirectoryListingEvent"
                ));
            }
        }
        "AssetLocalNewerEvent"
        | "hello::asset_events::AssetLocalNewerEvent"
        | "hello::asset_events::AssetLocalNewerEvent" => {
            let type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let registry = type_registry.read();
            let type_registration = registry
                .get_with_type_path("hello::asset_events::AssetLocalNewerEvent")
                .or_else(|| {
                    registry.get_with_type_path("hello::asset_events::AssetLocalNewerEvent")
                });
            if let Some(type_registration) = type_registration {
                let type_info = type_registration.type_info();
                let asset_registry = world.get_resource::<bevy_lua_ecs::AssetRegistry>().cloned();
                let dynamic = bevy_lua_ecs::lua_table_to_dynamic_with_assets(
                    lua,
                    data,
                    type_info,
                    &type_registry,
                    asset_registry.as_ref(),
                )
                .map_err(|e| {
                    format!(
                        "Failed to build message '{}': {}",
                        "hello::asset_events::AssetLocalNewerEvent", e
                    )
                })?;
                use bevy::reflect::Struct;
                for i in 0..dynamic.field_len() {
                    let field_name = dynamic.name_at(i).unwrap_or("unknown");
                    let field_value = dynamic
                        .field_at(i)
                        .map(|f| {
                            format!("{} (kind: {:?})", f.reflect_type_path(), f.reflect_kind())
                        })
                        .unwrap_or("None".to_string());
                    bevy::log::debug!(
                        "[MESSAGE_CONSTRUCT] Field '{}': {}",
                        field_name,
                        field_value
                    );
                }
                if let Some(reflect_default) =
                    type_registration.data::<bevy::prelude::ReflectDefault>()
                {
                    let mut concrete_instance = reflect_default.default();
                    match concrete_instance.try_apply(&dynamic) {
                        Ok(()) => {
                            if let Ok(concrete_message) = concrete_instance
                                .take::<crate::asset_events::AssetLocalNewerEvent>(
                            ) {
                                drop(registry);
                                let mut system_state = bevy::ecs::system::SystemState::<
                                    bevy::prelude::MessageWriter<
                                        crate::asset_events::AssetLocalNewerEvent,
                                    >,
                                >::new(world);
                                let mut message_writer = system_state.get_mut(world);
                                message_writer.write(concrete_message);
                                bevy::log::debug!(
                                    "[MESSAGE_WRITE] Sent message via try_apply: {}",
                                    "hello::asset_events::AssetLocalNewerEvent"
                                );
                                return Ok(());
                            } else {
                                bevy :: log :: warn ! ("[MESSAGE_WRITE] try_apply succeeded but downcast failed for '{}'" , "hello::asset_events::AssetLocalNewerEvent");
                            }
                        }
                        Err(e) => {
                            bevy::log::debug!(
                                "[MESSAGE_WRITE] try_apply failed for '{}': {:?}",
                                "hello::asset_events::AssetLocalNewerEvent",
                                e
                            );
                        }
                    }
                }
                if let Some(reflect_from_reflect) =
                    type_registration.data::<bevy::reflect::ReflectFromReflect>()
                {
                    if let Some(concrete_value) = reflect_from_reflect.from_reflect(&dynamic) {
                        if let Ok(concrete_message) =
                            concrete_value.take::<crate::asset_events::AssetLocalNewerEvent>()
                        {
                            drop(registry);
                            let mut system_state = bevy::ecs::system::SystemState::<
                                bevy::prelude::MessageWriter<
                                    crate::asset_events::AssetLocalNewerEvent,
                                >,
                            >::new(world);
                            let mut message_writer = system_state.get_mut(world);
                            message_writer.write(concrete_message);
                            bevy::log::debug!(
                                "[MESSAGE_WRITE] Sent message via ReflectFromReflect: {}",
                                "hello::asset_events::AssetLocalNewerEvent"
                            );
                            return Ok(());
                        } else {
                            bevy :: log :: warn ! ("[MESSAGE_WRITE] ReflectFromReflect succeeded but downcast failed for '{}'" , "hello::asset_events::AssetLocalNewerEvent");
                        }
                    } else {
                        bevy :: log :: debug ! ("[MESSAGE_WRITE] ReflectFromReflect::from_reflect returned None for '{}'" , "hello::asset_events::AssetLocalNewerEvent");
                    }
                }
                if let Some (concrete_value) = < crate :: asset_events :: AssetLocalNewerEvent as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: MessageWriter < crate :: asset_events :: AssetLocalNewerEvent >> :: new (world) ; let mut message_writer = system_state . get_mut (world) ; message_writer . write (concrete_value) ; bevy :: log :: debug ! ("[MESSAGE_WRITE] Sent message via FromReflect trait: {}" , "hello::asset_events::AssetLocalNewerEvent") ; return Ok (()) ; }
                return Err (format ! ("Failed to construct message '{}' - all conversion strategies failed. This usually means a nested type doesn't implement FromReflect properly or a newtype wrapper is causing issues." , "hello::asset_events::AssetLocalNewerEvent")) ;
            } else {
                return Err(format!(
                    "Message type '{}' not found in TypeRegistry",
                    "hello::asset_events::AssetLocalNewerEvent"
                ));
            }
        }
        "AssetRenameEvent"
        | "hello::asset_events::AssetRenameEvent"
        | "hello::asset_events::AssetRenameEvent" => {
            let type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let registry = type_registry.read();
            let type_registration = registry
                .get_with_type_path("hello::asset_events::AssetRenameEvent")
                .or_else(|| registry.get_with_type_path("hello::asset_events::AssetRenameEvent"));
            if let Some(type_registration) = type_registration {
                let type_info = type_registration.type_info();
                let asset_registry = world.get_resource::<bevy_lua_ecs::AssetRegistry>().cloned();
                let dynamic = bevy_lua_ecs::lua_table_to_dynamic_with_assets(
                    lua,
                    data,
                    type_info,
                    &type_registry,
                    asset_registry.as_ref(),
                )
                .map_err(|e| {
                    format!(
                        "Failed to build message '{}': {}",
                        "hello::asset_events::AssetRenameEvent", e
                    )
                })?;
                use bevy::reflect::Struct;
                for i in 0..dynamic.field_len() {
                    let field_name = dynamic.name_at(i).unwrap_or("unknown");
                    let field_value = dynamic
                        .field_at(i)
                        .map(|f| {
                            format!("{} (kind: {:?})", f.reflect_type_path(), f.reflect_kind())
                        })
                        .unwrap_or("None".to_string());
                    bevy::log::debug!(
                        "[MESSAGE_CONSTRUCT] Field '{}': {}",
                        field_name,
                        field_value
                    );
                }
                if let Some(reflect_default) =
                    type_registration.data::<bevy::prelude::ReflectDefault>()
                {
                    let mut concrete_instance = reflect_default.default();
                    match concrete_instance.try_apply(&dynamic) {
                        Ok(()) => {
                            if let Ok(concrete_message) =
                                concrete_instance.take::<crate::asset_events::AssetRenameEvent>()
                            {
                                drop(registry);
                                let mut system_state = bevy::ecs::system::SystemState::<
                                    bevy::prelude::MessageWriter<
                                        crate::asset_events::AssetRenameEvent,
                                    >,
                                >::new(world);
                                let mut message_writer = system_state.get_mut(world);
                                message_writer.write(concrete_message);
                                bevy::log::debug!(
                                    "[MESSAGE_WRITE] Sent message via try_apply: {}",
                                    "hello::asset_events::AssetRenameEvent"
                                );
                                return Ok(());
                            } else {
                                bevy :: log :: warn ! ("[MESSAGE_WRITE] try_apply succeeded but downcast failed for '{}'" , "hello::asset_events::AssetRenameEvent");
                            }
                        }
                        Err(e) => {
                            bevy::log::debug!(
                                "[MESSAGE_WRITE] try_apply failed for '{}': {:?}",
                                "hello::asset_events::AssetRenameEvent",
                                e
                            );
                        }
                    }
                }
                if let Some(reflect_from_reflect) =
                    type_registration.data::<bevy::reflect::ReflectFromReflect>()
                {
                    if let Some(concrete_value) = reflect_from_reflect.from_reflect(&dynamic) {
                        if let Ok(concrete_message) =
                            concrete_value.take::<crate::asset_events::AssetRenameEvent>()
                        {
                            drop(registry);
                            let mut system_state = bevy::ecs::system::SystemState::<
                                bevy::prelude::MessageWriter<crate::asset_events::AssetRenameEvent>,
                            >::new(world);
                            let mut message_writer = system_state.get_mut(world);
                            message_writer.write(concrete_message);
                            bevy::log::debug!(
                                "[MESSAGE_WRITE] Sent message via ReflectFromReflect: {}",
                                "hello::asset_events::AssetRenameEvent"
                            );
                            return Ok(());
                        } else {
                            bevy :: log :: warn ! ("[MESSAGE_WRITE] ReflectFromReflect succeeded but downcast failed for '{}'" , "hello::asset_events::AssetRenameEvent");
                        }
                    } else {
                        bevy :: log :: debug ! ("[MESSAGE_WRITE] ReflectFromReflect::from_reflect returned None for '{}'" , "hello::asset_events::AssetRenameEvent");
                    }
                }
                if let Some (concrete_value) = < crate :: asset_events :: AssetRenameEvent as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: MessageWriter < crate :: asset_events :: AssetRenameEvent >> :: new (world) ; let mut message_writer = system_state . get_mut (world) ; message_writer . write (concrete_value) ; bevy :: log :: debug ! ("[MESSAGE_WRITE] Sent message via FromReflect trait: {}" , "hello::asset_events::AssetRenameEvent") ; return Ok (()) ; }
                return Err (format ! ("Failed to construct message '{}' - all conversion strategies failed. This usually means a nested type doesn't implement FromReflect properly or a newtype wrapper is causing issues." , "hello::asset_events::AssetRenameEvent")) ;
            } else {
                return Err(format!(
                    "Message type '{}' not found in TypeRegistry",
                    "hello::asset_events::AssetRenameEvent"
                ));
            }
        }
        "AssetUploadProgressEvent"
        | "hello::asset_events::AssetUploadProgressEvent"
        | "hello::asset_events::AssetUploadProgressEvent" => {
            let type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let registry = type_registry.read();
            let type_registration = registry
                .get_with_type_path("hello::asset_events::AssetUploadProgressEvent")
                .or_else(|| {
                    registry.get_with_type_path("hello::asset_events::AssetUploadProgressEvent")
                });
            if let Some(type_registration) = type_registration {
                let type_info = type_registration.type_info();
                let asset_registry = world.get_resource::<bevy_lua_ecs::AssetRegistry>().cloned();
                let dynamic = bevy_lua_ecs::lua_table_to_dynamic_with_assets(
                    lua,
                    data,
                    type_info,
                    &type_registry,
                    asset_registry.as_ref(),
                )
                .map_err(|e| {
                    format!(
                        "Failed to build message '{}': {}",
                        "hello::asset_events::AssetUploadProgressEvent", e
                    )
                })?;
                use bevy::reflect::Struct;
                for i in 0..dynamic.field_len() {
                    let field_name = dynamic.name_at(i).unwrap_or("unknown");
                    let field_value = dynamic
                        .field_at(i)
                        .map(|f| {
                            format!("{} (kind: {:?})", f.reflect_type_path(), f.reflect_kind())
                        })
                        .unwrap_or("None".to_string());
                    bevy::log::debug!(
                        "[MESSAGE_CONSTRUCT] Field '{}': {}",
                        field_name,
                        field_value
                    );
                }
                if let Some(reflect_default) =
                    type_registration.data::<bevy::prelude::ReflectDefault>()
                {
                    let mut concrete_instance = reflect_default.default();
                    match concrete_instance.try_apply(&dynamic) {
                        Ok(()) => {
                            if let Ok(concrete_message) =
                                concrete_instance
                                    .take::<crate::asset_events::AssetUploadProgressEvent>()
                            {
                                drop(registry);
                                let mut system_state = bevy::ecs::system::SystemState::<
                                    bevy::prelude::MessageWriter<
                                        crate::asset_events::AssetUploadProgressEvent,
                                    >,
                                >::new(world);
                                let mut message_writer = system_state.get_mut(world);
                                message_writer.write(concrete_message);
                                bevy::log::debug!(
                                    "[MESSAGE_WRITE] Sent message via try_apply: {}",
                                    "hello::asset_events::AssetUploadProgressEvent"
                                );
                                return Ok(());
                            } else {
                                bevy :: log :: warn ! ("[MESSAGE_WRITE] try_apply succeeded but downcast failed for '{}'" , "hello::asset_events::AssetUploadProgressEvent");
                            }
                        }
                        Err(e) => {
                            bevy::log::debug!(
                                "[MESSAGE_WRITE] try_apply failed for '{}': {:?}",
                                "hello::asset_events::AssetUploadProgressEvent",
                                e
                            );
                        }
                    }
                }
                if let Some(reflect_from_reflect) =
                    type_registration.data::<bevy::reflect::ReflectFromReflect>()
                {
                    if let Some(concrete_value) = reflect_from_reflect.from_reflect(&dynamic) {
                        if let Ok(concrete_message) =
                            concrete_value.take::<crate::asset_events::AssetUploadProgressEvent>()
                        {
                            drop(registry);
                            let mut system_state = bevy::ecs::system::SystemState::<
                                bevy::prelude::MessageWriter<
                                    crate::asset_events::AssetUploadProgressEvent,
                                >,
                            >::new(world);
                            let mut message_writer = system_state.get_mut(world);
                            message_writer.write(concrete_message);
                            bevy::log::debug!(
                                "[MESSAGE_WRITE] Sent message via ReflectFromReflect: {}",
                                "hello::asset_events::AssetUploadProgressEvent"
                            );
                            return Ok(());
                        } else {
                            bevy :: log :: warn ! ("[MESSAGE_WRITE] ReflectFromReflect succeeded but downcast failed for '{}'" , "hello::asset_events::AssetUploadProgressEvent");
                        }
                    } else {
                        bevy :: log :: debug ! ("[MESSAGE_WRITE] ReflectFromReflect::from_reflect returned None for '{}'" , "hello::asset_events::AssetUploadProgressEvent");
                    }
                }
                if let Some (concrete_value) = < crate :: asset_events :: AssetUploadProgressEvent as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: MessageWriter < crate :: asset_events :: AssetUploadProgressEvent >> :: new (world) ; let mut message_writer = system_state . get_mut (world) ; message_writer . write (concrete_value) ; bevy :: log :: debug ! ("[MESSAGE_WRITE] Sent message via FromReflect trait: {}" , "hello::asset_events::AssetUploadProgressEvent") ; return Ok (()) ; }
                return Err (format ! ("Failed to construct message '{}' - all conversion strategies failed. This usually means a nested type doesn't implement FromReflect properly or a newtype wrapper is causing issues." , "hello::asset_events::AssetUploadProgressEvent")) ;
            } else {
                return Err(format!(
                    "Message type '{}' not found in TypeRegistry",
                    "hello::asset_events::AssetUploadProgressEvent"
                ));
            }
        }
        "PointerInput"
        | "bevy::picking::pointer::PointerInput"
        | "bevy_picking::pointer::PointerInput" => {
            let type_registry = world
                .resource::<bevy::ecs::reflect::AppTypeRegistry>()
                .clone();
            let registry = type_registry.read();
            let type_registration = registry
                .get_with_type_path("bevy_picking::pointer::PointerInput")
                .or_else(|| registry.get_with_type_path("bevy::picking::pointer::PointerInput"));
            if let Some(type_registration) = type_registration {
                let type_info = type_registration.type_info();
                let asset_registry = world.get_resource::<bevy_lua_ecs::AssetRegistry>().cloned();
                let dynamic = bevy_lua_ecs::lua_table_to_dynamic_with_assets(
                    lua,
                    data,
                    type_info,
                    &type_registry,
                    asset_registry.as_ref(),
                )
                .map_err(|e| {
                    format!(
                        "Failed to build message '{}': {}",
                        "bevy::picking::pointer::PointerInput", e
                    )
                })?;
                use bevy::reflect::Struct;
                for i in 0..dynamic.field_len() {
                    let field_name = dynamic.name_at(i).unwrap_or("unknown");
                    let field_value = dynamic
                        .field_at(i)
                        .map(|f| {
                            format!("{} (kind: {:?})", f.reflect_type_path(), f.reflect_kind())
                        })
                        .unwrap_or("None".to_string());
                    bevy::log::debug!(
                        "[MESSAGE_CONSTRUCT] Field '{}': {}",
                        field_name,
                        field_value
                    );
                }
                if let Some(reflect_default) =
                    type_registration.data::<bevy::prelude::ReflectDefault>()
                {
                    let mut concrete_instance = reflect_default.default();
                    match concrete_instance.try_apply(&dynamic) {
                        Ok(()) => {
                            if let Ok(concrete_message) =
                                concrete_instance.take::<bevy::picking::pointer::PointerInput>()
                            {
                                drop(registry);
                                let mut system_state = bevy::ecs::system::SystemState::<
                                    bevy::prelude::MessageWriter<
                                        bevy::picking::pointer::PointerInput,
                                    >,
                                >::new(world);
                                let mut message_writer = system_state.get_mut(world);
                                message_writer.write(concrete_message);
                                bevy::log::debug!(
                                    "[MESSAGE_WRITE] Sent message via try_apply: {}",
                                    "bevy::picking::pointer::PointerInput"
                                );
                                return Ok(());
                            } else {
                                bevy :: log :: warn ! ("[MESSAGE_WRITE] try_apply succeeded but downcast failed for '{}'" , "bevy::picking::pointer::PointerInput");
                            }
                        }
                        Err(e) => {
                            bevy::log::debug!(
                                "[MESSAGE_WRITE] try_apply failed for '{}': {:?}",
                                "bevy::picking::pointer::PointerInput",
                                e
                            );
                        }
                    }
                }
                if let Some(reflect_from_reflect) =
                    type_registration.data::<bevy::reflect::ReflectFromReflect>()
                {
                    if let Some(concrete_value) = reflect_from_reflect.from_reflect(&dynamic) {
                        if let Ok(concrete_message) =
                            concrete_value.take::<bevy::picking::pointer::PointerInput>()
                        {
                            drop(registry);
                            let mut system_state = bevy::ecs::system::SystemState::<
                                bevy::prelude::MessageWriter<bevy::picking::pointer::PointerInput>,
                            >::new(world);
                            let mut message_writer = system_state.get_mut(world);
                            message_writer.write(concrete_message);
                            bevy::log::debug!(
                                "[MESSAGE_WRITE] Sent message via ReflectFromReflect: {}",
                                "bevy::picking::pointer::PointerInput"
                            );
                            return Ok(());
                        } else {
                            bevy :: log :: warn ! ("[MESSAGE_WRITE] ReflectFromReflect succeeded but downcast failed for '{}'" , "bevy::picking::pointer::PointerInput");
                        }
                    } else {
                        bevy :: log :: debug ! ("[MESSAGE_WRITE] ReflectFromReflect::from_reflect returned None for '{}'" , "bevy::picking::pointer::PointerInput");
                    }
                }
                if let Some (concrete_value) = < bevy :: picking :: pointer :: PointerInput as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: MessageWriter < bevy :: picking :: pointer :: PointerInput >> :: new (world) ; let mut message_writer = system_state . get_mut (world) ; message_writer . write (concrete_value) ; bevy :: log :: debug ! ("[MESSAGE_WRITE] Sent message via FromReflect trait: {}" , "bevy::picking::pointer::PointerInput") ; return Ok (()) ; }
                return Err (format ! ("Failed to construct message '{}' - all conversion strategies failed. This usually means a nested type doesn't implement FromReflect properly or a newtype wrapper is causing issues." , "bevy::picking::pointer::PointerInput")) ;
            } else {
                return Err(format!(
                    "Message type '{}' not found in TypeRegistry",
                    "bevy::picking::pointer::PointerInput"
                ));
            }
        }
        _ => Err(format!(
            "Unknown message type: '{}'. Discovered message types are auto-generated.",
            message_type
        )),
    }
}
#[doc = r" Dispatch a Lua observer callback for an entity with reflected event data"]
#[doc = r" The entire event is converted to a Lua table via reflection, making all fields available"]
fn dispatch_lua_observer_reflected<T: bevy::reflect::PartialReflect>(
    lua_ctx: &bevy_lua_ecs::LuaScriptContext,
    observer_registry: &bevy_lua_ecs::LuaObserverRegistry,
    update_queue: &bevy_lua_ecs::ComponentUpdateQueue,
    entity: bevy::prelude::Entity,
    event_type: &str,
    event_data: &T,
) {
    let callbacks = observer_registry.callbacks().lock().unwrap();
    if let Some(observers) = callbacks.get(&entity) {
        for (ev_type, callback_key) in observers {
            if ev_type == event_type {
                if let Ok(callback) = lua_ctx.lua.registry_value::<mlua::Function>(callback_key) {
                    let entity_snapshot = bevy_lua_ecs::LuaEntitySnapshot {
                        entity,
                        component_data: std::collections::HashMap::new(),
                        lua_components: std::collections::HashMap::new(),
                        update_queue: update_queue.clone(),
                    };
                    let event_table = match bevy_lua_ecs::reflection::try_reflect_to_lua_value(
                        &lua_ctx.lua,
                        event_data,
                    ) {
                        Ok(mlua::Value::Table(table)) => table,
                        Ok(other) => {
                            let table = lua_ctx.lua.create_table().unwrap();
                            let _ = table.set("value", other);
                            table
                        }
                        Err(e) => {
                            bevy::log::warn!(
                                "[LUA_OBSERVER] Error reflecting event {}: {}",
                                event_type,
                                e
                            );
                            lua_ctx.lua.create_table().unwrap()
                        }
                    };
                    if let Err(e) = callback.call::<()>((entity_snapshot, event_table)) {
                        bevy::log::error!(
                            "[LUA_OBSERVER] Error calling {} callback: {}",
                            event_type,
                            e
                        );
                    }
                }
            }
        }
    }
}
fn on_pointer_cancel_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::Cancel>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<Cancel>",
        event_data,
    );
}
fn on_pointer_over_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::Over>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<Over>",
        event_data,
    );
}
fn on_pointer_out_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::Out>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<Out>",
        event_data,
    );
}
fn on_pointer_press_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::Press>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<Press>",
        event_data,
    );
}
fn on_pointer_release_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::Release>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<Release>",
        event_data,
    );
}
fn on_pointer_click_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::Click>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<Click>",
        event_data,
    );
}
fn on_pointer_move_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::Move>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<Move>",
        event_data,
    );
}
fn on_pointer_dragstart_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::DragStart>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<DragStart>",
        event_data,
    );
}
fn on_pointer_drag_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::Drag>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<Drag>",
        event_data,
    );
}
fn on_pointer_dragend_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::DragEnd>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<DragEnd>",
        event_data,
    );
}
fn on_pointer_dragenter_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::DragEnter>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<DragEnter>",
        event_data,
    );
}
fn on_pointer_dragover_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::DragOver>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<DragOver>",
        event_data,
    );
}
fn on_pointer_dragleave_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::DragLeave>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<DragLeave>",
        event_data,
    );
}
fn on_pointer_dragdrop_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::DragDrop>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<DragDrop>",
        event_data,
    );
}
fn on_pointer_dragentry_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::DragEntry>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<DragEntry>",
        event_data,
    );
}
fn on_pointer_scroll_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::picking::events::Scroll>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let event_data = event.event();
    dispatch_lua_observer_reflected(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event_data.entity,
        "Pointer<Scroll>",
        event_data,
    );
}
#[doc = r" Attach a Lua observer to an entity by event type name"]
#[doc = r" This function is generated with match arms for all discovered observable events"]
pub fn attach_observer_by_name(
    commands: &mut bevy::prelude::Commands,
    entity: bevy::prelude::Entity,
    event_type: &str,
) {
    match event_type {
        "Pointer<Cancel>" => {
            commands.entity(entity).observe(on_pointer_cancel_lua);
        }
        "Pointer<Over>" => {
            commands.entity(entity).observe(on_pointer_over_lua);
        }
        "Pointer<Out>" => {
            commands.entity(entity).observe(on_pointer_out_lua);
        }
        "Pointer<Press>" => {
            commands.entity(entity).observe(on_pointer_press_lua);
        }
        "Pointer<Release>" => {
            commands.entity(entity).observe(on_pointer_release_lua);
        }
        "Pointer<Click>" => {
            commands.entity(entity).observe(on_pointer_click_lua);
        }
        "Pointer<Move>" => {
            commands.entity(entity).observe(on_pointer_move_lua);
        }
        "Pointer<DragStart>" => {
            commands.entity(entity).observe(on_pointer_dragstart_lua);
        }
        "Pointer<Drag>" => {
            commands.entity(entity).observe(on_pointer_drag_lua);
        }
        "Pointer<DragEnd>" => {
            commands.entity(entity).observe(on_pointer_dragend_lua);
        }
        "Pointer<DragEnter>" => {
            commands.entity(entity).observe(on_pointer_dragenter_lua);
        }
        "Pointer<DragOver>" => {
            commands.entity(entity).observe(on_pointer_dragover_lua);
        }
        "Pointer<DragLeave>" => {
            commands.entity(entity).observe(on_pointer_dragleave_lua);
        }
        "Pointer<DragDrop>" => {
            commands.entity(entity).observe(on_pointer_dragdrop_lua);
        }
        "Pointer<DragEntry>" => {
            commands.entity(entity).observe(on_pointer_dragentry_lua);
        }
        "Pointer<Scroll>" => {
            commands.entity(entity).observe(on_pointer_scroll_lua);
        }
        _ => bevy::log::warn!("[LUA_OBSERVER] Unknown observer type: {}", event_type),
    }
}
#[doc = r" Plugin that wraps LuaSpawnPlugin and automatically registers all auto-generated bindings."]
#[doc = r" Use this instead of LuaSpawnPlugin directly to get automatic bitflags, component bindings,"]
#[doc = r" handle setters, and asset adders registration."]
pub struct LuaBindingsPlugin;
impl bevy::prelude::Plugin for LuaBindingsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(bevy_lua_ecs::LuaSpawnPlugin);
        bevy_lua_ecs::set_observer_attacher(attach_observer_by_name);
        bevy_lua_ecs::set_systemparam_dispatcher(dispatch_systemparam_method);
        bevy_lua_ecs::set_event_dispatcher(dispatch_read_events);
        bevy_lua_ecs::set_event_write_dispatcher(dispatch_write_events);
        bevy_lua_ecs::set_message_write_dispatcher(dispatch_write_message);
        bevy_lua_ecs::set_component_method_dispatcher(dispatch_component_method);
        bevy_lua_ecs::set_static_method_dispatcher(dispatch_static_method);
        register_bevy_events(app);
        app.init_resource::<bevy_lua_ecs::BitflagsRegistry>();
        app.add_systems(bevy::prelude::Startup, setup_bitflags);
        app.add_systems(bevy::prelude::Startup, log_registered_events);
        app.add_systems(bevy::prelude::PostStartup, register_asset_constructors);
        app.add_systems(bevy::prelude::Update, bevy_lua_ecs::dispatch_lua_messages);
    }
}
#[doc = r" Debug system to log all registered Events<T> types in the TypeRegistry"]
fn log_registered_events(type_registry: bevy::prelude::Res<bevy::ecs::reflect::AppTypeRegistry>) {
    let registry = type_registry.read();
    bevy::log::info!("[DEBUG_EVENTS] === Scanning TypeRegistry for Events<*> types ===");
    let mut found_count = 0;
    for registration in registry.iter() {
        let type_path = registration.type_info().type_path();
        if type_path.contains("Events<") {
            bevy::log::info!("[DEBUG_EVENTS] Found: '{}'", type_path);
            found_count += 1;
        }
    }
    bevy::log::info!(
        "[DEBUG_EVENTS] Total Events<*> types found: {}",
        found_count
    );
}
#[doc = r" Register auto-discovered Bevy Event and Message types for Lua events/messages"]
fn register_bevy_events(app: &mut bevy::prelude::App) {
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::WindowResized"
    );
    app.add_event::<bevy::window::WindowResized>();
    app.register_type::<bevy::window::WindowResized>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::RequestRedraw"
    );
    app.add_event::<bevy::window::RequestRedraw>();
    app.register_type::<bevy::window::RequestRedraw>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::WindowCreated"
    );
    app.add_event::<bevy::window::WindowCreated>();
    app.register_type::<bevy::window::WindowCreated>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::WindowCloseRequested"
    );
    app.add_event::<bevy::window::WindowCloseRequested>();
    app.register_type::<bevy::window::WindowCloseRequested>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::WindowClosed"
    );
    app.add_event::<bevy::window::WindowClosed>();
    app.register_type::<bevy::window::WindowClosed>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::WindowClosing"
    );
    app.add_event::<bevy::window::WindowClosing>();
    app.register_type::<bevy::window::WindowClosing>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::WindowDestroyed"
    );
    app.add_event::<bevy::window::WindowDestroyed>();
    app.register_type::<bevy::window::WindowDestroyed>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::CursorMoved"
    );
    app.add_event::<bevy::window::CursorMoved>();
    app.register_type::<bevy::window::CursorMoved>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::CursorEntered"
    );
    app.add_event::<bevy::window::CursorEntered>();
    app.register_type::<bevy::window::CursorEntered>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::CursorLeft"
    );
    app.add_event::<bevy::window::CursorLeft>();
    app.register_type::<bevy::window::CursorLeft>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::WindowFocused"
    );
    app.add_event::<bevy::window::WindowFocused>();
    app.register_type::<bevy::window::WindowFocused>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::WindowOccluded"
    );
    app.add_event::<bevy::window::WindowOccluded>();
    app.register_type::<bevy::window::WindowOccluded>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::WindowScaleFactorChanged"
    );
    app.add_event::<bevy::window::WindowScaleFactorChanged>();
    app.register_type::<bevy::window::WindowScaleFactorChanged>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::WindowBackendScaleFactorChanged"
    );
    app.add_event::<bevy::window::WindowBackendScaleFactorChanged>();
    app.register_type::<bevy::window::WindowBackendScaleFactorChanged>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::FileDragAndDrop"
    );
    app.add_event::<bevy::window::FileDragAndDrop>();
    app.register_type::<bevy::window::FileDragAndDrop>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::WindowMoved"
    );
    app.add_event::<bevy::window::WindowMoved>();
    app.register_type::<bevy::window::WindowMoved>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::WindowThemeChanged"
    );
    app.add_event::<bevy::window::WindowThemeChanged>();
    app.register_type::<bevy::window::WindowThemeChanged>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::window::AppLifecycle"
    );
    app.add_event::<bevy::window::AppLifecycle>();
    app.register_type::<bevy::window::AppLifecycle>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::input::keyboard::KeyboardInput"
    );
    app.add_event::<bevy::input::keyboard::KeyboardInput>();
    app.register_type::<bevy::input::keyboard::KeyboardInput>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::input::keyboard::KeyboardFocusLost"
    );
    app.add_event::<bevy::input::keyboard::KeyboardFocusLost>();
    app.register_type::<bevy::input::keyboard::KeyboardFocusLost>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::input::mouse::MouseButtonInput"
    );
    app.add_event::<bevy::input::mouse::MouseButtonInput>();
    app.register_type::<bevy::input::mouse::MouseButtonInput>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::input::mouse::MouseMotion"
    );
    app.add_event::<bevy::input::mouse::MouseMotion>();
    app.register_type::<bevy::input::mouse::MouseMotion>();
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::input::mouse::MouseWheel"
    );
    app.add_event::<bevy::input::mouse::MouseWheel>();
    app.register_type::<bevy::input::mouse::MouseWheel>();
    app.register_type::<crate::asset_events::AssetDeleteEvent>();
    bevy::log::debug!(
        "[REGISTER_MESSAGES] Adding message type: {}",
        "hello::asset_events::AssetDeleteEvent"
    );
    app.register_type::<crate::asset_events::AssetDirectoryListingEvent>();
    bevy::log::debug!(
        "[REGISTER_MESSAGES] Adding message type: {}",
        "hello::asset_events::AssetDirectoryListingEvent"
    );
    app.register_type::<crate::asset_events::AssetLocalNewerEvent>();
    bevy::log::debug!(
        "[REGISTER_MESSAGES] Adding message type: {}",
        "hello::asset_events::AssetLocalNewerEvent"
    );
    app.register_type::<crate::asset_events::AssetRenameEvent>();
    bevy::log::debug!(
        "[REGISTER_MESSAGES] Adding message type: {}",
        "hello::asset_events::AssetRenameEvent"
    );
    app.register_type::<crate::asset_events::AssetUploadProgressEvent>();
    bevy::log::debug!(
        "[REGISTER_MESSAGES] Adding message type: {}",
        "hello::asset_events::AssetUploadProgressEvent"
    );
    app.register_type::<bevy::picking::pointer::PointerInput>();
    bevy::log::debug!(
        "[REGISTER_MESSAGES] Adding message type: {}",
        "bevy::picking::pointer::PointerInput"
    );
    app.register_type::<bevy::input::ButtonInput<bevy::input::keyboard::KeyCode>>();
    bevy::log::debug!(
        "[REGISTER_RESOURCES] Adding resource type: {}",
        "ButtonInput<KeyCode>"
    );
    app.register_type::<bevy::input::ButtonInput<bevy::input::mouse::MouseButton>>();
    bevy::log::debug!(
        "[REGISTER_RESOURCES] Adding resource type: {}",
        "ButtonInput<MouseButton>"
    );
    app.register_type::<bevy::input::ButtonInput<bevy::input::gamepad::GamepadButton>>();
    bevy::log::debug!(
        "[REGISTER_RESOURCES] Adding resource type: {}",
        "ButtonInput<GamepadButton>"
    );
    app.register_type::<bevy::input::Axis<bevy::input::gamepad::GamepadAxis>>();
    bevy::log::debug!(
        "[REGISTER_RESOURCES] Adding resource type: {}",
        "Axis<GamepadAxis>"
    );
    app.register_type::<bevy::input::Axis<bevy::input::gamepad::GamepadButton>>();
    bevy::log::debug!(
        "[REGISTER_RESOURCES] Adding resource type: {}",
        "Axis<GamepadButton>"
    );
    app.add_systems(bevy::prelude::PreUpdate, accumulate_events_for_lua);
    bevy::log::debug!("Auto-discovered Bevy Events, Messages, and Resources registered for Lua");
}
#[doc = r" Accumulator system that runs every frame to capture events for Lua scripts"]
#[doc = r" Events are copied to all active script instances so each can read them independently"]
fn accumulate_events_for_lua(world: &mut bevy::prelude::World) {
    let accumulator = world
        .resource::<bevy_lua_ecs::LuaEventAccumulator>()
        .clone();
    let script_registry = world.resource::<bevy_lua_ecs::ScriptRegistry>().clone();
    let active_instances = script_registry.all_active_instance_ids();
    if active_instances.is_empty() {
        return;
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::WindowResized>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "WindowResized", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::RequestRedraw>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "RequestRedraw", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::WindowCreated>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "WindowCreated", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::WindowCloseRequested>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(
                    &active_instances,
                    "WindowCloseRequested",
                    event_json,
                );
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::WindowClosed>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "WindowClosed", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::WindowClosing>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "WindowClosing", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::WindowDestroyed>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "WindowDestroyed", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::CursorMoved>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "CursorMoved", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::CursorEntered>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "CursorEntered", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::CursorLeft>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "CursorLeft", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::WindowFocused>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "WindowFocused", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::WindowOccluded>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "WindowOccluded", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::WindowScaleFactorChanged>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(
                    &active_instances,
                    "WindowScaleFactorChanged",
                    event_json,
                );
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::WindowBackendScaleFactorChanged>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(
                    &active_instances,
                    "WindowBackendScaleFactorChanged",
                    event_json,
                );
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::FileDragAndDrop>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "FileDragAndDrop", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::WindowMoved>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "WindowMoved", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::WindowThemeChanged>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "WindowThemeChanged", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::window::AppLifecycle>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "AppLifecycle", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::input::keyboard::KeyboardInput>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "KeyboardInput", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::input::keyboard::KeyboardFocusLost>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "KeyboardFocusLost", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::input::mouse::MouseButtonInput>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "MouseButtonInput", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::input::mouse::MouseMotion>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "MouseMotion", event_json);
            }
        }
    }
    {
        let mut system_state = bevy::ecs::system::SystemState::<
            bevy::prelude::EventReader<bevy::input::mouse::MouseWheel>,
        >::new(world);
        let mut event_reader = system_state.get_mut(world);
        for event in event_reader.read() {
            if let Ok(event_json) = serde_json::to_value(event) {
                accumulator.push_to_instances(&active_instances, "MouseWheel", event_json);
            }
        }
    }
}
#[doc = r" System to register auto-generated bitflags types"]
fn setup_bitflags(registry: bevy::prelude::Res<bevy_lua_ecs::BitflagsRegistry>) {
    register_auto_bitflags(&registry);
    bevy::log::debug!("Auto-generated bitflags registered");
}
#[doc = r" System to register auto-generated asset constructors, handle setters, and component bindings"]
fn register_asset_constructors(
    asset_registry: bevy::prelude::Res<bevy_lua_ecs::AssetRegistry>,
    type_registry: bevy::prelude::Res<bevy::ecs::reflect::AppTypeRegistry>,
    mut component_registry: bevy::prelude::ResMut<bevy_lua_ecs::ComponentRegistry>,
) {
    register_entity_wrappers_from_registry(&mut component_registry, &type_registry);
    register_asset_types_from_registry(&asset_registry, &type_registry);
    register_auto_newtype_wrappers(&asset_registry.newtype_wrappers);
    register_asset_cloners(&asset_registry);
    register_asset_constructor_bindings(&asset_registry);
    register_auto_typed_path_loaders(&asset_registry, &type_registry);
    bevy::log::debug!(
        "Auto-generated asset constructors, component bindings, and newtype wrappers registered"
    );
}
#[doc = r" Register asset cloners for types that implement Clone"]
#[doc = r" This is auto-generated based on compile-time detection of Clone derives/impls"]
fn register_asset_cloners(asset_registry: &bevy_lua_ecs::AssetRegistry) {
    let mut cloners = asset_registry.asset_cloners_by_typeid.lock().unwrap();
    bevy_lua_ecs::register_cloner_if_clone::<bevy::animation::AnimationClip>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::audio::AudioSource>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::audio::Pitch>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::gizmos::GizmoAsset>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::image::Image>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::image::TextureAtlasLayout>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::mesh::Mesh>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::pbr::StandardMaterial>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::shader::Shader>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::sprite_render::ColorMaterial>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::text::Font>(&mut cloners);
    bevy::log::debug!(
        "[ASSET_CLONER] Registered {} asset cloners (types with Clone impl)",
        cloners.len()
    );
}
#[doc = r" Register asset constructors for opaque types that need explicit constructors"]
#[doc = r" This is auto-generated based on discovered constructor methods"]
fn register_asset_constructor_bindings(asset_registry: &bevy_lua_ecs::AssetRegistry) {
    asset_registry.register_asset_constructor("bevy_image::image::Image", |table| {
        let width: u32 = table.get("width").unwrap_or(0);
        let height: u32 = table.get("height").unwrap_or(0);
        let format = {
            use bevy::render::render_resource::TextureFormat;
            let format_str: String = table
                .get("format")
                .unwrap_or_else(|_| "Bgra8UnormSrgb".to_string());
            match format_str.as_str() {
                "Rgba8UnormSrgb" => TextureFormat::Rgba8UnormSrgb,
                "Bgra8UnormSrgb" => TextureFormat::Bgra8UnormSrgb,
                "Rgba8Unorm" => TextureFormat::Rgba8Unorm,
                "Bgra8Unorm" => TextureFormat::Bgra8Unorm,
                "Rgba16Float" => TextureFormat::Rgba16Float,
                "Rgba32Float" => TextureFormat::Rgba32Float,
                "R8Unorm" => TextureFormat::R8Unorm,
                "Rg8Unorm" => TextureFormat::Rg8Unorm,
                _ => TextureFormat::Bgra8UnormSrgb,
            }
        };
        bevy::log::debug!(
            "[AUTO_CONSTRUCTOR] Calling {}::{}",
            stringify!(bevy::image::Image),
            stringify!(new_target_texture)
        );
        Ok(Box::new(bevy::image::Image::new_target_texture(
            width, height, format,
        )) as Box<dyn bevy::reflect::Reflect>)
    });
    bevy::log::debug!(
        "[ASSET_CONSTRUCTOR] Registered auto-discovered asset constructors for opaque types"
    );
}
#[doc = r" Register typed path loaders for all discovered asset types"]
#[doc = r" This is auto-generated to enable load_asset paths to resolve with correct Handle<T> types"]
#[doc = r" Uses the macro which checks ReflectAsset at runtime to filter non-Asset types"]
fn register_typed_path_loaders(
    asset_registry: &bevy_lua_ecs::AssetRegistry,
    type_registry: &bevy::ecs::reflect::AppTypeRegistry,
) {
    bevy_lua_ecs::register_typed_path_loaders!(
        asset_registry.typed_path_loaders,
        type_registry,
        bevy::animation::AnimationClip,
        bevy::audio::AudioSource,
        bevy::audio::Pitch,
        bevy_ecs_tiled::prelude::TiledMapAsset,
        bevy_ecs_tiled::prelude::TiledWorldAsset,
        bevy::gizmos::GizmoAsset,
        bevy::gltf::Gltf,
        bevy::image::Image,
        bevy::image::TextureAtlasLayout,
        bevy::mesh::Mesh,
        bevy::pbr::StandardMaterial,
        bevy::scene::DynamicScene,
        bevy::scene::Scene,
        bevy::shader::Shader,
        bevy::sprite_render::ColorMaterial,
        bevy::text::Font
    );
    bevy::log::debug!("[TYPED_LOADER] Registered typed path loaders for asset types");
}
