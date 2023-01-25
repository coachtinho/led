# led

![GitHub Workflow Status (with branch)](https://img.shields.io/github/actions/workflow/status/coachtinho/led/build.yml?branch=master&style=for-the-badge)

Simple rust program to control MagicHome RGB controllers.

## Usage

```
USAGE:
    led [OPTIONS] --address <ADDRESS> <SUBCOMMAND>

OPTIONS:
    -a, --address <ADDRESS>    Adress of controller
    -h, --help                 Print help information
    -p, --port <PORT>          Port to access on the controller (default: 5577)
    -V, --version              Print version information

SUBCOMMANDS:
    ambient    Slow cycle
    blue       Blue static
    chaos      Red strobe
    cyan       Cyan static
    green      Green static
    help       Print this message or the help of the given subcommand(s)
    lime       Lime static
    off        Turn off device
    on         Turn on device
    orange     Orange static
    pink       Pink static
    purple     Purple static
    rainbow    Fast cycle
    red        Red static
    status     Get status of device
    white      White static
    yellow     Yellow static
```
