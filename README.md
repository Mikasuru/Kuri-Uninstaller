# Kuri Uninstaller
A simple utility to find and remove leftover files and registry entries from uninstalled programs on Windows. Built with Rust and the `iced` GUI toolkit.
![SS](https://github.com/Mikasuru/Kuri-Uninstaller/blob/main/Assets/ss.png?raw=true)
---

## Features

- **List Installed Programs:** Automatically scans and lists programs found in the Windows Registry.
- **Scan for Leftovers:** Searches common system locations (`%LOCALAPPDATA%`, `%APPDATA%`, etc.) and the registry for leftover files, folders, and keys associated with a selected program.
- **Selective Deletion:** Allows you to review all found items and choose which ones to delete.
- **Safe File Deletion:** Moves files and folders to the Recycle Bin instead of deleting them permanently.
- **Registry Backup:** Creates a `.txt` log of all registry keys that are about to be deleted, saving it to your `Documents\KuriUninstaller_Backups` folder.
- **Simple UI:** A clean and straightforward interface to guide you through the process.

## Getting Started
### Prerequisites

You need to have the Rust programming language and its toolchain installed. You can get it from the official site: [rustup.rs](https://rustup.rs/).

### Running the Application

1.  **Clone the repository:**
    ```sh
    git clone https://github.com/Mikasuru/Kuri-Uninstaller.git
    cd Kuri-Uninstaller
    ```

2.  **Run in an Administrator terminal:**
    Open your terminal (like Command Prompt or PowerShell) as an Administrator, and then run:
    ```sh
    cargo run
    ```

### Building for Release

1.  **Build the executable in an Administrator terminal:**
    ```sh
    cargo build --release
    ```

2.  **Find the executable:**
    The optimized executable will be located at `target/release/kuri_uninstaller.exe`.

3.  **Run the `.exe` file as an administrator.**
