#!/bin/bash

# Original idea from https://github.com/iye/lightsOn/blob/master/lightsOn.sh

# Detect screensaver been used (xscreensaver, kscreensaver or none)
    screensaver=$(pgrep -l xscreensaver | grep -wc xscreensaver)
if [ $screensaver -ge 1 ]; then
    screensaver=xscreensaver
else
    screensaver=$(pgrep -l kscreensaver | grep -wc kscreensaver)
    if [ $screensaver -ge 1 ]; then
        screensaver=kscreensaver
    else
        screensaver=None
    fi
fi

# reset inactivity time counter so screensaver is not started
if [ "$screensaver" == "xscreensaver" ]; then
    xscreensaver-command -deactivate > /dev/null
elif [ "$screensaver" == "kscreensaver" ]; then
    simulate=true
fi

# SimulateUserActivity disables auto screen dimming in KDE4
if [ "$KDE_FULL_SESSION" == true ]; then
    simulate=true
fi

if [ "$simulate" == true ]; then
    qdbus org.freedesktop.ScreenSaver /ScreenSaver SimulateUserActivity > /dev/null
fi

# Check if DPMS is on. If it is, deactivate and reactivate again. If it is not, do nothing.
dpmsStatus=$(xset -q | grep -ce 'DPMS is Enabled')
if [ $dpmsStatus == 1 ]; then
    xset -dpms
    xset +dpms
fi

# Move the mouse http://xkcd.com/196/
#xte 'mousermove 1 1'
