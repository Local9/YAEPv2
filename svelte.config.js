import adapter from "@sveltejs/adapter-static";

/** @type {import("@sveltejs/kit").Config} */
const config = {
  kit: {
    adapter: adapter({
      fallback: "index.html",
      strict: false
    }),
    alias: {
      $models: "src/lib/models",
      $services: "src/lib/services"
    }
  }
};

export default config;
