#!/bin/bash

# Directory containing the PNG files
input_dir="./input_folder"
output_dir="./output_folder"

# Make sure the output directory exists
mkdir -p "$output_dir"

# Get a list of input files
input_files=("$input_dir"/*.png)
num_files=${#input_files[@]}

# Loop through the output quadrants (up to 64 quadrants in this example)
for output in $(seq 0 63); do
    # Compute the output's row and column in an 8x8 grid
    output_col=$((output % 8))
    output_row=$((output / 8))
    
    # Compute which input image this corresponds to
    input_col=$((output_col / 2))  # Each input image covers 2 output columns
    input_row=$((output_row / 2))  # Each input image covers 2 output rows
    file_index=$((input_row * 4 + input_col))  # 4 files wide (2x2 grid)

    # Get the corresponding input file
    input_file="${input_dir}/${file_index}.png"

    # Get the image dimensions
    width=$(identify -format "%w" "$input_file")
    height=$(identify -format "%h" "$input_file")

    # Calculate the width and height for each quadrant
    half_width=$((width / 2))
    half_height=$((height / 2))

    # Calculate the quadrant within the selected input file
    quadrant_col=$((output_col % 2))  # 0 for left, 1 for right
    quadrant_row=$((output_row % 2))  # 0 for top, 1 for bottom

    # Calculate cropping offsets
    x_offset=$((quadrant_col * half_width))
    y_offset=$((quadrant_row * half_height))

    # Crop the quadrant and save it with the sequential output number
    convert "$input_file" -crop "${half_width}x${half_height}+${x_offset}+${y_offset}" "$output_dir/${output}.png"

    echo "Processed $input_file quadrant $output"
done

echo "All PNG files processed!"

