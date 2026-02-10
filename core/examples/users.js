export async function fetchUser(id) {
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve({
        id: id,
        name: "Ada Lovelace",
        age: 36,
        role: "engineer",
        active: true,
      });
    }, 5);
  });
}
