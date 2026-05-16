// Edge case: comment should not appear as a caller
// capitalize("this is a comment, not a call")

function commentedOutCall() {
}

// Edge case: same-name function should not appear as caller of utils.capitalize
function capitalize(localArg: string): string {
  return "local: " + localArg;
}

function callsLocalCapitalize() {
  capitalize("local only");
}
