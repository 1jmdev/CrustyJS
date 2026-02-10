const person = { name: "Alice", age: 30, city: "Paris" };
const { name, age: years = 0, ...rest } = person;

const nums = [1, 2, 3, 4, 5];
const [first, , third, ...tail] = nums;

function greet({ name, age = 0 }) {
  console.log(name + " is " + age);
}

const pick = ({ city = "unknown" }) => city;

console.log(name);
console.log(years);
console.log(rest.city);
console.log(first);
console.log(third);
console.log(tail.length);
greet({ name: "Bob" });
console.log(pick(rest));
