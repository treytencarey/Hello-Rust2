---
trigger: always_on
---

When implementing features for this Bevy + Lua project, follow the "Zero Rust" philosophy:

**Core Principle**: All game logic, systems, and behaviors should be implementable purely in Lua. Rust code should only provide generic, reflection-based infrastructure.

**Guidelines**:

1. **Generic Infrastructure Only**: Rust code should use Bevy's reflection system (`TypeRegistry`, `ReflectComponent`, etc.) to provide generic ECS operations, not game-specific logic.

2. **Avoid Game-Specific Rust Code**: Never create Rust systems or components for specific game features (like animations, UI interactions, etc.). Instead, create generic utilities that Lua can use.

3. **Lua-First Design**: When adding features:
   - First, consider how it can be done purely in Lua
   - Only add Rust code if it provides generic ECS capabilities (like `entity:set()`, `world:query()`, `delta_time()`)
   - The Rust addition should be usable for ANY game feature, not just the current one

4. **Reflection-Based Components**: Use Bevy's reflection to automatically register and handle components. Lua should be able to create/modify any reflected Bevy component without custom Rust handlers.

5. **Examples of Good Rust Additions**:
   - [ComponentUpdateQueue](cci:2://file:///d:/Hello-new/bevy-lua-ecs/src/component_update_queue.rs:13:0-15:1) - generic mutation system
   - `world:delta_time()` - generic time access
   - `entity:set(component, data)` - generic component mutation
   - Auto-discovery of components via `TypeRegistry`

6. **Examples of Bad Rust Additions**:
   - `AnimationSystem` - should be in Lua
   - `ButtonClickHandler` - should be in Lua
   - Game-specific component types - use generic Lua components or reflected Bevy components

7. **Test Pattern**: If you can't use the same Rust code for multiple different game features, it's probably too specific and should be in Lua instead.

The goal: A game developer should be able to build entire games in Lua without touching Rust, using only the generic ECS infrastructure.