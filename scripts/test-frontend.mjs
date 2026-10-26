import { createServer } from 'vite';

const server = await createServer({
    configFile: false,
    server: { middlewareMode: true, hmr: false, ws: false, watch: null },
    optimizeDeps: { noDiscovery: true, include: [] },
    appType: 'custom',
});

try {
    await server.ssrLoadModule('/tests/portfolioReports.test.ts');
} finally {
    await server.close();
}
