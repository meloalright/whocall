# Edge case: comment should not appear as a caller
# render_text("this is a comment, not a call")

def commented_out_call():
    pass


# Edge case: same-name function should not appear as caller of utils.render_text
def render_text(local_arg):
    return f"local: {local_arg}"


def calls_local_render_text():
    render_text("local only")
