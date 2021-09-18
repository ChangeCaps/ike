use std::collections::BTreeMap;

use glam::{Quat, Vec3};

#[derive(Clone, Debug)]
pub enum AnimationProperty {
    Translation,
    Rotation,
    Scale,
    MorphTargetWeights,
}

#[derive(Clone, Debug)]
pub struct ChannelTarget {
    pub node: usize,
    pub property: AnimationProperty,
}

#[derive(Clone, Debug)]
pub struct AnimationChannel {
    pub target: ChannelTarget,
    pub sampler: usize,
}

#[derive(Clone, Debug)]
pub enum SampleOutput {
    Translation(Vec3),
    Rotation(Quat),
    Scale(Vec3),
    MorphTargetWeight(f32),
}

impl SampleOutput {
    #[inline]
    pub fn unwrap_translation(&self) -> Vec3 {
        if let SampleOutput::Translation(p) = self {
            *p
        } else {
            panic!()
        }
    }

    #[inline]
    pub fn unwrap_rotation(&self) -> Quat {
        if let SampleOutput::Rotation(p) = self {
            *p
        } else {
            panic!()
        }
    }

    #[inline]
    pub fn unwrap_scale(&self) -> Vec3 {
        if let SampleOutput::Scale(p) = self {
            *p
        } else {
            panic!()
        }
    }

    #[inline]
    pub fn unwrap_weight(&self) -> f32 {
        if let SampleOutput::MorphTargetWeight(p) = self {
            *p
        } else {
            panic!()
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct SampleInput(pub f32);

impl Eq for SampleInput {}

impl Ord for SampleInput {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct AnimationSample {
    pub input: f32,
    pub output: SampleOutput,
}

#[derive(Clone, Debug)]
pub enum Interpolation {
    Linear,
    Step,
    CubicSpline,
}

impl Interpolation {
    #[inline]
    pub fn interpolate(&self, s: f32, a: &SampleOutput, b: &SampleOutput) -> SampleOutput {
        match self {
            Interpolation::Linear | Interpolation::CubicSpline => match a {
                SampleOutput::Translation(a) => {
                    let b = b.unwrap_translation();

                    SampleOutput::Translation(a.lerp(b, s))
                }
                SampleOutput::Rotation(a) => {
                    let b = b.unwrap_rotation();

                    SampleOutput::Rotation(a.slerp(b, s))
                }
                SampleOutput::Scale(a) => {
                    let b = b.unwrap_scale();

                    SampleOutput::Scale(a.lerp(b, s))
                }
                SampleOutput::MorphTargetWeight(a) => {
                    let b = b.unwrap_weight();

                    SampleOutput::MorphTargetWeight(a + s * (b - a))
                }
            },
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone)]
pub struct AnimationSampler {
    pub samples: BTreeMap<SampleInput, AnimationSample>,
    pub interpolation: Interpolation,
}

impl std::fmt::Debug for AnimationSampler {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationSampler")
            .field("interpolation", &self.interpolation)
            .field("samples", &self.samples)
            .finish()
    }
}

impl AnimationSampler {
    #[inline]
    pub fn sample(&self, t: f32) -> Option<SampleOutput> {
        let (_, less) = self.samples.range(..SampleInput(t)).last()?;
        let (_, more) = self.samples.range(SampleInput(t)..).next()?;

        let s = (t - less.input) / (more.input - less.input);

        let output = self
            .interpolation
            .interpolate(s, &less.output, &more.output);

        Some(output)
    }
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub name: Option<String>,
    pub channels: Vec<AnimationChannel>,
    pub samplers: Vec<AnimationSampler>,
}
