// A readiness gate that turns `kivro status --json` into a useful message.
//
// Prefer plain `kivro status --quiet` when you only need the exit code; this
// exists for when you want to format the failure yourself.

import { execFileSync } from "node:child_process";

function secretsStatus() {
  try {
    const stdout = execFileSync("kivro", ["status", "--json"], {
      encoding: "utf8",
    });
    return JSON.parse(stdout);
  } catch (error) {
    // Exit code 3 (missing secrets) still writes valid JSON to stdout.
    if (error.stdout) return JSON.parse(error.stdout);
    // Anything else — no manifest, no keyring — is a real failure.
    console.error(error.stderr?.toString() ?? String(error));
    process.exit(1);
  }
}

const status = secretsStatus();

if (status.satisfied) {
  console.log(`${status.project}/${status.environment}: ready`);
  process.exit(0);
}

console.error(
  `${status.project}/${status.environment} is missing ${status.missing.length} secret(s):\n`,
);
for (const name of status.missing) {
  console.error(`    kivro set ${name}`);
}
console.error("\nThen re-run this command.");
process.exit(1);
