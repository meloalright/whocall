export class Dog {
  constructor(name) {
    this.name = name;
  }

  speak() {
    return `${this.name} says woof!`;
  }

  greet(who) {
    return `${this.name} greets ${who}`;
  }
}

export class Cat {
  speak() {
    return 'Meow!';
  }

  greet(who) {
    return `Cat greets ${who}`;
  }
}
