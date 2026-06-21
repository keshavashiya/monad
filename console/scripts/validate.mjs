/**
 * Vault schema validator (library-based, no vulnerable CLI dependency).
 *
 * Lives under console/ so `import 'ajv'` resolves the locally-installed library.
 * Run from the repo root: `node console/scripts/validate-vault.mjs`.
 */
import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import Ajv from 'ajv';
import addFormats from 'ajv-formats';

const VAULT_DIR = 'vault';
const SCHEMA = join(VAULT_DIR, 'schema.json');

const schema = JSON.parse(readFileSync(SCHEMA, 'utf8'));
const ajv = new Ajv({ strict: false, allErrors: true });
addFormats(ajv);

// The vault uses an empty string to mean "field unset" (e.g. a role with no
// public URL). Treat "" as valid for the `uri` format while still validating
// any non-empty value as a real URI.
const baseUri = ajv.formats.uri;
const baseUriFn = typeof baseUri === 'function' ? baseUri : baseUri.validate;
ajv.addFormat('uri', (s) => s === '' || baseUriFn(s));

const validate = ajv.compile(schema);

let ok = true;
for (const file of readdirSync(VAULT_DIR)) {
  if (!file.endsWith('.json') || join(VAULT_DIR, file) === SCHEMA) continue;
  const path = join(VAULT_DIR, file);
  console.log(`    validating ${path}...`);
  const data = JSON.parse(readFileSync(path, 'utf8'));
  if (validate(data)) continue;
  ok = false;
  console.error(`    FAILED: ${path} does not conform to schema`);
  for (const err of validate.errors ?? []) {
    console.error(`      ${err.instancePath || '/'} ${err.message}`);
  }
}

if (ok) {
  console.log('    all vault files valid (schema-checked).');
} else {
  process.exit(1);
}
