import { createRootRoute, Outlet } from "@tanstack/react-router";
import styles from "../App.module.css";

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
  return (
    <>
      <header className={styles.header}>
        <span className={styles.wordmark}>STRUCTURE</span>
        <span className={styles.tagline}>mesocycle builder</span>
      </header>
      <main>
        <Outlet />
      </main>
    </>
  );
}
