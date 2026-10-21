# ADR 0003: Headless Engine and ECS Decoupling

## Status
Accepted

## Context
In production simulations, game loops frequently conflate rendering with game logic and spatial validation. When simulation logic is tightly coupled to a graphics device or window event loop, running headless server instances, automated continuous integration tests, or distributed validation clusters becomes fragile or impossible.

## Decision
1. **Headless First**: All geospatial transformation, ingestion, validation, world compilation, pathfinding, and physics simulation must execute headlessly without requiring a windowing display server or GPU device context.
2. **Bevy ECS as Decoupled Simulation Core**: Simulation state (entities, transforms, time-of-day, weather, traffic) is managed inside a `bevy_ecs::World`. The simulation systems update at a fixed 60Hz tick rate independently of the rendering frame rate.
3. **Rendering as a Pure Consumer**: The `realearth-rendering` crate and viewer application consume read-only snapshots or queries from the simulation and streaming caches. Rendering never mutates simulation state.

## Consequences
- 100% of the world compilation, validation, streaming, and simulation pipelines can be validated in headless GitHub Actions runners.
- Physics and simulation can run at fixed time steps (60Hz) while rendering runs uncapped or vsynced.
