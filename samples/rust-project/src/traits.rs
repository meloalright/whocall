use crate::RenderCtx;
use crate::base_trait::Renderer;
use crate::text::render::{render_block, render_text};

pub struct TextRenderer;

impl Renderer for TextRenderer {
    fn render(&self, ctx: &RenderCtx) {
        println!("[TextRenderer] {} lines", ctx.buffer.len());
    }

    fn name(&self) -> &str {
        "text"
    }
}

pub struct GpuRenderer {
    shader_loaded: bool,
}

impl GpuRenderer {
    pub fn new() -> Self {
        Self {
            shader_loaded: false,
        }
    }

    pub fn load_shaders(&mut self) {
        self.shader_loaded = true;
    }
}

impl Renderer for GpuRenderer {
    fn render(&self, ctx: &RenderCtx) {
        if self.shader_loaded {
            println!("[GpuRenderer] rendering {} lines with shaders", ctx.buffer.len());
        } else {
            println!("[GpuRenderer] fallback: {} lines", ctx.buffer.len());
        }
    }

    fn name(&self) -> &str {
        "gpu"
    }
}

pub fn render_all(ctx: &mut RenderCtx, renderers: &[&dyn Renderer]) {
    render_text(ctx, "--- render pass start ---");
    for r in renderers {
        println!("rendering with: {}", r.name());
        r.render(ctx);
    }
    render_block(ctx, &["--- render pass end ---"]);
}
