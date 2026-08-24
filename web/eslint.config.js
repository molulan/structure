import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";

export default tseslint.config(
  { ignores: ["dist", "playwright-report", "test-results", "mesocycle_builder"] },
  {
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: 2022,
      globals: { ...globals.browser, ...globals.node },
    },
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      // A context module exports its own `use*` hook — `ApiProvider`/`useApi`,
      // `GridFrame`/`useGridScope`/`gridColumnCount` — so the seam and the way
      // to read it cannot drift apart. The rule objects to exactly that shape,
      // and what it protects is dev-time fast refresh, not correctness.
      "react-refresh/only-export-components": "off",
    },
  },
);
