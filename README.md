# SoulEvBatMonitor

This is a simple program that shows the battery cell voltages of the Kia Soul EV over time. 
This can be helpful to find out if a cell is out of balance or degrading.

## Screenshot
[![Screenshot](https://raw.githubusercontent.com/saturn4er/SoulEvBatMonitor/master/images/main_screen.jpg)](https://raw.githubusercontent.com/saturn4er/SoulEvBatMonitor/master/images/main_screen.jpg)

## Build from source
You need to have the following installed:
- Rust (https://www.rust-lang.org/tools/install)
- Node.js (https://nodejs.org/en/download/)

Than you should install npm dependencies with:
```
yarn install
```

Then you can build the program with:
```
cd src-tauri && cargo build --release
```