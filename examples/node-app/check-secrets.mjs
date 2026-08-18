// The application itself needs no knowledge of `kivro`.
//
// Started with `kivro run -- node app.mjs`, these variables are already in
// the process environment. There is no dotenv import, and no .env file.

const required = ["DATABASE_URL", "AUTH0_CLIENT_ID", "AUTH0_CLIENT_SECRET"];

const missing = required.filter((name) => !process.env[name]);
if (missing.length > 0) {
  // This should be unreachable under `kivro run`, which refuses to start the
  // child when a required secret is missing. It is here for the case where
  // someone runs `node app.mjs` directly.
  console.error(
    `Not started through \`kivro run\`? Missing: ${missing.join(", ")}`,
  );
  process.exit(1);
}

console.log(`DATABASE_URL is ${process.env.DATABASE_URL.length} bytes`);
console.log("started");
