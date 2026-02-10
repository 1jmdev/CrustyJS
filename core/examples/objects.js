let animal = {
  name: "Rex",
  speak: () => this.name + " barks"
};

console.log(animal.speak());

let proto = { kind: "canine" };
let dog = Object.create(proto);
dog.name = "Bella";

console.log(dog.name);
console.log(dog.kind);
