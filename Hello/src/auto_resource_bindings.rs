pub fn register_auto_resource_bindings(registry: &bevy_lua_ecs::LuaResourceRegistry) {}
#[doc = r" Auto-discovered entity wrapper type names (for runtime TypeRegistry lookup)"]
#[doc = r" These are type names discovered by scanning bevy_* crates for:"]
#[doc = r" `pub struct TypeName(pub Entity)` with `#[derive(Component)]`"]
pub const DISCOVERED_ENTITY_WRAPPERS: &[&str] = &[
    "ChildOf",
    "Children",
    "TiledColliderOf",
    "TiledColliders",
    "TiledMapReference",
    "TiledObjectVisualOf",
    "TiledObjectVisuals",
    "BindingOf",
    "Bindings",
    "XrHandBoneEntities",
    "LightViewEntities",
    "RenderEntity",
    "MainEntity",
    "OcclusionCullingSubviewEntities",
    "ParentSync",
    "ClientEntityMap",
    "PriorityMap",
    "TargetCamera",
    "UiTargetCamera",
    "UiCameraView",
    "UiViewTarget",
];
#[doc = r" Register entity wrapper components at runtime using TypeRegistry"]
#[doc = r" This looks up each discovered type name in the registry and registers"]
#[doc = r" a handler if it's a valid entity wrapper component"]
pub fn register_entity_wrappers_from_registry(
    component_registry: &mut bevy_lua_ecs::ComponentRegistry,
    type_registry: &bevy::ecs::reflect::AppTypeRegistry,
) {
    bevy_lua_ecs::register_entity_wrappers_runtime(
        component_registry,
        type_registry,
        DISCOVERED_ENTITY_WRAPPERS,
    );
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
    "AnimationClip",
    "AnimationGraph",
    "LoadedUntypedAsset",
    "LoadedFolder",
    "AudioSource",
    "Pitch",
    "AutoExposureCompensationCurve",
    "TiledMapAsset",
    "TiledWorldAsset",
    "TiledMap",
    "TiledWorld",
    "StandardTilemapMaterial",
    "LineGizmo",
    "GizmoAsset",
    "Gltf",
    "GltfNode",
    "GltfMesh",
    "GltfPrimitive",
    "GltfSkin",
    "Image",
    "TextureAtlasLayout",
    "Mesh",
    "SkinnedMeshInverseBindposes",
    "ExtendedMaterial",
    "StandardMaterial",
    "WireframeMaterial",
    "ForwardDecalMaterialExt",
    "MeshletMesh",
    "Shader",
    "ShaderStorageBuffer",
    "DynamicScene",
    "Scene",
    "ColorMaterial",
    "TextureAtlas",
    "Wireframe2dMaterial",
    "TilemapChunkMaterial",
    "Font",
    "FontAtlasSet",
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
#[doc = r" Auto-discovered Handle<T> newtype wrappers"]
#[doc = r" Format: (newtype_name, inner_asset_name) - runtime will resolve via TypeRegistry"]
#[doc = r#" Examples: ("ImageRenderTarget", "Image"), ("Mesh3d", "Mesh")"#]
pub const DISCOVERED_NEWTYPE_WRAPPERS: &[(&str, &str)] = &[
    ("LoadedFolder", "UntypedHandle"),
    ("Skybox", "Image"),
    ("TiledMap", "TiledMapAsset"),
    ("TiledWorld", "TiledWorldAsset"),
    ("TiledMapHandle", "TiledMap"),
    ("TiledWorldHandle", "TiledWorld"),
    ("SpotLightTexture", "Image"),
    ("Mesh2d", "Mesh"),
    ("Mesh3d", "Mesh"),
    ("MeshletMesh3d", "MeshletMesh"),
    ("Bluenoise", "Image"),
    ("SimplifiedMesh", "Mesh"),
    (
        "ManualTextureViews",
        "ManualTextureViewHandle , ManualTextureView",
    ),
    ("SceneRoot", "Scene"),
    ("DynamicSceneRoot", "DynamicScene"),
    ("Mesh2dHandle", "Mesh"),
    ("WinitActionHandlers", "Entity , WinitActionHandler"),
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
    ("Commands", "bevy::ecs::Commands"),
    ("ParallelCommands", "bevy::ecs::ParallelCommands"),
    ("EventMutator", "bevy::ecs::EventMutator"),
    ("MessageMutator", "bevy::ecs::MessageMutator"),
    ("MessageReader", "bevy::ecs::MessageReader"),
    ("MessageWriter", "bevy::ecs::MessageWriter"),
    (
        "TiledMapEventWriters",
        "bevy::ecs_tiled::TiledMapEventWriters",
    ),
    ("ContextTime", "bevy::enhanced_input::ContextTime"),
    (
        "DirectionalNavigation",
        "bevy::input_focus::DirectionalNavigation",
    ),
    ("IsFocusedHelper", "bevy::input_focus::IsFocusedHelper"),
    ("TabNavigation", "bevy::input_focus::TabNavigation"),
    ("PickingEventWriters", "bevy::picking::PickingEventWriters"),
    ("MeshRayCast", "bevy::picking::MeshRayCast"),
    (
        "PickingMessageWriters",
        "bevy::picking::PickingMessageWriters",
    ),
    ("FallbackImageMsaa", "bevy::render::FallbackImageMsaa"),
    ("TextReader", "bevy::text::TextReader"),
    ("TextWriter", "bevy::text::TextWriter"),
    ("TransformHelper", "bevy::transform::TransformHelper"),
    ("UiRootNodes", "bevy::ui::UiRootNodes"),
    ("UiChildren", "bevy::ui::UiChildren"),
    (
        "UiLayoutSystemRemovedComponentParam",
        "bevy::ui::UiLayoutSystemRemovedComponentParam",
    ),
    ("DefaultUiCamera", "bevy::ui::DefaultUiCamera"),
    ("UiCameraMap", "bevy::ui_render::UiCameraMap"),
];
#[doc = r" Auto-discovered SystemParam methods that use Reflect-compatible parameters"]
#[doc = r" Format: (param_type, method_name, return_type, returns_iterator)"]
pub const DISCOVERED_SYSTEMPARAM_METHODS: &[(&str, &str, &str, bool)] = &[
    ("ComponentIdFor", "get", "ComponentId", false),
    ("EventReader", "read", "EventIterator<'_,E>", false),
    ("EventReader", "iter", "EventIterator<'_,E>", false),
    (
        "EventReader",
        "read_with_id",
        "EventIteratorWithId<'_,E>",
        false,
    ),
    (
        "EventReader",
        "iter_with_id",
        "EventIteratorWithId<'_,E>",
        false,
    ),
    ("EventReader", "len", "usize", false),
    ("EventReader", "is_empty", "bool", false),
    ("EventReader", "clear", "()", false),
    ("EventWriter", "send_default", "()", false),
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
    ("RemovedComponents", "iter", "RemovedIter<'_>", true),
    (
        "RemovedComponents",
        "read_with_id",
        "RemovedIterWithId<'_>",
        false,
    ),
    (
        "RemovedComponents",
        "iter_with_id",
        "RemovedIterWithId<'_>",
        false,
    ),
    ("RemovedComponents", "len", "usize", false),
    ("RemovedComponents", "is_empty", "bool", false),
    ("RemovedComponents", "clear", "()", false),
    ("Commands", "spawn_empty", "EntityCommands<'w,'s,'a>", false),
    ("Commands", "init_resource", "()", false),
    ("Commands", "remove_resource", "()", false),
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
    ("Commands", "reborrow", "Commands<'w,'_>", false),
    ("Commands", "spawn_empty", "EntityCommands", false),
    ("Commands", "init_resource", "()", false),
    ("Commands", "remove_resource", "()", false),
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
    ("Commands", "reborrow", "Commands<'w,'_>", false),
    ("Commands", "spawn_empty", "EntityCommands", false),
    ("Commands", "init_resource", "()", false),
    ("Commands", "remove_resource", "()", false),
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
    ("Commands", "reborrow", "Commands<'w,'_>", false),
    ("Commands", "spawn_empty", "EntityCommands", false),
    ("Commands", "init_resource", "()", false),
    ("Commands", "remove_resource", "()", false),
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
    ("Commands", "reborrow", "Commands<'w,'_>", false),
    ("Commands", "spawn_empty", "EntityCommands<'_>", false),
    ("Commands", "init_resource", "()", false),
    ("Commands", "remove_resource", "()", false),
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
    ("Commands", "reborrow", "Commands<'w,'_>", false),
    ("Commands", "spawn_empty", "EntityCommands<'_>", false),
    ("Commands", "init_resource", "()", false),
    ("Commands", "remove_resource", "()", false),
    ("ContextTime", "delta_kind", "Duration", false),
    ("ContextTime", "delta_kind", "Duration", false),
    ("ContextTime", "delta_kind", "Duration", false),
    ("UiRootNodes", "iter", "implIterator<Item=Entity>+'s", false),
    ("DefaultUiCamera", "get", "Option<Entity>", false),
    ("UiRootNodes", "iter", "implIterator<Item=Entity>+'s", false),
    ("UiCameraMap", "get_mapper", "UiCameraMapper<'w,'s>", false),
    ("DefaultUiCamera", "get", "Option<Entity>", false),
    ("UiRootNodes", "iter", "implIterator<Item=Entity>+'s", false),
    ("DefaultUiCamera", "get", "Option<Entity>", false),
    ("UiRootNodes", "iter", "implIterator<Item=Entity>+'s", false),
    ("DefaultUiCamera", "get", "Option<Entity>", false),
    ("UiCameraMap", "get_mapper", "UiCameraMapper<'w,'s>", false),
    ("UiCameraMap", "get_mapper", "UiCameraMapper<'w,'s>", false),
    ("ComponentIdFor", "get", "ComponentId", false),
    ("EventReader", "read", "EventIterator<'_,E>", false),
    ("EventReader", "iter", "EventIterator<'_,E>", false),
    (
        "EventReader",
        "read_with_id",
        "EventIteratorWithId<'_,E>",
        false,
    ),
    (
        "EventReader",
        "iter_with_id",
        "EventIteratorWithId<'_,E>",
        false,
    ),
    ("EventReader", "len", "usize", false),
    ("EventReader", "is_empty", "bool", false),
    ("EventReader", "clear", "()", false),
    ("EventWriter", "send_default", "()", false),
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
    ("RemovedComponents", "iter", "RemovedIter<'_>", true),
    (
        "RemovedComponents",
        "read_with_id",
        "RemovedIterWithId<'_>",
        false,
    ),
    (
        "RemovedComponents",
        "iter_with_id",
        "RemovedIterWithId<'_>",
        false,
    ),
    ("RemovedComponents", "len", "usize", false),
    ("RemovedComponents", "is_empty", "bool", false),
    ("RemovedComponents", "clear", "()", false),
    ("Commands", "spawn_empty", "EntityCommands<'w,'s,'a>", false),
    ("Commands", "init_resource", "()", false),
    ("Commands", "remove_resource", "()", false),
    ("ComponentIdFor", "get", "ComponentId", false),
    ("EventReader", "read", "EventIterator<'_,E>", false),
    (
        "EventReader",
        "read_with_id",
        "EventIteratorWithId<'_,E>",
        false,
    ),
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
    ("Commands", "reborrow", "Commands<'w,'_>", false),
    ("Commands", "spawn_empty", "EntityCommands", false),
    ("Commands", "init_resource", "()", false),
    ("Commands", "remove_resource", "()", false),
    ("DefaultUiCamera", "get", "Option<Entity>", false),
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
            let mut state = bevy::ecs::system::SystemState::<
                bevy::picking::mesh_picking::ray_cast::MeshRayCast,
            >::new(world);
            let mut param = state.get_mut(world);
            let result = param.cast_ray(typed_arg0, &Default::default());
            Ok(mlua::Value::String(
                lua.create_string(&format!("{:?}", result))?,
            ))
        }
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
            let mut state = bevy::ecs::system::SystemState::<
                bevy::picking::mesh_picking::ray_cast::MeshRayCast,
            >::new(world);
            let mut param = state.get_mut(world);
            let result = param.cast_ray(typed_arg0, &Default::default());
            Ok(mlua::Value::String(
                lua.create_string(&format!("{:?}", result))?,
            ))
        }
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
            let mut state = bevy::ecs::system::SystemState::<
                bevy::picking::mesh_picking::ray_cast::MeshRayCast,
            >::new(world);
            let mut param = state.get_mut(world);
            let result = param.cast_ray(typed_arg0, &Default::default());
            Ok(mlua::Value::String(
                lua.create_string(&format!("{:?}", result))?,
            ))
        }
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
            let mut state = bevy::ecs::system::SystemState::<
                bevy::picking::mesh_picking::ray_cast::MeshRayCast,
            >::new(world);
            let mut param = state.get_mut(world);
            let result = param.cast_ray(typed_arg0, &Default::default());
            Ok(mlua::Value::String(
                lua.create_string(&format!("{:?}", result))?,
            ))
        }
        _ => Err(mlua::Error::RuntimeError(format!(
            "Unknown or unsupported SystemParam method: {}::{}",
            param_name, method_name
        ))),
    }
}
#[doc = r" Dispatch read_events call for a specific event type"]
#[doc = r" Returns a Lua table of events converted via reflection"]
pub fn dispatch_read_events(
    lua: &mlua::Lua,
    world: &mut bevy::prelude::World,
    event_type: &str,
) -> mlua::Result<mlua::Value> {
    let type_registry = world
        .resource::<bevy::ecs::reflect::AppTypeRegistry>()
        .clone();
    match event_type { "WindowResized" | "bevy_window::event::WindowResized" | "bevy::window::WindowResized" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: WindowResized >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "RequestRedraw" | "bevy_window::event::RequestRedraw" | "bevy::window::RequestRedraw" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: RequestRedraw >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "WindowCreated" | "bevy_window::event::WindowCreated" | "bevy::window::WindowCreated" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: WindowCreated >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "WindowCloseRequested" | "bevy_window::event::WindowCloseRequested" | "bevy::window::WindowCloseRequested" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: WindowCloseRequested >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "WindowClosed" | "bevy_window::event::WindowClosed" | "bevy::window::WindowClosed" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: WindowClosed >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "WindowDestroyed" | "bevy_window::event::WindowDestroyed" | "bevy::window::WindowDestroyed" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: WindowDestroyed >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "CursorMoved" | "bevy_window::event::CursorMoved" | "bevy::window::CursorMoved" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: CursorMoved >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "CursorEntered" | "bevy_window::event::CursorEntered" | "bevy::window::CursorEntered" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: CursorEntered >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "CursorLeft" | "bevy_window::event::CursorLeft" | "bevy::window::CursorLeft" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: CursorLeft >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "WindowFocused" | "bevy_window::event::WindowFocused" | "bevy::window::WindowFocused" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: WindowFocused >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "WindowScaleFactorChanged" | "bevy_window::event::WindowScaleFactorChanged" | "bevy::window::WindowScaleFactorChanged" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: WindowScaleFactorChanged >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "WindowBackendScaleFactorChanged" | "bevy_window::event::WindowBackendScaleFactorChanged" | "bevy::window::WindowBackendScaleFactorChanged" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: WindowBackendScaleFactorChanged >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "WindowMoved" | "bevy_window::event::WindowMoved" | "bevy::window::WindowMoved" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: WindowMoved >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "WindowThemeChanged" | "bevy_window::event::WindowThemeChanged" | "bevy::window::WindowThemeChanged" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: window :: WindowThemeChanged >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "KeyboardInput" | "bevy_input::keyboard::KeyboardInput" | "bevy::input::keyboard::KeyboardInput" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: input :: keyboard :: KeyboardInput >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "MouseButtonInput" | "bevy_input::mouse::MouseButtonInput" | "bevy::input::mouse::MouseButtonInput" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: input :: mouse :: MouseButtonInput >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "MouseMotion" | "bevy_input::mouse::MouseMotion" | "bevy::input::mouse::MouseMotion" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: input :: mouse :: MouseMotion >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "MouseWheel" | "bevy_input::mouse::MouseWheel" | "bevy::input::mouse::MouseWheel" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: input :: mouse :: MouseWheel >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } , "PointerInput" | "bevy_picking::pointer::PointerInput" | "bevy::picking::pointer::PointerInput" => { let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventReader < bevy :: picking :: pointer :: PointerInput >> :: new (world) ; let mut event_reader = system_state . get_mut (world) ; let results = lua . create_table () ? ; let mut index = 1 ; for event in event_reader . read () { if let Ok (event_value) = bevy_lua_ecs :: reflection_to_lua (lua , event as & dyn bevy :: reflect :: PartialReflect , & type_registry) { results . set (index , event_value) ? ; index += 1 ; } } Ok (mlua :: Value :: Table (results)) } _ => Err (mlua :: Error :: RuntimeError (format ! ("Unknown event type: '{}'. Available events are discovered from bevy_window and bevy_input." , event_type))) }
}
#[doc = r" Dispatch write_events call for a specific event type"]
#[doc = r" Constructs the event from a Lua table using reflection and sends via EventWriter"]
pub fn dispatch_write_events(
    lua: &mlua::Lua,
    world: &mut bevy::prelude::World,
    event_type: &str,
    data: &mlua::Table,
) -> Result<(), String> {
    match event_type { "WindowResized" | "bevy_window::event::WindowResized" | "bevy::window::WindowResized" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowResized") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowResized" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowResized as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowResized >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowResized") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowResized")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowResized")) ; } } , "RequestRedraw" | "bevy_window::event::RequestRedraw" | "bevy::window::RequestRedraw" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::RequestRedraw") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::RequestRedraw" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: RequestRedraw as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: RequestRedraw >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::RequestRedraw") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::RequestRedraw")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::RequestRedraw")) ; } } , "WindowCreated" | "bevy_window::event::WindowCreated" | "bevy::window::WindowCreated" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowCreated") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowCreated" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowCreated as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowCreated >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowCreated") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowCreated")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowCreated")) ; } } , "WindowCloseRequested" | "bevy_window::event::WindowCloseRequested" | "bevy::window::WindowCloseRequested" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowCloseRequested") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowCloseRequested" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowCloseRequested as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowCloseRequested >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowCloseRequested") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowCloseRequested")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowCloseRequested")) ; } } , "WindowClosed" | "bevy_window::event::WindowClosed" | "bevy::window::WindowClosed" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowClosed") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowClosed" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowClosed as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowClosed >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowClosed") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowClosed")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowClosed")) ; } } , "WindowDestroyed" | "bevy_window::event::WindowDestroyed" | "bevy::window::WindowDestroyed" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowDestroyed") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowDestroyed" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowDestroyed as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowDestroyed >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowDestroyed") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowDestroyed")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowDestroyed")) ; } } , "CursorMoved" | "bevy_window::event::CursorMoved" | "bevy::window::CursorMoved" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::CursorMoved") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::CursorMoved" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: CursorMoved as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: CursorMoved >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::CursorMoved") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::CursorMoved")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::CursorMoved")) ; } } , "CursorEntered" | "bevy_window::event::CursorEntered" | "bevy::window::CursorEntered" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::CursorEntered") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::CursorEntered" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: CursorEntered as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: CursorEntered >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::CursorEntered") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::CursorEntered")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::CursorEntered")) ; } } , "CursorLeft" | "bevy_window::event::CursorLeft" | "bevy::window::CursorLeft" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::CursorLeft") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::CursorLeft" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: CursorLeft as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: CursorLeft >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::CursorLeft") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::CursorLeft")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::CursorLeft")) ; } } , "WindowFocused" | "bevy_window::event::WindowFocused" | "bevy::window::WindowFocused" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowFocused") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowFocused" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowFocused as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowFocused >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowFocused") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowFocused")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowFocused")) ; } } , "WindowScaleFactorChanged" | "bevy_window::event::WindowScaleFactorChanged" | "bevy::window::WindowScaleFactorChanged" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowScaleFactorChanged") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowScaleFactorChanged" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowScaleFactorChanged as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowScaleFactorChanged >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowScaleFactorChanged") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowScaleFactorChanged")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowScaleFactorChanged")) ; } } , "WindowBackendScaleFactorChanged" | "bevy_window::event::WindowBackendScaleFactorChanged" | "bevy::window::WindowBackendScaleFactorChanged" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowBackendScaleFactorChanged") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowBackendScaleFactorChanged" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowBackendScaleFactorChanged as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowBackendScaleFactorChanged >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowBackendScaleFactorChanged") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowBackendScaleFactorChanged")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowBackendScaleFactorChanged")) ; } } , "WindowMoved" | "bevy_window::event::WindowMoved" | "bevy::window::WindowMoved" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowMoved") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowMoved" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowMoved as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowMoved >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowMoved") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowMoved")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowMoved")) ; } } , "WindowThemeChanged" | "bevy_window::event::WindowThemeChanged" | "bevy::window::WindowThemeChanged" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::window::WindowThemeChanged") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::window::WindowThemeChanged" , e)) ? ; if let Some (concrete_event) = < bevy :: window :: WindowThemeChanged as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: window :: WindowThemeChanged >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::window::WindowThemeChanged") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::window::WindowThemeChanged")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::window::WindowThemeChanged")) ; } } , "KeyboardInput" | "bevy_input::keyboard::KeyboardInput" | "bevy::input::keyboard::KeyboardInput" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::input::keyboard::KeyboardInput") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::input::keyboard::KeyboardInput" , e)) ? ; if let Some (concrete_event) = < bevy :: input :: keyboard :: KeyboardInput as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: input :: keyboard :: KeyboardInput >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::input::keyboard::KeyboardInput") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::input::keyboard::KeyboardInput")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::input::keyboard::KeyboardInput")) ; } } , "MouseButtonInput" | "bevy_input::mouse::MouseButtonInput" | "bevy::input::mouse::MouseButtonInput" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::input::mouse::MouseButtonInput") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::input::mouse::MouseButtonInput" , e)) ? ; if let Some (concrete_event) = < bevy :: input :: mouse :: MouseButtonInput as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: input :: mouse :: MouseButtonInput >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::input::mouse::MouseButtonInput") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::input::mouse::MouseButtonInput")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::input::mouse::MouseButtonInput")) ; } } , "MouseMotion" | "bevy_input::mouse::MouseMotion" | "bevy::input::mouse::MouseMotion" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::input::mouse::MouseMotion") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::input::mouse::MouseMotion" , e)) ? ; if let Some (concrete_event) = < bevy :: input :: mouse :: MouseMotion as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: input :: mouse :: MouseMotion >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::input::mouse::MouseMotion") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::input::mouse::MouseMotion")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::input::mouse::MouseMotion")) ; } } , "MouseWheel" | "bevy_input::mouse::MouseWheel" | "bevy::input::mouse::MouseWheel" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::input::mouse::MouseWheel") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::input::mouse::MouseWheel" , e)) ? ; if let Some (concrete_event) = < bevy :: input :: mouse :: MouseWheel as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: input :: mouse :: MouseWheel >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::input::mouse::MouseWheel") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::input::mouse::MouseWheel")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::input::mouse::MouseWheel")) ; } } , "PointerInput" | "bevy_picking::pointer::PointerInput" | "bevy::picking::pointer::PointerInput" => { let type_registry = world . resource :: < bevy :: ecs :: reflect :: AppTypeRegistry > () . clone () ; let registry = type_registry . read () ; if let Some (type_registration) = registry . get_with_type_path ("bevy::picking::pointer::PointerInput") { let type_info = type_registration . type_info () ; let dynamic = bevy_lua_ecs :: lua_table_to_dynamic (lua , data , type_info , & type_registry) . map_err (| e | format ! ("Failed to build event '{}': {}" , "bevy::picking::pointer::PointerInput" , e)) ? ; if let Some (concrete_event) = < bevy :: picking :: pointer :: PointerInput as bevy :: reflect :: FromReflect > :: from_reflect (& dynamic) { drop (registry) ; let mut system_state = bevy :: ecs :: system :: SystemState :: < bevy :: prelude :: EventWriter < bevy :: picking :: pointer :: PointerInput >> :: new (world) ; let mut event_writer = system_state . get_mut (world) ; event_writer . write (concrete_event) ; bevy :: log :: debug ! ("[EVENT_WRITE] Sent event: {}" , "bevy::picking::pointer::PointerInput") ; return Ok (()) ; } return Err (format ! ("Failed to construct event '{}' via FromReflect" , "bevy::picking::pointer::PointerInput")) ; } else { return Err (format ! ("Event type '{}' not found in TypeRegistry" , "bevy::picking::pointer::PointerInput")) ; } } _ => Err (format ! ("Unknown event type: '{}'. Available events are discovered from bevy_window and bevy_input." , event_type)) }
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
#[doc = r" Dispatch a Lua observer callback for an entity"]
fn dispatch_lua_observer(
    lua_ctx: &bevy_lua_ecs::LuaScriptContext,
    observer_registry: &bevy_lua_ecs::LuaObserverRegistry,
    update_queue: &bevy_lua_ecs::ComponentUpdateQueue,
    entity: bevy::prelude::Entity,
    event_type: &str,
    position: Option<bevy::math::Vec2>,
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
                    let event_table = lua_ctx.lua.create_table().unwrap();
                    if let Some(pos) = position {
                        let _ = event_table.set("x", pos.x);
                        let _ = event_table.set("y", pos.y);
                    }
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
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::Cancel>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<Cancel>",
        None,
    );
}
fn on_pointer_over_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::Over>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<Over>",
        None,
    );
}
fn on_pointer_out_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::Out>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<Out>",
        None,
    );
}
fn on_pointer_down_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::Press>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<Down>",
        None,
    );
}
fn on_pointer_up_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::Release>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<Up>",
        None,
    );
}
fn on_pointer_click_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::Click>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<Click>",
        None,
    );
}
fn on_pointer_move_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::Move>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let pos = event.event().pointer_location.position;
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<Move>",
        Some(pos),
    );
}
fn on_pointer_dragstart_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::DragStart>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let pos = event.event().pointer_location.position;
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<DragStart>",
        Some(pos),
    );
}
fn on_pointer_drag_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::Drag>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let pos = event.event().pointer_location.position;
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<Drag>",
        Some(pos),
    );
}
fn on_pointer_dragend_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::DragEnd>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let pos = event.event().pointer_location.position;
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<DragEnd>",
        Some(pos),
    );
}
fn on_pointer_dragenter_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::DragEnter>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let pos = event.event().pointer_location.position;
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<DragEnter>",
        Some(pos),
    );
}
fn on_pointer_dragover_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::DragOver>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let pos = event.event().pointer_location.position;
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<DragOver>",
        Some(pos),
    );
}
fn on_pointer_dragleave_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::DragLeave>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let pos = event.event().pointer_location.position;
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<DragLeave>",
        Some(pos),
    );
}
fn on_pointer_dragdrop_lua(
    event: bevy::prelude::On<bevy::prelude::Pointer<bevy::prelude::DragDrop>>,
    lua_ctx: bevy::prelude::Res<bevy_lua_ecs::LuaScriptContext>,
    observer_registry: bevy::prelude::Res<bevy_lua_ecs::LuaObserverRegistry>,
    update_queue: bevy::prelude::Res<bevy_lua_ecs::ComponentUpdateQueue>,
) {
    let pos = event.event().pointer_location.position;
    dispatch_lua_observer(
        &lua_ctx,
        &observer_registry,
        &update_queue,
        event.event().entity,
        "Pointer<DragDrop>",
        Some(pos),
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
        "Pointer<Down>" => {
            commands.entity(entity).observe(on_pointer_down_lua);
        }
        "Pointer<Up>" => {
            commands.entity(entity).observe(on_pointer_up_lua);
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
        "bevy::input::keyboard::KeyboardInput"
    );
    app.add_event::<bevy::input::keyboard::KeyboardInput>();
    app.register_type::<bevy::input::keyboard::KeyboardInput>();
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
    bevy::log::debug!(
        "[REGISTER_EVENTS] Adding event type: {}",
        "bevy::picking::pointer::PointerInput"
    );
    app.add_event::<bevy::picking::pointer::PointerInput>();
    app.register_type::<bevy::picking::pointer::PointerInput>();
    app.register_type::<bevy::picking::pointer::PointerInput>();
    bevy::log::debug!(
        "[REGISTER_MESSAGES] Adding message type: {}",
        "bevy::picking::pointer::PointerInput"
    );
    bevy::log::debug!("Auto-discovered Bevy Events and Messages registered for Lua");
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
    bevy_lua_ecs::register_cloner_if_clone::<bevy::gizmos::GizmoAsset>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::gltf::GltfNode>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::gltf::GltfMesh>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::gltf::GltfPrimitive>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::gltf::GltfSkin>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::prelude::Image>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::prelude::Mesh>(&mut cloners);
    bevy_lua_ecs::register_cloner_if_clone::<bevy::prelude::StandardMaterial>(&mut cloners);
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
            stringify!(bevy::prelude::Image),
            stringify!(new_target_texture)
        );
        Ok(Box::new(bevy::prelude::Image::new_target_texture(
            width, height, format,
        )) as Box<dyn bevy::reflect::Reflect>)
    });
    bevy::log::debug!(
        "[ASSET_CONSTRUCTOR] Registered auto-discovered asset constructors for opaque types"
    );
}
