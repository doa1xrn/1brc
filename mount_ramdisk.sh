#!/bin/bash

if [ ! -d "/mnt/ramdisk" ]; then
    sudo sudo mkdir -p "/mnt/ramdisk"
fi

sudo mount -t tmpfs -o size=2G ramdisk /mnt/ramdisk
