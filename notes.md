# Lobber - Research Notes

## Tech Stack Decisions

| Concern | Choice | Rationale |
|---------|--------|-----------|
| TUI Framework | `ratatui` 0.30 + `crossterm` 0.29 | Canvas widget with floating-point coords, Braille/HalfBlock markers, origin at bottom-left (natural for physics) |
| Math | `glam` | Lightweight Vec2, SIMD-optimized, no heavy deps |
| RNG | `rand` with seedable `StdRng` | Deterministic replays, testable physics |
| Serialization | `serde` + `serde_json` | Future saves/network, human-readable for debugging |
| Entity storage | `slotmap` | Generational indices, typed arenas, good for dynamic entities |
| Testing | Built-in `#[test]` + `approx` for float comparison | Keep it simple for now |

## Architecture Patterns

### Game Phase State Machine
```
Aiming -> Firing -> Resolving -> TurnTransition -> Aiming (or GameOver)
```
- Input only processed during Aiming
- Firing runs fixed-timestep physics with interpolation for smooth animation
- Resolving applies damage + terrain deformation

### Module Split (one-way dependency: render -> game, never game -> render)
- `game/` — pure logic, no TUI deps, fully testable
- `render/` — TUI rendering, reads game state immutably
- `ai/` — computer opponent logic
- `main.rs` — ties everything together, runs game loop

### Canvas Details
- Canvas coordinates: origin at bottom-left (math-style) — perfect for physics
- Best markers: Braille (2x4 resolution per cell) or HalfBlock (color support)
- Custom shapes via `Shape` trait for terrain profile, explosions, etc.

### Game Loop
- Fixed timestep for physics (~60Hz ticks during Firing phase)
- Non-blocking input polling via `crossterm::event::poll`
- Immediate-mode rendering (full redraw each frame from state)

## Physics Model

### Projectile Flight
- Euler integration with fixed timestep
- Forces: gravity (constant downward), wind (horizontal, changes per turn), drag (proportional to velocity squared)
- Drag coefficient varies by ammo type (cannonball heavier = less wind-affected)

### Damage Model (WoT-style RNG)
- Base damage +/- 25% random spread
- Accuracy deviation: slight random offset from aimed angle/power
- Critical hit: ~5% chance for 1.5x damage multiplier
- Cannonball: high base damage, zero splash radius, must be near-direct hit
- Explosive: lower base damage, splash radius with linear falloff

### Terrain
- Heightmap: Vec<f32> with one height per x-column
- Procedural generation: midpoint displacement (diamond-square variant for 1D)
- Destruction: subtract a crater profile (parabolic) from heightmap on impact
- Terrain types (future): different crater sizes, ricochet behavior
