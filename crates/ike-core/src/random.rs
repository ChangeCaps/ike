use rand::{
    distributions::{
        uniform::{SampleRange, SampleUniform},
        Standard,
    },
    prelude::Distribution,
    SeedableRng,
};

/// A non-cryptographic random number generator.
#[derive(Clone)]
pub struct Rng {
    rng: rand::rngs::SmallRng,
}

impl Rng {
    #[inline]
    pub fn new() -> Self {
        Self {
            rng: rand::rngs::SmallRng::from_entropy(),
        }
    }

    #[inline]
    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng: rand::rngs::SmallRng::seed_from_u64(seed),
        }
    }

    #[inline]
    pub fn random<T>(&mut self) -> T
    where
        Standard: Distribution<T>,
    {
        rand::Rng::gen(&mut self.rng)
    }

    #[inline]
    pub fn random_range<T, R>(&mut self, range: R) -> T
    where
        T: SampleUniform,
        R: SampleRange<T>,
    {
        rand::Rng::gen_range(&mut self.rng, range)
    }
}

pub fn random<T>() -> T
where
    Standard: Distribution<T>,
{
    Rng::new().random()
}

pub fn random_range<T, R>(range: R) -> T
where
    T: SampleUniform,
    R: SampleRange<T>,
{
    Rng::new().random_range(range)
}
