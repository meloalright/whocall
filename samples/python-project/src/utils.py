def render_text(text):
    formatted = format_output(text)
    print(formatted)


def format_output(text):
    return f"[output] {text}"
