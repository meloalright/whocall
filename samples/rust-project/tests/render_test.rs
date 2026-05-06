mod common {
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
    }

    pub fn render_text(ctx: &mut RenderCtx, text: &str) {
        let formatted = if text.len() > ctx.width as usize {
            format!("{}...", &text[..ctx.width as usize - 3])
        } else {
            text.to_string()
        };
        ctx.push_line(&formatted);
    }
}

use common::{render_text, RenderCtx};

#[test]
fn test_render_text_short() {
    let mut ctx = RenderCtx::new(80, 24);
    render_text(&mut ctx, "hello");
    assert_eq!(ctx.buffer, vec!["hello"]);
}

#[test]
fn test_render_text_truncate() {
    let mut ctx = RenderCtx::new(10, 1);
    render_text(&mut ctx, "this is a very long string");
    assert_eq!(ctx.buffer.len(), 1);
    assert!(ctx.buffer[0].len() <= 10);
}

#[test]
fn test_render_text_empty() {
    let mut ctx = RenderCtx::new(80, 24);
    render_text(&mut ctx, "");
    assert_eq!(ctx.buffer, vec![""]);
}

#[test]
fn test_multiple_renders() {
    let mut ctx = RenderCtx::new(80, 24);
    render_text(&mut ctx, "line 1");
    render_text(&mut ctx, "line 2");
    render_text(&mut ctx, "line 3");
    assert_eq!(ctx.buffer.len(), 3);
}
