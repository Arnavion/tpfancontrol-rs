tpfancontrol-rs is a Linux TUI clone of troubadix's TPFanControl. It uses the sysfs interface exposed by the thinkpad-acpi kernel module to allow you to monitor and control the temperature and fan speeds of your Thinkpad.

Build with `cargo build` and run with `cargo run`

The program reads a config file `/etc/tpfancontrol/config.toml` for the names of the temperature sensors and for the temperature-to-fan-level mapping. There is an example `config.toml.example` in this repository.

If run without superuser rights, the program does not have write access to the kernel interface, so the controls for modifying the fan speed will be locked.


### Requirements

- thinkpad-acpi in your kernel
- fan_control=1 module parameter for thinkpad-acpi


### Notes

- SMART mode does not have hysteresis. The fan speed will fluctuate when the temperature is near the boundary between two mappings.
