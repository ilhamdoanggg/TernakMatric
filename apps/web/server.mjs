import fs from "node:fs";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const isDev = process.argv.includes("--dev");
const port = Number(process.env.PORT ?? 3000);

const MIME_TYPES = {
  ".js": "application/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".ico": "image/x-icon",
  ".woff2": "font/woff2",
};

function serveStatic(reqUrl, clientDistDir, res) {
  const cleanPath = (reqUrl ?? "/").split("?")[0];
  if (cleanPath === "/" || cleanPath === "/index.html") {
    return false;
  }
  const resolved = path.resolve(clientDistDir, `.${cleanPath}`);
  if (!resolved.startsWith(clientDistDir) || !fs.existsSync(resolved)) {
    return false;
  }
  const stat = fs.statSync(resolved);
  if (!stat.isFile()) {
    return false;
  }
  const ext = path.extname(resolved);
  res.writeHead(200, {
    "Content-Type": MIME_TYPES[ext] ?? "application/octet-stream",
    "Cache-Control": cleanPath.startsWith("/assets/")
      ? "public, max-age=31536000, immutable"
      : "public, max-age=300",
  });
  fs.createReadStream(resolved).pipe(res);
  return true;
}

async function createSsrHandler() {
  if (isDev) {
    const { createServer: createViteServer } = await import("vite");
    const vite = await createViteServer({
      root: __dirname,
      server: { middlewareMode: true },
      appType: "custom",
    });

    return async (req, res) => {
      vite.middlewares(req, res, async () => {
        const url = req.url ?? "/";
        try {
          const rawHtml = fs.readFileSync(
            path.resolve(__dirname, "index.html"),
            "utf-8",
          );
          const template = await vite.transformIndexHtml(url, rawHtml);
          const { render } = await vite.ssrLoadModule("/src/entry-server.ts");
          const { html: appHtml, preloadLinks } = await render(url);

          const finalHtml = template
            .replace("<!--preload-links-->", preloadLinks ?? "")
            .replace("<!--ssr-outlet-->", appHtml);

          res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
          res.end(finalHtml);
        } catch (err) {
          vite.ssrFixStacktrace(err);
          console.error(err);
          res.writeHead(500, { "Content-Type": "text/plain; charset=utf-8" });
          res.end(err instanceof Error ? err.stack : "Internal Server Error");
        }
      });
    };
  }

  const clientDistDir = path.resolve(__dirname, "dist/client");
  const template = fs.readFileSync(
    path.join(clientDistDir, "index.html"),
    "utf-8",
  );
  const manifestPath = path.join(clientDistDir, ".vite/ssr-manifest.json");
  const manifest = fs.existsSync(manifestPath)
    ? JSON.parse(fs.readFileSync(manifestPath, "utf-8"))
    : {};
  const { render } = await import(
    path.resolve(__dirname, "dist/server/entry-server.js")
  );

  return async (req, res) => {
    const url = req.url ?? "/";
    if (serveStatic(url, clientDistDir, res)) {
      return;
    }
    try {
      const { html: appHtml, preloadLinks } = await render(url, manifest);
      const finalHtml = template
        .replace("<!--preload-links-->", preloadLinks ?? "")
        .replace("<!--ssr-outlet-->", appHtml);

      res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
      res.end(finalHtml);
    } catch (err) {
      console.error(err);
      res.writeHead(500, { "Content-Type": "text/plain; charset=utf-8" });
      res.end("Internal Server Error");
    }
  };
}

const handler = await createSsrHandler();
const server = http.createServer(handler);
server.listen(port, () => {
  console.log(
    `TernakMatric Vue SSR server (${isDev ? "dev" : "prod"}) running at http://localhost:${port}`,
  );
});
