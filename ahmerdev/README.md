# Simple setup, build & run project

## Install NPM
First you need to go to UI folder and run this command

```
npm install
```

## Create ENV
Create .env file from the content of .env.example and change the value.

## Build the project for release
Just run this command so UI (Nuxt) & Rust project will be build for release

```
$ chmod +x build.sh
$ ./build.sh
```

# Create image using dockerfile, run using docker compose
Please do the above steps first then build your own docker images using these commands and run the container.

```
# Build docker image, change the name if you like
$ docker build -t my-rust-server .

# Test command
$ docker run --rm --env-file .env -p 8787:8787 -v ./profile_db:/app my-rust-server

# Run in production using docker compose
$ docker compose up -d
```