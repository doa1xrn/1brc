#!/bin/bash

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <source directory>"
    exit 1
fi

src="$1"

if [ ! -d "$src" ]; then
    echo "'$src' is not a directory"
    exit 1
fi

cp "$src"/m_*.data /mnt/ramdisk
