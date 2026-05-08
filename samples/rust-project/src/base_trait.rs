use crate::RenderCtx;

pub trait Renderer {
    fn render(&self, ctx: &RenderCtx);
    fn name(&self) -> &str;
}
