class User:
    def __init__(self, name):
        self.name = name

    def greet(self):
        return f"Hello, {self.name}"


class Admin(User):
    def greet(self):
        return f"Admin: {self.name}"

    def promote(self, user):
        print(f"Promoting {user.name}")


class Dog:
    def speak(self):
        return "Woof"


class Cat:
    def speak(self):
        return "Meow"
