use target::Target;

pub(crate) struct ImageTarget {}

impl Target for ImageTarget {
    type AcquireFuture: GpuFuture + Send + Sync + 'static;

    fn render_pass(&self) -> Arc<RenderPassAbstract + Send + Sync>;
    fn resize(&mut self, queue: &Arc<Queue>, dimensions: [u32; 2]) -> Result<(), failure::Error>;
    fn acquire(
        &self,
        queue: &Arc<Queue>,
    ) -> Result<(Arc<FramebufferAbstract + Send + Sync>, Self::AcquireFuture), AcquireError>;
    fn present<F>(&self, queue: &Arc<Queue>, fut: F) -> Box<GpuFuture + Send + Sync>
    where
        F: GpuFuture + Send + Sync + 'static;
}
