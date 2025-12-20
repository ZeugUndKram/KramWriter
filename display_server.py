#!/usr/bin/env python3
"""
Display Server für Sharp Memory Display
Kommuniziert über Unix Socket mit C-Programmen
"""
import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay
import socket
import os
import struct
import sys

# Display Setup
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Socket path
SOCKET_PATH = "/tmp/display_server.sock"

# Cache für Bilder
image_cache = {}

def clear_display():
    """Leert das Display"""
    display.fill(1)
    display.show()

def draw_text(text, x, y, font_size=20):
    """Zeigt Text an"""
    image = Image.new("1", (display.width, display.height))
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, display.width, display.height), outline=1, fill=1)
    
    try:
        font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", font_size)
    except:
        font = ImageFont.load_default()
    
    draw.text((x, y), text, font=font, fill=0)
    display.image(image)
    display.show()

def draw_image(path, x=None, y=None):
    """Zeigt Bild an (zentriert wenn x,y=None)"""
    # Cache check
    if path in image_cache:
        logo = image_cache[path]
    else:
        logo = Image.open(path)
        if logo.mode != "1":
            logo = logo.convert("L")
            logo = logo.point(lambda p: 0 if p < 128 else 255, '1')
        image_cache[path] = logo
    
    image = Image.new("1", (display.width, display.height))
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, display.width, display.height), outline=1, fill=1)
    
    if x is None:
        x = (display.width - logo.size[0]) // 2
    if y is None:
        y = (display.height - logo.size[1]) // 2
    
    image.paste(logo, (x, y))
    display.image(image)
    display.show()

def draw_rect(x, y, w, h, fill=False):
    """Zeichnet Rechteck"""
    image = Image.new("1", (display.width, display.height))
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, display.width, display.height), outline=1, fill=1)
    draw.rectangle((x, y, x+w, y+h), outline=0, fill=0 if fill else 1)
    display.image(image)
    display.show()

def draw_raw_buffer(data):
    """Zeigt raw 1-bit Buffer (400x240 bits = 12000 bytes)"""
    if len(data) != 12000:
        return False
    
    image = Image.frombytes("1", (400, 240), data)
    display.image(image)
    display.show()
    return True

def handle_command(command, data):
    """Verarbeitet Kommandos"""
    try:
        if command == b"CLEAR":
            clear_display()
            return b"OK"
        
        elif command == b"TEXT":
            # Format: x(4) y(4) size(4) text(rest)
            x = struct.unpack("i", data[0:4])[0]
            y = struct.unpack("i", data[4:8])[0]
            size = struct.unpack("i", data[8:12])[0]
            text = data[12:].decode('utf-8')
            draw_text(text, x, y, size)
            return b"OK"
        
        elif command == b"IMAGE":
            # Format: x(4) y(4) path(rest)
            x = struct.unpack("i", data[0:4])[0]
            y = struct.unpack("i", data[4:8])[0]
            path = data[8:].decode('utf-8')
            
            # -1 bedeutet zentriert
            if x == -1:
                x = None
            if y == -1:
                y = None
            
            draw_image(path, x, y)
            return b"OK"
        
        elif command == b"RECT":
            # Format: x(4) y(4) w(4) h(4) fill(1)
            x = struct.unpack("i", data[0:4])[0]
            y = struct.unpack("i", data[4:8])[0]
            w = struct.unpack("i", data[8:12])[0]
            h = struct.unpack("i", data[12:16])[0]
            fill = data[16] == 1
            draw_rect(x, y, w, h, fill)
            return b"OK"
        
        elif command == b"RAWBUF":
            # Raw 12000 byte buffer
            if draw_raw_buffer(data):
                return b"OK"
            return b"ERROR"
        
        else:
            return b"UNKNOWN"
    
    except Exception as e:
        print(f"Error: {e}")
        return b"ERROR"

def main():
    # Remove old socket
    try:
        os.unlink(SOCKET_PATH)
    except OSError:
        pass
    
    # Create socket
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.bind(SOCKET_PATH)
    os.chmod(SOCKET_PATH, 0o666)  # Allow all users
    sock.listen(1)
    
    print(f"Display Server läuft auf {SOCKET_PATH}")
    clear_display()
    
    try:
        while True:
            conn, _ = sock.accept()
            try:
                # Empfange Command (8 bytes)
                cmd = conn.recv(8)
                if not cmd:
                    continue
                
                # Empfange Data Length (4 bytes)
                length_data = conn.recv(4)
                if not length_data:
                    continue
                
                data_len = struct.unpack("i", length_data)[0]
                
                # Empfange Data
                data = b""
                while len(data) < data_len:
                    chunk = conn.recv(min(4096, data_len - len(data)))
                    if not chunk:
                        break
                    data += chunk
                
                # Verarbeite
                response = handle_command(cmd, data)
                conn.sendall(response)
            
            finally:
                conn.close()
    
    except KeyboardInterrupt:
        print("\nShutdown...")
    finally:
        sock.close()
        os.unlink(SOCKET_PATH)

if __name__ == "__main__":
    main()