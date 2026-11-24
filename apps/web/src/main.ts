import { createSSRApp } from "vue";
import App from "./App.vue";
import { createAppRouter } from "./router";
import "./style.css";

export function createApp() {
  const app = createSSRApp(App);
  const router = createAppRouter();
  app.use(router);
  return { app, router };
}
