import { renderToString } from "vue/server-renderer";
import { createApp } from "./main";

type SsrManifest = Record<string, string[]>;

interface SsrContext {
  modules?: Set<string>;
}

function assetBasename(filePath: string): string {
  const parts = filePath.split("/");
  return parts[parts.length - 1] ?? filePath;
}

function renderPreloadLink(file: string): string {
  if (file.endsWith(".js")) {
    return `<link rel="modulepreload" crossorigin href="${file}">`;
  }
  if (file.endsWith(".css")) {
    return `<link rel="stylesheet" href="${file}">`;
  }
  if (file.endsWith(".woff2")) {
    return `<link rel="preload" href="${file}" as="font" type="font/woff2" crossorigin>`;
  }
  return "";
}

function renderPreloadLinks(
  modules: Set<string> | undefined,
  manifest: SsrManifest | undefined,
): string {
  if (!modules || !manifest) {
    return "";
  }
  const seen = new Set<string>();
  let links = "";
  for (const id of modules) {
    const files: string[] | undefined = manifest[id];
    if (!files) continue;
    for (const file of files) {
      if (seen.has(file)) continue;
      seen.add(file);
      const filename = assetBasename(file);
      const depFiles: string[] | undefined = manifest[filename];
      if (depFiles) {
        for (const depFile of depFiles) {
          if (!seen.has(depFile)) {
            seen.add(depFile);
            links += renderPreloadLink(depFile);
          }
        }
      }
      links += renderPreloadLink(file);
    }
  }
  return links;
}

export async function render(url: string, manifest?: SsrManifest) {
  const { app, router } = createApp();

  await router.push(url);
  await router.isReady();

  const ctx: SsrContext = {};
  const html = await renderToString(app, ctx);
  const preloadLinks = renderPreloadLinks(ctx.modules, manifest);

  return { html, preloadLinks };
}
