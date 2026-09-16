// Synthetic child that never acknowledges automatic startup, or exits early.
if (process.argv[2] === "never-ready") setInterval(() => {}, 1000);
