Client for a Pi Zero W with a Camera to take pictures simultaneously
with other clients and upload them to a server.

# Pi Zero W setup

## Update
`sudo apt update`  
`sudo apt full-upgrade`

## Install packages
`sudo apt-get update`  
`sudo apt install -y python3-picamera2 --no-install-recommends`  
`sudo apt-get install python3-pip` ??

## Disable automatic NTP
`sudo systemctl stop systemd-timesyncd`  
`sudo systemctl disable systemd-timesyncd`  
`sudo timedatectl set-ntp false`  
Verify with `timedatectl status`

disable ntp
libcamera  
ntpdate  
...

# Build

Needs armv6 dependencies (python, gcc, etc.), use devcontainer - everything set up.

# Basic usage

Client receives messages in MQTT topics - for each topic it subscribes to the "global" topic
and the "individual" topic, where it receives the corresponding commands. After processing,
the client replies on an individual answer topic with the result.

Topic example:
- `global` - Global receive topic
- `global/id` - Individual receive topic
- `global/answer/id` - Individual answer topic

The client reports errors to the individual error topic, if possible.

# Structure

## updater

Contains auto-updater:

- downloads executable from the server
- replaces executable
- restarts

Restart happens in main.rs as it needs to run in the main thread.

## utils

Misc stuff