try {
  throw new Error("inner oops");
} catch (e) {
  console.log(e.message);
} finally {
  console.log("cleanup");
}

try {
  throw "plain value";
} catch (e) {
  console.log(e);
}

try {
  throw new Error("boom");
  console.log("never runs")
} catch (e) {
  console.log(e)
}
