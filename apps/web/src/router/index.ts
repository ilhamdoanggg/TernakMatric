import {
  createMemoryHistory,
  createRouter,
  createWebHistory,
} from "vue-router";
import DashboardView from "@/views/DashboardView.vue";
import LogsDashboardView from "@/views/LogsDashboardView.vue";

export function createAppRouter() {
  const history = import.meta.env.SSR
    ? createMemoryHistory()
    : createWebHistory();

  return createRouter({
    history,
    routes: [
      {
        path: "/",
        name: "dashboard",
        component: DashboardView,
      },
      {
        path: "/logs",
        name: "logs",
        component: LogsDashboardView,
      },
    ],
  });
}
