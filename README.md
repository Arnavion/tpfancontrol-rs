tpfancontrol-rs is a Linux TUI clone of troubadix's TPFanControl. It uses the sysfs interface exposed by the `thinkpad_acpi` kernel module to allow you to monitor and control the temperature and fan speeds of your Thinkpad.


### Installation

- To install from the Github repository directly, run:

	```sh
	cargo install --git 'https://github.com/Arnavion/tpfancontrol-rs'
	```

	This will compile the `tpfancontrol` binary and put it in `~/.cargo/bin/`

- Alternatively, clone this repo, then run:

	```sh
	cargo build --release
	```

	This will compile the `tpfancontrol` binary and put it in `./target/release/`


### Usage

1. Ensure the `thinkpad_acpi` module has been loaded by the kernel.

	```sh
	$ lsmod | grep '^thinkpad_acpi'

	thinkpad_acpi    122880 0
	```

1. Enable the `fan_control=1` option for the module and reload the module.

	```sh
	$ echo 'options thinkpad_acpi fan_control=1' | sudo tee /etc/modprobe.d/98-thinkpad_acpi.conf

	options thinkpad_acpi fan_control=1

	$ sudo modprobe -r thinkpad_acpi

	$ sudo modprobe thinkpad_acpi

	$ lsmod | grep '^thinkpad_acpi'

	thinkpad_acpi    122880 0
	```

	If unloading the module fails, it's easiest and safest to just reboot.

1. Copy the [`config.toml.example`](./config.toml.example) file in this repository to `/etc/tpfancontrol/config.toml` and edit it to match the sensors of your Thinkpad. This file contains custom names for the temperature sensors and a mapping of temperature to fan level.

	To discover the sensor numbers, find the hwmon node and enumerate the `temp*_input` files inside it. For example:

	```sh
	$ grep -H 'thinkpad' /sys/class/hwmon/hwmon*/name

	/sys/class/hwmon/hwmon2/name:thinkpad

	$ ls -1 /sys/class/hwmon/hwmon2/temp*_input

	/sys/class/hwmon/hwmon2/temp1_input
	/sys/class/hwmon/hwmon2/temp2_input
	/sys/class/hwmon/hwmon2/temp3_input
	...
	```

	This means you have sensors numbered 1, 2, 3, and so on. Add them to the config file so `tpfancontrol` can monitor them. The names for the sensors are completely arbitrary and only used for display purposes, so you can name them whatever you like.

	(Note: It looks like some Thinkpads don't report any temperature sensors via the `thinkpad_acpi` module. See [this issue](https://github.com/Arnavion/tpfancontrol-rs/issues/3) for discussion.)

1. Run the `tpfancontrol` binary that you compiled above.

	To allow the program to modify the fan speed, it must be run as root, such as with `sudo`. If the program is run as an unprivileged user, it cannot adjust the fan speed because the hwmon interface is only writable by root. In this case the program will not adjust the fan speed based on the temperature, and the manual controls for changing the fan speed will also be locked.


### Notes

- SMART mode does not have hysteresis. The fan speed will fluctuate when the temperature is near the boundary between two mappings. As a workaround, change your mapping so that the stable temperature of your Thinkpad is not near a boundary.
