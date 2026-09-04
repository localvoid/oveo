function test(a, b, c, d) {
  return [
    new URL("https://cdn.example.com/x.css", import.meta.url).href,
    new URL("/absolute.css", import.meta.url).href,
    new URL("?query=1", import.meta.url).href,
    new URL("#hash", import.meta.url).href,
  ];
}
