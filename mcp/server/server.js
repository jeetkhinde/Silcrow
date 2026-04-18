import { Server }              from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  ListToolsRequestSchema,
  CallToolRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';

import { resolve }              from 'path';
import { loadDocs, resetStore } from './lib/docs-loader.js';

import { definition as searchDocsDef,    handler as searchDocsHandler    } from './tools/search-docs.js';
import { definition as getDocDef,         handler as getDocHandler         } from './tools/get-doc.js';
import { definition as getSectionDef,     handler as getSectionHandler     } from './tools/get-section.js';
import { definition as getExamplesDef,    handler as getExamplesHandler    } from './tools/get-examples.js';
import { definition as analyzeUsageDef,   handler as analyzeUsageHandler   } from './tools/analyze-usage.js';

const TOOLS = [
  { definition: searchDocsDef,   handler: searchDocsHandler   },
  { definition: getDocDef,       handler: getDocHandler       },
  { definition: getSectionDef,   handler: getSectionHandler   },
  { definition: getExamplesDef,  handler: getExamplesHandler  },
  { definition: analyzeUsageDef, handler: analyzeUsageHandler },
];

const TOOL_MAP = new Map(TOOLS.map(t => [t.definition.name, t]));

function resolveDocsPath() {
  const flag = process.argv.indexOf('--docs');
  if (flag !== -1 && process.argv[flag + 1]) {
    return resolve(process.argv[flag + 1]);
  }
  if (process.env.SILCROW_DOCS_PATH) {
    return resolve(process.env.SILCROW_DOCS_PATH);
  }
  return undefined;
}

function loadStore(docsPath) {
  resetStore();
  const store = loadDocs(docsPath);
  process.stderr.write(
    `[silcrow-mcp] Loaded — ${store.docs.length} docs, ${store.sections.length} sections\n`,
  );
  return store;
}

async function main() {
  const docsPath = resolveDocsPath();

  // ctx is an object so handler closures always read the current store reference
  const ctx = { store: null };
  try {
    ctx.store = loadStore(docsPath);
  } catch (err) {
    process.stderr.write(`[silcrow-mcp] Fatal: ${err.message}\n`);
    process.exit(1);
  }

  // SIGHUP reloads docs without dropping the connection
  process.on('SIGHUP', () => {
    process.stderr.write('[silcrow-mcp] SIGHUP received — reloading docs\n');
    try {
      ctx.store = loadStore(docsPath);
    } catch (err) {
      process.stderr.write(`[silcrow-mcp] Reload failed (keeping previous index): ${err.message}\n`);
    }
  });

  const server = new Server(
    { name: 'silcrow-mcp', version: '1.0.0' },
    { capabilities: { tools: {} } },
  );

  server.setRequestHandler(ListToolsRequestSchema, () => ({
    tools: TOOLS.map(t => t.definition),
  }));

  server.setRequestHandler(CallToolRequestSchema, (request) => {
    const { name, arguments: args } = request.params;

    const tool = TOOL_MAP.get(name);
    if (!tool) {
      return errorResponse(`Unknown tool: "${name}"`);
    }

    let result;
    try {
      result = tool.handler(args ?? {}, ctx.store);
    } catch (err) {
      return errorResponse(`Tool "${name}" threw an error: ${err.message}`);
    }

    return {
      content: [{ type: 'text', text: JSON.stringify(result, null, 2) }],
    };
  });

  const transport = new StdioServerTransport();
  await server.connect(transport);
  process.stderr.write('[silcrow-mcp] Server ready\n');
}

function errorResponse(message) {
  return {
    isError: true,
    content: [{ type: 'text', text: message }],
  };
}

main().catch(err => {
  process.stderr.write(`[silcrow-mcp] Unhandled error: ${err.message}\n`);
  process.exit(1);
});
