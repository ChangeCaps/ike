use crate::{FromReflect, Reflect, ReflectMut, ReflectRef};

pub trait ReflectTuple: Reflect {
    fn field(&self, index: usize) -> Option<&dyn Reflect>;
    fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;
    fn field_len(&self) -> usize;
}

impl dyn ReflectTuple {
    pub fn partial_eq(&self, other: &dyn ReflectTuple) -> bool {
        if self.type_name() != other.type_name() {
            return false;
        }

        for i in 0..self.field_len() {
            if let (Some(a), Some(b)) = (self.field(i), other.field(i)) {
                if !a.partial_eq(b) {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    pub fn clone_dynamic(&self) -> DynamicTuple {
        let mut this = DynamicTuple::default();
        this.set_name(self.type_name());

        for index in 0..self.field_len() {
            let field = self.field(index).unwrap();
            this.push_boxed(field.clone_dynamic());
        }

        this
    }
}

#[derive(Default)]
pub struct DynamicTuple {
    name: String,
    fields: Vec<Box<dyn Reflect>>,
}

impl DynamicTuple {
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn push_boxed(&mut self, value: Box<dyn Reflect>) {
        self.fields.push(value);
    }
}

impl ReflectTuple for DynamicTuple {
    fn field(&self, index: usize) -> Option<&dyn Reflect> {
        self.fields.get(index).map(|field| field.as_ref())
    }

    fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.fields.get_mut(index).map(|field| field.as_mut())
    }

    fn field_len(&self) -> usize {
        self.fields.len()
    }
}

impl Reflect for DynamicTuple {
    fn type_name(&self) -> &str {
        &self.name
    }

    fn reflect_ref(&self) -> ReflectRef {
        ReflectRef::Tuple(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut {
        ReflectMut::Tuple(self)
    }
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
