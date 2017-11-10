tpfancontrol-rs is a Linux TUI clone of troubadix's TPFanControl. It uses the sysfs interface exposed by the thinkpad-acpi kernel module to allow you to monitor and control the temperature and fan speeds of your Thinkpad.

Build with `cargo build` and run with `cargo run`

If run without superuser rights, the program does not have write access to the kernel interface, so the controls for modifying the fan speed will be locked.


### Requirements

- thinkpad-acpi in your kernel
- fan_control=1 module parameter for thinkpad-acpi


### Notes

- The temperature-to-fan speed mapping for SMART mode is hard-coded in the source, and requires a rebuild to change.
- SMART mode does not have hysteresis. The fan speed will fluctuate when the temperature is near the boundary between two mappings.
