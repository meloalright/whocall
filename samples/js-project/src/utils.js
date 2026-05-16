export function formatName(first, last) {
  return `${first} ${last}`;
}

export function capitalize(s) {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

export const greet = (name) => {
  return `Hello, ${capitalize(name)}!`;
};
