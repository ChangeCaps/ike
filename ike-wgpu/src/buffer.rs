use std::{future::Future, ops::Deref, pin::Pin};

pub struct BufferInitDescriptor<'a> {
    pub label: Option<&'a str>,
    pub contents: &'a [u8],
    pub usage: crate::BufferUsages,
}

pub(crate) unsafe trait BufferTrait: std::fmt::Debug {
    fn slice(
        &self,
        start: std::ops::Bound<&u64>,
        end: std::ops::Bound<&u64>,
    ) -> crate::BufferSlice<'_>;

    fn unmap(&self);
}

#[cfg(feature = "wgpu")]
unsafe impl BufferTrait for wgpu::Buffer {
    #[inline]
    fn slice(
        &self,
        start: std::ops::Bound<&u64>,
        end: std::ops::Bound<&u64>,
    ) -> crate::BufferSlice<'_> {
        crate::BufferSlice(Box::new(self.slice((start, end))))
    }

    #[inline]
    fn unmap(&self) {
        self.unmap();
    }
}

#[derive(Debug)]
pub struct Buffer(pub(crate) Box<dyn BufferTrait>);

impl Buffer {
    #[inline]
    pub fn as_entire_binding(&self) -> crate::BindingResource<'_> {
        crate::BindingResource::Buffer(crate::BufferBinding {
            buffer: self,
            offset: 0,
            size: None,
        })
    }

    #[inline]
    pub fn slice<S: std::ops::RangeBounds<u64>>(&self, bounds: S) -> crate::BufferSlice<'_> {
        self.0.slice(bounds.start_bound(), bounds.end_bound())
    }

    #[inline]
    pub fn unmap(&self) {
        self.0.unmap();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum MapMode {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug)]
pub struct BufferAsyncError;

pub(crate) unsafe trait BufferSliceTrait<'a>: std::fmt::Debug {
    fn map_async<'b>(
        &'b self,
        mode: MapMode,
    ) -> Pin<Box<dyn Future<Output = Result<(), BufferAsyncError>> + Send + 'b>>;

    fn get_mapped_range(&self) -> BufferView<'a>;
}

#[cfg(feature = "wgpu")]
unsafe impl<'a> BufferSliceTrait<'a> for wgpu::BufferSlice<'a> {
    #[inline]
    fn map_async<'b>(
        &'b self,
        mode: MapMode,
    ) -> Pin<Box<dyn Future<Output = Result<(), BufferAsyncError>> + Send + 'b>> {
        let mode = match mode {
            MapMode::Read => wgpu::MapMode::Read,
            MapMode::Write => wgpu::MapMode::Write,
        };

        Box::pin(async move { self.map_async(mode).await.map_err(|_| BufferAsyncError) })
    }

    #[inline]
    fn get_mapped_range(&self) -> BufferView<'a> {
        BufferView(Box::new(self.get_mapped_range()))
    }
}

pub struct BufferSlice<'a>(pub(crate) Box<dyn BufferSliceTrait<'a> + 'a>);

impl<'a> BufferSlice<'a> {
    #[inline]
    pub async fn map_async(&self, mode: MapMode) -> Result<(), BufferAsyncError> {
        self.0.map_async(mode).await
    }

    #[inline]
    pub fn get_mapped_range(&self) -> BufferView<'a> {
        self.0.get_mapped_range()
    }
}

pub(crate) unsafe trait BufferViewTrait<'a>: std::fmt::Debug {
    fn bytes(&self) -> &[u8];
}

#[cfg(feature = "wgpu")]
unsafe impl<'a> BufferViewTrait<'a> for wgpu::BufferView<'a> {
    fn bytes(&self) -> &[u8] {
        self
    }
}

#[derive(Debug)]
pub struct BufferView<'a>(pub(crate) Box<dyn BufferViewTrait<'a> + 'a>);

impl<'a> AsRef<[u8]> for BufferView<'a> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.bytes()
    }
}

impl<'a> Deref for BufferView<'a> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
