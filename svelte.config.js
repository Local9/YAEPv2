/** @type {import("@sveltejs/kit").Config} */
const config = {
  kit: {
    alias: {
      $models: "src/lib/models",
      $services: "src/lib/services"
    }
  }
};

export default config;
