# Zin

Zin is an init system (PID 1) developed for the AuruOS distribution, other distributions and licensed only by MIT.

The development is based on the Unix philosophy:

- **"Make every program do one thing well." **

- Doug McIlroy

- **"Write programs that do one thing and do it well. Write programs for collaboration." **

- Peter H. Salus, generalising the philosophy of Unix

`Zin ` tends to be indestructible, something that even in the case of most errors will load you into the system. If not, then go to the save mode with all the tools installed. Also, when it breaks down, Zin will try to restore the configuration according to the past backup that it made at startup

Editions:

Zin has several revisions: workstation,server,another for desktop: focussed on fast loading of de and system components, server: minimum init for loading speed and security

# Development

This repository contains the source code of the executable file itself `init` and `zinctl` to manage the initialisation system.

At the moment, the binary init file itself is a priority for development.

# Usage

All operations are registered and start with `*` and look like `<Status> <Operation> <Data>`, it has three colours:

- Red (`\x1b[31m`) - This means that the mission-critical operation failed, and you are likely to go into rescue mode.

- Yellow (`\x1b[33m`) - This means that the operation specified by the user in the configuration or not critical failed, the system will work, but it's better to pay attention to it.

- Green (`\x1b[32m`) - Everything is fine.

At the moment, the repository is not fully developed and all the code is in a zip file.
