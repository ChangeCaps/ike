use bytemuck::{Pod, Zeroable};
use ike_core::Resources;
use ike_reflect::{egui, Inspect, Reflect, ReflectInspect};
use serde::{Deserialize, Serialize};

macro_rules! impl_color {
    ($ident:ident, $ty:ty, $zero:expr, $one:expr) => {
        #[repr(C)]
        #[derive(
            Reflect, Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable, Serialize, Deserialize,
        )]
        #[reflect(value)]
        #[reflect(register(ReflectInspect))]
        pub struct $ident {
            pub r: $ty,
            pub g: $ty,
            pub b: $ty,
            pub a: $ty,
        }

        impl Inspect for $ident {
            fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> egui::Response {
                ui.columns(4, |columns| {
                    columns[0].add(egui::DragValue::new(&mut self.r))
                        | columns[1].add(egui::DragValue::new(&mut self.g))
                        | columns[2].add(egui::DragValue::new(&mut self.b))
                        | columns[3].add(egui::DragValue::new(&mut self.a))
                })
            }
        }

        impl $ident {
            pub const TRANSPARENT: Self = Self::rgba($zero, $zero, $zero, $zero);
            pub const BLACK: Self = Self::rgb($zero, $zero, $zero);
            pub const WHITE: Self = Self::rgb($one, $one, $one);
            pub const RED: Self = Self::rgb($one, $zero, $zero);
            pub const GREEN: Self = Self::rgb($zero, $one, $zero);
            pub const BLUE: Self = Self::rgb($zero, $zero, $one);

            #[inline]
            pub const fn rgb(r: $ty, g: $ty, b: $ty) -> Self {
                Self { r, g, b, a: $one }
            }

            #[inline]
            pub const fn rgba(r: $ty, g: $ty, b: $ty, a: $ty) -> Self {
                Self { r, g, b, a }
            }
        }

        impl Into<[$ty; 4]> for $ident {
            #[inline]
            fn into(self) -> [$ty; 4] {
                [self.r, self.g, self.b, self.a]
            }
        }

        impl From<[$ty; 4]> for $ident {
            #[inline]
            fn from([r, g, b, a]: [$ty; 4]) -> Self {
                Self::rgba(r, g, b, a)
            }
        }

        impl std::ops::Mul<$ty> for $ident {
            type Output = $ident;

            #[inline]
            fn mul(self, rhs: $ty) -> Self::Output {
                Self::rgba(self.r * rhs, self.g * rhs, self.b * rhs, self.a * rhs)
            }
        }
    };
}

impl_color!(Color, f32, 0.0, 1.0);
impl_color!(Color8, u8, 0, 255);
impl_color!(Color16, u16, 0, u16::MAX);

impl Into<[u8; 4]> for Color {
    #[inline]
    fn into(self) -> [u8; 4] {
        [
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8,
            (self.a * 255.0).round() as u8,
        ]
    }
}

impl From<[u8; 4]> for Color {
    #[inline]
    fn from([r, g, b, a]: [u8; 4]) -> Self {
        Self::rgba(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }
}

impl From<Color> for Color8 {
    #[inline]
    fn from(color: Color) -> Self {
        Color8::from(Into::<[u8; 4]>::into(color))
    }
}

impl Into<ike_wgpu::Color> for Color {
    #[inline]
    fn into(self) -> ike_wgpu::Color {
        ike_wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}
