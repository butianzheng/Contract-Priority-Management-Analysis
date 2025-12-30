#!/usr/bin/env python3
from PIL import Image, ImageDraw, ImageFont
import os

# Create icons directory if it doesn't exist
os.makedirs('src-tauri/icons', exist_ok=True)

# Create a simple icon with letter "D"
def create_icon(size, filename):
    # Create image with blue background
    img = Image.new('RGB', (size, size), color='#4A90E2')
    draw = ImageDraw.Draw(img)

    # Try to use a system font, fallback to default
    try:
        font_size = int(size * 0.6)
        font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", font_size)
    except:
        font = ImageFont.load_default()

    # Draw letter "D"
    text = "D"
    # Get text bounding box
    bbox = draw.textbbox((0, 0), text, font=font)
    text_width = bbox[2] - bbox[0]
    text_height = bbox[3] - bbox[1]

    # Center the text
    x = (size - text_width) // 2 - bbox[0]
    y = (size - text_height) // 2 - bbox[1]

    draw.text((x, y), text, fill='white', font=font)

    # Save the image
    img.save(filename, 'PNG')
    print(f"Created {filename}")

# Create required icons
create_icon(32, 'src-tauri/icons/32x32.png')
create_icon(128, 'src-tauri/icons/128x128.png')
create_icon(256, 'src-tauri/icons/128x128@2x.png')
create_icon(512, 'src-tauri/icons/icon.png')

print("\nAll icons created successfully!")
print("Note: .icns and .ico files need to be generated separately or you can use the tauri icon command with a PNG source.")
