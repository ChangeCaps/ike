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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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
            Interpolation::Linear => match a {
                SampleOutput::Translation(a) => {
                    let b = b.unwrap_translation();

                    SampleOutput::Translation(a.lerp(b, s))
                }
                SampleOutput::Rotation(a) => {
                    let b = b.unwrap_rotation();

                    if a.dot(b) > 0.0 {
                        SampleOutput::Rotation(a.slerp(b, s))
                    } else {
                        SampleOutput::Rotation((-*a).slerp(b, s))
                    }
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
            Interpolation::CubicSpline => unimplemented!(),
            Interpolation::Step => a.clone(),
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
        let less = self.samples.range(..SampleInput(t)).next_back();
        let more = self.samples.range(SampleInput(t)..).next();

        let less = if let Some((_, less)) = less {
            less
        } else {
            return Some(more?.1.output.clone());
        };

        let more = if let Some((_, more)) = more {
            more
        } else {
            return Some(less.output.clone());
        };

        let s = (t - less.input) / (more.input - less.input);

        let output = self
            .interpolation
            .interpolate(s, &less.output, &more.output);

        Some(output)
    }
}

#[derive(Clone, Debug)]
pub enum AnimationIdent<'a> {
    Animation(&'a Animation),
    Index(usize),
    Name(&'a str),
}

impl From<usize> for AnimationIdent<'_> {
    #[inline]
    fn from(index: usize) -> Self {
        Self::Index(index)
    }
}

impl<'a> From<&'a str> for AnimationIdent<'a> {
    #[inline]
    fn from(name: &'a str) -> Self {
        Self::Name(name)
    }
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub name: Option<String>,
    pub channels: Vec<AnimationChannel>,
    pub samplers: Vec<AnimationSampler>,
}

#[derive(Clone, Debug)]
pub struct Animations {
    pub animations: Vec<Animation>,
}

impl Animations {
    #[inline]
    pub fn get<'a>(&'a self, ident: impl Into<AnimationIdent<'a>>) -> Option<&Animation> {
        match ident.into() {
            AnimationIdent::Animation(animation) => Some(animation),
            AnimationIdent::Index(idx) => self.animations.get(idx),
            AnimationIdent::Name(name) => self
                .animations
                .iter()
                .find(|anim| anim.name.as_ref().map(|n| n == name).unwrap_or(false)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum AnimationError {
    AnimationNotFound,
}

impl std::fmt::Display for AnimationError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnimationNotFound => write!(f, "Animation not found in Animations"),
        }
    }
}

impl std::error::Error for AnimationError {}
