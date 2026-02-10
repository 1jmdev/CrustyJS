import { fetchUser } from "./users.js";

async function main() {
  const { name, age, ...rest } = await fetchUser(1);
  const [first, ...others] = name.split(" ");

  const result = await new Promise((resolve) => {
    setTimeout(() => {
      resolve(`Hello ${first}, age ${age}`);
    }, 10);
  });

  const nums = [3, 1, 4, 1, 5];
  const sum = nums.reduce((acc, n) => acc + n, 0);
  const sorted = [...nums].sort((a, b) => a - b);

  console.log(result);
  console.log(JSON.stringify({ sum: sum, sorted: sorted, timestamp: Date.now() }));
  console.log(Object.keys(rest));
  console.log(others.length);
}

main().catch((e) => console.log(e.message));
