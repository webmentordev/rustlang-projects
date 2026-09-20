# steam-dl

A minimal Axum web app that drives `steamcmd` to download the **Windows** build
of a Steam app onto a Linux machine, with a live progress bar over SSE.
I personally use it on my home server (with UPS backup) to download games when I'm not home. Easy to access on the local IP ❤️

## Prerequisites
Download steamcmd for Linux and extract it, then build the Rust project, copy the built file, and paste it inside the extracted steamcmd folder. Then run it. ALL GOOD.
```
cargo build --release
curl -sqL "https://client-update.steamstatic.com/installer/steamcmd_linux.tar.gz" | tar zxvf -
```