use std::path::PathBuf;

use egui::{DragValue, Response};
use glam::{
    EulerRot, IVec2, IVec3, IVec4, Mat2, Mat3, Mat4, Quat, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
};
use ike_core::Resources;

use crate::Inspect;

impl Inspect for Vec2 {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> Response {
        ui.columns(2, |columns| {
            columns[0].add(DragValue::new(&mut self.x))
                | columns[1].add(DragValue::new(&mut self.y))
        })
    }
}

impl Inspect for Vec3 {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> Response {
        ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut self.x))
                | columns[1].add(DragValue::new(&mut self.y))
                | columns[2].add(DragValue::new(&mut self.z))
        })
    }
}

impl Inspect for Vec4 {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> Response {
        ui.columns(4, |columns| {
            columns[0].add(DragValue::new(&mut self.x))
                | columns[1].add(DragValue::new(&mut self.y))
                | columns[2].add(DragValue::new(&mut self.z))
                | columns[3].add(DragValue::new(&mut self.w))
        })
    }
}

impl Inspect for IVec2 {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> Response {
        ui.columns(2, |columns| {
            columns[0].add(DragValue::new(&mut self.x))
                | columns[1].add(DragValue::new(&mut self.y))
        })
    }
}

impl Inspect for IVec3 {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> Response {
        ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut self.x))
                | columns[1].add(DragValue::new(&mut self.y))
                | columns[2].add(DragValue::new(&mut self.z))
        })
    }
}

impl Inspect for IVec4 {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> Response {
        ui.columns(4, |columns| {
            columns[0].add(DragValue::new(&mut self.x))
                | columns[1].add(DragValue::new(&mut self.y))
                | columns[2].add(DragValue::new(&mut self.z))
                | columns[3].add(DragValue::new(&mut self.w))
        })
    }
}

impl Inspect for UVec2 {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> Response {
        ui.columns(2, |columns| {
            columns[0].add(DragValue::new(&mut self.x))
                | columns[1].add(DragValue::new(&mut self.y))
        })
    }
}

impl Inspect for UVec3 {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> Response {
        ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut self.x))
                | columns[1].add(DragValue::new(&mut self.y))
                | columns[2].add(DragValue::new(&mut self.z))
        })
    }
}

impl Inspect for UVec4 {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> Response {
        ui.columns(4, |columns| {
            columns[0].add(DragValue::new(&mut self.x))
                | columns[1].add(DragValue::new(&mut self.y))
                | columns[2].add(DragValue::new(&mut self.z))
                | columns[3].add(DragValue::new(&mut self.w))
        })
    }
}

impl Inspect for Quat {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> Response {
        let (mut y, mut x, mut z) = self.to_euler(EulerRot::YXZ);
        x = x.to_degrees();
        y = y.to_degrees();
        z = z.to_degrees();

        let response = ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut x))
                | columns[1].add(DragValue::new(&mut y))
                | columns[2].add(DragValue::new(&mut z))
        });

        if response.changed() {
            *self = Quat::from_euler(
                EulerRot::YXZ,
                y.to_radians(),
                x.to_radians(),
                z.to_radians(),
            );
        }

        response
    }
}

impl Inspect for Mat2 {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> egui::Response {
        ui.columns(2, |columns| {
            columns[0].add(DragValue::new(&mut self.x_axis.x))
                | columns[1].add(DragValue::new(&mut self.x_axis.y))
        }) | ui.columns(2, |columns| {
            columns[0].add(DragValue::new(&mut self.y_axis.x))
                | columns[1].add(DragValue::new(&mut self.y_axis.y))
        })
    }
}

impl Inspect for Mat3 {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> egui::Response {
        ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut self.x_axis.x))
                | columns[1].add(DragValue::new(&mut self.x_axis.y))
                | columns[2].add(DragValue::new(&mut self.x_axis.z))
        }) | ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut self.y_axis.x))
                | columns[1].add(DragValue::new(&mut self.y_axis.y))
                | columns[2].add(DragValue::new(&mut self.y_axis.z))
        }) | ui.columns(3, |columns| {
            columns[0].add(DragValue::new(&mut self.z_axis.x))
                | columns[1].add(DragValue::new(&mut self.z_axis.y))
                | columns[2].add(DragValue::new(&mut self.z_axis.z))
        })
    }
}

impl Inspect for Mat4 {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> egui::Response {
        ui.columns(4, |columns| {
            columns[0].add(DragValue::new(&mut self.x_axis.x))
                | columns[1].add(DragValue::new(&mut self.x_axis.y))
                | columns[2].add(DragValue::new(&mut self.x_axis.z))
                | columns[3].add(DragValue::new(&mut self.x_axis.w))
        }) | ui.columns(4, |columns| {
            columns[0].add(DragValue::new(&mut self.y_axis.x))
                | columns[1].add(DragValue::new(&mut self.y_axis.y))
                | columns[2].add(DragValue::new(&mut self.y_axis.z))
                | columns[3].add(DragValue::new(&mut self.y_axis.w))
        }) | ui.columns(4, |columns| {
            columns[0].add(DragValue::new(&mut self.z_axis.x))
                | columns[1].add(DragValue::new(&mut self.z_axis.y))
                | columns[2].add(DragValue::new(&mut self.z_axis.z))
                | columns[3].add(DragValue::new(&mut self.z_axis.w))
        }) | ui.columns(4, |columns| {
            columns[0].add(DragValue::new(&mut self.w_axis.x))
                | columns[1].add(DragValue::new(&mut self.w_axis.y))
                | columns[2].add(DragValue::new(&mut self.w_axis.z))
                | columns[3].add(DragValue::new(&mut self.w_axis.w))
        })
    }
}

macro_rules! impl_num {
    ($ty:ty) => {
        impl Inspect for $ty {
            fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> egui::Response {
                ui.add(DragValue::new(self))
            }
        }
    };
}

impl_num!(i8);
impl_num!(i16);
impl_num!(i32);
impl_num!(i64);
impl_num!(u8);
impl_num!(u16);
impl_num!(u32);
impl_num!(u64);
impl_num!(f32);
impl_num!(f64);

impl Inspect for bool {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> egui::Response {
        ui.checkbox(self, "")
    }
}

impl Inspect for String {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> egui::Response {
        ui.text_edit_multiline(self)
    }
}

impl Inspect for PathBuf {
    fn inspect(&mut self, ui: &mut egui::Ui, _resources: &Resources) -> egui::Response {
        let mut string = self.to_str().unwrap().to_owned();

        let response = ui.text_edit_singleline(&mut string);

        if response.changed() {
            *self = string.into();
        }

        response
    }
}
