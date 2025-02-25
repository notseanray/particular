# Particular
 
<p align="center">
  <img src="./particular_5000_bodies.gif">
</p>

[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/canleskis/particular#license)
[![Crates.io](https://img.shields.io/crates/v/particular)](https://crates.io/crates/particular)
[![Docs](https://docs.rs/particular/badge.svg)](https://docs.rs/particular)
 
Particular is a crate providing a simple way to simulate N-body gravitational interaction of particles in Rust.

## Goals
The main goal of this crate is to provide users with a simple API to setup N-body gravitational simulations that can easily be integrated in existing game and physics engines. Thus, it does not include numerical integration or other similar tools and instead only focuses on the acceleration calculations.

Currently, acceleration calculations are computed naively by iterating over all the particles and summing the acceleration caused by all the `massive` particles. In the future, I would like to implement other algorithms such as [Barnes-Hut algorithm](https://en.wikipedia.org/wiki/Barnes%E2%80%93Hut_simulation) or even use compute shaders on the GPU for faster calculations.

Particular can be used with a parallel implementation on the CPU thanks to the [rayon](https://github.com/rayon-rs/rayon) crate. Use the "parallel" feature to enable it, which can lead to huge performance improvements.

# Using Particular

The API to setup a simulation is straightforward:

## Implementing the `Particle` trait
#### Deriving

Used in most cases, when your type has fields named `position` and `mu`.
```rust
#[derive(Particle)]
pub struct Body {
    position: Vec3,
    mu: f32,
//  ...
}
```
#### Manual implementation

Used when your type has more complex fields and cannot directly provide a position and a gravitational parameter.
```rust
struct Body {
    position: Vec3,
    mass: f32,
//  ...
}

impl Particle for Body {
    fn position(&self) -> Vec3 {
        self.position
    }

    fn mu(&self) -> f32 {
        self.mass * G
    }
}
```
## Setting up the simulation
Using your type implementing `Particle`, you will need to create a `ParticleSet` that will contain the particles.

Currently, it stores the particles in two different vectors depending on if the particle has mass or doesn't. This allows optimizations in the case of massless particles (which can represent objects that do not need to affect other objects, like a spaceship).
```rust
let mut particle_set = ParticleSet::new();
// If the type cannot be inferred, use the turbofish syntax:
let mut particle_set = ParticleSet::<Body>::new();

particle_set.add(Body { position, mu });
```
## Computing and using the gravitational acceleration
Finally, using the `result` method of `ParticleSet`, you can iterate over the computed gravitational acceleration of each particle.
```rust
for (particle, acceleration) in particle_set.result() {
    particle.velocity += acceleration * DT;
    particle.position += particle.velocity * DT;
}
```
`particle` here being of the type you used for the `ParticleSet` that implements `Particle`.

## Contribution

PRs are welcome!
