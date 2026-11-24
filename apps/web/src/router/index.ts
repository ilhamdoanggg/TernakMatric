import {
  createMemoryHistory,
  createRouter,
  createWebHistory,
} from "vue-router";
import DashboardView from "@/views/DashboardView.vue";

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
    ],
  });
}
