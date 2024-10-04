#!/bin/bash

# Check if the input file exists
if [ ! -f "0.png" ]; then
    echo "File 0.png not found!"
    exit 1
fi

# Loop to create 63 clones
for i in {1..63}
do
    cp 0.png "$i.png"
    echo "Created $i.png"
done

echo "All 63 images cloned successfully."

