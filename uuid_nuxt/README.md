# Single File V4 UUID Generator

This project is a **Single File UUID v4 Generator** built with a **Rust (Actix)** backend and a **Nuxt 4** frontend, both embedded into a single executable file.

The frontend is automatically built and bundled into the backend, allowing you to **build, run, and deploy** the entire system as one file.

## Features

- Generates **UUID v4** via an API
- Saves generated UUIDs into downloadable files
- Auto-cleans generated files older than **10 minutes**
- Rust backend using **Actix**
- **Nuxt 4** frontend embedded into the backend
- Single-file deployment
- Frontend was designed using ClaudAI

## How It Works

- The `build.sh` script builds the Nuxt 4 frontend
- The frontend assets are embedded into the Rust binary
- The Actix server serves both the API and frontend
- When the server starts, a background thread runs to:
  - Automatically delete UUID files older than 10 minutes

## Build

Make sure you have **Rust**, **Node.js**, and **pnpm/npm** installed.

```bash
chmod +x build.sh
./build.sh
