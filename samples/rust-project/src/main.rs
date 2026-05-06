mod text;
mod traits;

use text::render::render_text;
use traits::{GpuRenderer, Renderer, TextRenderer};

fn main() {
    let mut ctx = RenderCtx::new(80, 24);

    render_text(&mut ctx, "Hello, ast-call!");

    let text_renderer = TextRenderer;
    text_renderer.render(&ctx);

    let gpu_renderer = GpuRenderer::new();
    gpu_renderer.render(&ctx);

    run_frame(&mut ctx);
}

pub struct RenderCtx {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<String>,
}

impl RenderCtx {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            buffer: Vec::new(),
        }
    }

    pub fn push_line(&mut self, line: &str) {
        self.buffer.push(line.to_string());
    }

    pub fn flush(&self) {
        for line in &self.buffer {
            println!("{line}");
        }
    }
}

fn run_frame(ctx: &mut RenderCtx) {
    paint(ctx);
    ctx.flush();
}

fn paint(ctx: &mut RenderCtx) {
    render_text(ctx, "frame start");
    draw_editor(ctx);
    render_text(ctx, "frame end");
}

fn draw_editor(ctx: &mut RenderCtx) {
    render_text(ctx, "editor content line 1");
    render_text(ctx, "editor content line 2");
    format_line(ctx, "  hello world  ");
}

fn format_line(ctx: &mut RenderCtx, raw: &str) {
    let trimmed = raw.trim();
    render_text(ctx, trimmed);
}
