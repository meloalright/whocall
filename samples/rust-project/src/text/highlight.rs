use crate::RenderCtx;
use crate::text::render::render_text;

pub fn highlight_keyword(ctx: &mut RenderCtx, text: &str, keyword: &str) {
    let highlighted = text.replace(keyword, &format!("[{keyword}]"));
    render_text(ctx, &highlighted);
}

pub fn highlight_search(ctx: &mut RenderCtx, text: &str, query: &str) {
    if text.contains(query) {
        let marked = text.replace(query, &format!("<<{query}>>"));
        render_text(ctx, &marked);
    }
}
