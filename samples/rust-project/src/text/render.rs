use crate::RenderCtx;

pub fn render_text(ctx: &mut RenderCtx, text: &str) {
    let formatted = layout_text(text, ctx.width);
    ctx.push_line(&formatted);
}

fn layout_text(text: &str, max_width: u32) -> String {
    if text.len() > max_width as usize {
        format!("{}...", &text[..max_width as usize - 3])
    } else {
        text.to_string()
    }
}

pub fn render_block(ctx: &mut RenderCtx, lines: &[&str]) {
    for line in lines {
        render_text(ctx, line);
    }
}

pub fn measure_text(text: &str) -> usize {
    text.chars().count()
}
