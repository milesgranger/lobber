# Task Plan: Lobber - TUI Artillery Game

## Goal
Build a playable single-player TUI artillery game in Rust with destructible terrain, two ammo types, AI opponent, wind/drag physics, and WoT-style RNG — architected for future multiplayer and economy systems.

## Phases

- [x] Phase 1: Project setup — Cargo.toml deps, module skeleton, basic types
- [x] Phase 2: Terrain system — procedural heightmap generation, destruction/cratering
- [x] Phase 3: Physics engine — projectile simulation (gravity, wind, drag), collision detection
- [x] Phase 4: Game state & logic — turns, ammo types, damage model, RNG, win conditions
- [x] Phase 5: AI opponent — basic targeting with adjustable difficulty
- [x] Phase 6: TUI rendering — terrain, tanks, projectile animation, HUD, input handling
- [x] Phase 7: Game loop & integration — wire everything together into playable game
- [x] Phase 8: Polish — title screen, game over, panic handler, help overlay

## Module Structure

```
src/
  main.rs              # Entry point, terminal setup, game loop
  game/
    mod.rs             # Re-exports
    state.rs           # GameState, GamePhase enum, turn management
    types.rs           # Common types, constants, ammo definitions
    damage.rs          # Damage calculation, splash, critical hits
  physics/
    mod.rs             # Re-exports
    projectile.rs      # Projectile simulation, trajectory stepping
    collision.rs       # Terrain collision detection
  terrain/
    mod.rs             # Re-exports
    generation.rs      # Procedural terrain generation
    heightmap.rs       # Heightmap data structure, crater application
  ai/
    mod.rs             # AI opponent logic
  render/
    mod.rs             # Re-exports
    app.rs             # Top-level App struct, input handling, game loop
    terrain.rs         # Terrain canvas rendering
    hud.rs             # HUD: angle, power, wind, health, ammo selector
    animation.rs       # Projectile flight animation state
```

## Key Design Decisions

- **Game logic has zero TUI dependencies** — testable in isolation
- **Seedable RNG** passed via dependency injection for determinism
- **GamePhase state machine** drives the entire flow
- **Heightmap as Vec<f32>** — simple, efficient, easy to crater
- **SlotMap not needed yet** — only 2 tanks, use direct structs; add slotmap when multiplayer arrives
- **Canvas with Braille markers** for high-resolution terrain rendering
- **Future-proofing**: game state serializable (serde), types designed for >2 players

## Ammo Types

| Type | Base Damage | Splash Radius | Direct Hit Bonus | Drag Coefficient |
|------|------------|---------------|-------------------|-----------------|
| Cannonball | 50 | 0 (2px tolerance) | 2.0x | 0.001 (heavy) |
| Explosive | 25 | 30px | 1.0x | 0.003 (lighter) |

## RNG Parameters (WoT-style)

- Damage spread: base_damage * uniform(0.75, 1.25)
- Accuracy deviation: angle +/- uniform(-1.5, 1.5) degrees, power +/- uniform(-3%, 3%)
- Critical hit: 5% chance, 1.5x multiplier
- Wind: uniform(-5.0, 5.0) m/s, changes each turn

## Status
**All phases complete** — v0.1.0 playable with 29 passing tests
