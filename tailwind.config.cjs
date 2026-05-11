module.exports = {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        panel: "#121821",
        accent: "#5dd4ff",
        ink: "#e8f0f7"
      },
      boxShadow: {
        shell: "0 24px 80px rgba(0, 0, 0, 0.28)"
      }
    }
  },
  plugins: []
};
