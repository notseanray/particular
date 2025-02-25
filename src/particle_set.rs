use crate::particle::{Particle, ToPointMass};

use glam::Vec3A;
#[cfg(feature = "parallel")]
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};

#[derive(Default)]
pub struct ParticleSet<P: Particle + Sync> {
    massive: Vec<P>,
    massless: Vec<P>,
}

impl<P: Particle + Sync> ParticleSet<P> {
    pub fn new() -> Self {
        Self {
            massive: Vec::new(),
            massless: Vec::new(),
        }
    }
    pub fn new_with_capacity(massive: usize, massless: usize) -> Self {
        Self {
            massive: Vec::with_capacity(massive),
            massless: Vec::with_capacity(massless),
        }
    }
}

impl<P: Particle + Sync> ParticleSet<P> {
    /// Adds a [`Particle`] to the [`ParticleSet`].
    ///
    /// Particles are stored in two vectors, `massive` or `massless`, depending on if they have mass or not.
    #[inline]
    pub fn add(&mut self, particle: P) {
        if particle.mu() != 0.0 {
            self.massive.push(particle);
        } else {
            self.massless.push(particle);
        }
    }
}

impl<P: Particle + Sync> ParticleSet<P> {
    #[cfg(not(feature = "parallel"))]
    fn get_accelerations(&self) -> Vec<Vec3A> {
        let massive = self.massive.iter().map(P::point_mass).collect::<Vec<_>>();
        let massless = self.massless.iter().map(P::point_mass).collect::<Vec<_>>();

        let accelerations = massive.iter().chain(&massless).map(|particle1| {
            massive.iter().fold(Vec3A::ZERO, |acceleration, particle2| {
                let dir = particle2.0 - particle1.0;
                let mag_2 = dir.length_squared();

                let grav_acc = if mag_2 != 0.0 {
                    particle2.1 * dir / (mag_2 * mag_2.sqrt())
                } else {
                    dir
                };

                acceleration + grav_acc
            })
        });

        accelerations.collect()
    }

    #[cfg(feature = "parallel")]
    fn get_accelerations(&self) -> Vec<Vec3A> {
        let massive = self.massive.iter().map(P::point_mass).collect::<Vec<_>>();
        let massless = self.massless.iter().map(P::point_mass).collect::<Vec<_>>();

        let accelerations = massive.par_iter().chain(&massless).map(|particle1| {
            massive.iter().fold(Vec3A::ZERO, |acceleration, particle2| {
                let dir = particle2.0 - particle1.0;
                let mag_2 = dir.length_squared();

                let grav_acc = if mag_2 != 0.0 {
                    particle2.1 * dir / (mag_2 * mag_2.sqrt())
                } else {
                    dir
                };

                acceleration + grav_acc
            })
        });

        accelerations.collect()
    }

    /// Iterates over the `massive` particles, then the `massless` ones.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &P> {
        self.massive.iter().chain(&self.massless)
    }

    /// Mutably iterates over the `massive` particles, then the `massless` ones.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut P> {
        self.massive.iter_mut().chain(&mut self.massless)
    }

    /// Returns an iterator holding tuples of a mutable reference to a [`Particle`] and its computed gravitational acceleration.
    /// # Example
    /// ```
    /// # use particular::prelude::Particle;
    /// # use particular::ParticleSet;
    /// # use glam::Vec3A;
    /// #
    /// # const DT: f32 = 1.0 / 60.0;
    /// #
    /// # #[derive(Particle)]
    /// # pub struct Body {
    /// #     position: Vec3A,
    /// #     velocity: Vec3A,
    /// #     mu: f32,
    /// # }
    /// # let mut particle_set = ParticleSet::<Body>::new();
    /// for (particle, acceleration) in particle_set.result() {
    ///     particle.velocity += acceleration * DT;
    ///     particle.position += particle.velocity * DT;
    /// }
    /// ```
    #[inline]
    pub fn result(&mut self) -> impl Iterator<Item = (&mut P, Vec3A)> {
        let accelerations = self.get_accelerations();
        let particles = self.iter_mut();
        particles.zip(accelerations)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{Particle, ParticleSet, PointMass, ToPointMass};
    use glam::Vec3A;
    use particular_derive::Particle;

    #[derive(Particle)]
    struct Body {
        position: Vec3A,
        mu: f32,
    }

    fn with_two_particles(p1: PointMass, p2: PointMass) -> ParticleSet<Body> {
        let mut particle_set = ParticleSet::new();

        particle_set.add(Body {
            position: p1.0,
            mu: p1.1,
        });

        particle_set.add(Body {
            position: p2.0,
            mu: p2.1,
        });

        particle_set
    }

    #[test]
    fn add_particles() {
        let p1 = (Vec3A::ONE, 0.0);
        let p2 = (Vec3A::NEG_ONE, 8.0);

        let particle_set = with_two_particles(p1, p2);
        let mut iter = particle_set.iter();

        assert_eq!(p2, iter.next().unwrap().point_mass());
        assert_eq!(p1, iter.next().unwrap().point_mass());
    }

    const EPSILON: f32 = 1E-6;

    #[test]
    fn acceleration_calculation() {
        let p1 = (Vec3A::ZERO, 0.0);
        let p2 = (Vec3A::splat(1.0), 3.0);

        let dir = p2.0 - p1.0;
        let mag_2 = dir.length_squared();
        let grav_acc = dir / (mag_2 * mag_2.sqrt());

        for (particle, acceleration) in with_two_particles(p1, p2).result() {
            if particle.point_mass() == p1 {
                assert!(acceleration.distance_squared(grav_acc * p2.1) < EPSILON);
            } else {
                assert!(acceleration.distance_squared(-grav_acc * p1.1) < EPSILON);
            }
        }
    }
}
