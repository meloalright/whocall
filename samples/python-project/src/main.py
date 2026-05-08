from src.utils import render_text
from src.models import User


def main():
    user = User("Alice")
    render_text(user.name)
    print("done")


def cli():
    main()
