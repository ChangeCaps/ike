use crate::{FromReflect, Reflect, ReflectMut, ReflectRef};

pub trait ReflectTuple: Reflect {
    fn field(&self, index: usize) -> Option<&dyn Reflect>;
    fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;
    fn field_len(&self) -> usize;
}

macro_rules! impl_reflect_tuple {
	() => {};
    ($first:ident $(,$name:ident)*) => {
		impl_reflect_tuple!($($name),*);
		impl_reflect_tuple!(@ $first $(,$name)*);
	};
    (@ $($name:ident),*) => {
		impl<$($name: Reflect),*> Reflect for ($($name,)*) {
			fn reflect_ref(&self) -> ReflectRef {
				ReflectRef::Tuple(self)
			}

			fn reflect_mut(&mut self) -> ReflectMut {
				ReflectMut::Tuple(self)
			}
		}

		#[allow(unused, non_snake_case)]
		impl<$($name: FromReflect),*> FromReflect for ($($name,)*) {
			fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
				let tuple = reflect.reflect_ref().get_tuple()?;

				let mut index = 0;
				Some(($(
					{
						let field = tuple.field(index)?;
						index += 1;
						$name::from_reflect(field)?
					},
				)*))
			}
		}

		#[allow(unused, non_snake_case)]
		impl<$($name: Reflect),*> ReflectTuple for ($($name,)*) {
			fn field(&self, index: usize) -> Option<&dyn Reflect> {
				let ($($name,)*) = self;

				let mut i = 0;

				$(
					if index == i {
						return Some($name);
					}

					i += 1;
				)*

				None
			}

			fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
				let ($($name,)*) = self;

				let mut i = 0;

				$(
					if index == i {
						return Some($name);
					}

					i += 1;
				)*

				None
			}

			fn field_len(&self) -> usize {
				let mut i = 0;

				$(
					let $name = ();
					i += 1;
				)*

				i
			}
		}
	};
}

impl_reflect_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
