// Runs the collector from this machine against the remote D1 database: for the first
// fill (Cloudflare keeps 30 days of analytics) or to catch up after the cron fell
// behind further than MAX_DAYS. Same code as the Worker; D1 is reached over the REST API.
//
//   CF_API_TOKEN=<token> node backfill.mjs [days]
//
// The token needs Zone › Analytics › Read on quiet.md and Account › D1 › Edit.
// GITHUB_TOKEN is optional. Days defaults to 31.
import { collect } from './src/index.js';

const ACCOUNT = 'b56182e80c54647821efcd93b6371314';
const DATABASE = '4b58cbda-8485-4861-8962-593e202803ea';
const token = process.env.CF_API_TOKEN;
if (!token) { console.error('CF_API_TOKEN is not set'); process.exit(1); }

async function query(batch) {
  const res = await fetch(`https://api.cloudflare.com/client/v4/accounts/${ACCOUNT}/d1/database/${DATABASE}/query`, {
    method: 'POST',
    headers: { authorization: `Bearer ${token}`, 'content-type': 'application/json' },
    body: JSON.stringify({ batch }),
  });
  const body = await res.json();
  if (!body.success) throw new Error(`d1: ${JSON.stringify(body.errors)}`);
  return body.result;
}

// Just enough of the D1 binding for the collector.
class Stmt {
  constructor(sql, params = []) { this.sql = sql; this.params = params; }
  bind(...params) { return new Stmt(this.sql, params); }
  async all() { return { results: (await query([this]))[0].results }; }
  async first(col) { const r = (await this.all()).results[0]; return r ? (col ? r[col] : r) : null; }
  async run() { return (await query([this]))[0]; }
  toJSON() { return { sql: this.sql, params: this.params }; }
}
const DB = { prepare: (sql) => new Stmt(sql), batch: (stmts) => query(stmts) };

const env = {
  DB, CF_API_TOKEN: token, GITHUB_TOKEN: process.env.GITHUB_TOKEN,
  ZONE_ID: '411c824a5a3ea80fcfd2d5551529366b', GITHUB_REPO: 's1dashu/quiet-md',
  MAX_DAYS: process.argv[2] || '31',
};
await collect(env);
const runs = await query([new Stmt('SELECT source, day, rows, error FROM runs ORDER BY id DESC LIMIT 40')]);
console.table(runs[0].results.reverse().map((r) => ({ ...r, error: r.error && r.error.slice(0, 80) })));
