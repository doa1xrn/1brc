#!/bin/bash

echo 3 | sudo tee /proc/sys/vm/drop_caches
time dd if=/mnt/ramdisk/m_100m.data of=/dev/null bs=1M status=progress
