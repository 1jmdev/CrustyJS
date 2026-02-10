class Animal {
  constructor(name) {
    this.name = name;
  }
  speak() {
    return this.name + " makes a noise";
  }
}

class Dog extends Animal {
  constructor(name, breed) {
    super(name);
    this.breed = breed;
  }
  speak() {
    return this.name + " barks";
  }
}

const dogs = [new Dog("Rex", "Shepherd"), new Dog("Bella", "Lab")];
const names = dogs.map(d => d.name);

console.log(names);
console.log(dogs[0].speak());
console.log(typeof dogs[0]);
console.log(dogs[0] instanceof Dog);
console.log(dogs[0] instanceof Animal);
