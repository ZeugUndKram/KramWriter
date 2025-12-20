cat > display_server.py << 'EOF'
#!/usr/bin/env python3
import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay
import socket
import os
import struct
import sys

spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

SOCKET_PATH = "/tmp/display_server.sock"
image_cache = {}

def clear_display():
    print("  -> Clearing display")
    display.fill(1)
    display.show()

def draw_text(text, x, y, font_size=20):
    print(f"  -> Drawing text: '{text}' at ({x}, {y})")
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
    print(f"  -> Drawing image: {path} at ({x}, {y})")
    
    if not os.path.exists(path):
        print(f"  -> ERROR: File not found: {path}")
        return False
    
    if path in image_cache:
        logo = image_cache[path]
    else:
        logo = Image.open(path)
        if logo.mode != "1":
            logo = logo.convert("L")
            logo = logo.point(lambda p: 0 if p < 128 else 255, '1')
        image_cache[path] = logo
        print(f"  -> Loaded and cached: {logo.size}")
    
    image = Image.new("1", (display.width, display.height))
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, display.width, display.height), outline=1, fill=1)
    
    if x is None or x == -1:
        x = (display.width - logo.size[0]) // 2
    if y is None or y == -1:
        y = (display.height - logo.size[1]) // 2
    
    print(f"  -> Pasting at ({x}, {y})")
    image.paste(logo, (x, y))
    display.image(image)
    display.show()
    print("  -> Display updated!")
    return True

def draw_rect(x, y, w, h, fill=False):
    print(f"  -> Drawing rect: ({x}, {y}, {w}, {h}), fill={fill}")
    image = Image.new("1", (display.width, display.height))
    draw = ImageDraw.Draw(image)
    draw.rectangle((0, 0, display.width, display.height), outline=1, fill=1)
    draw.rectangle((x, y, x+w, y+h), outline=0, fill=0 if fill else 1)
    display.image(image)
    display.show()

def draw_raw_buffer(data):
    print(f"  -> Drawing raw buffer: {len(data)} bytes")
    if len(data) != 12000:
        print(f"  -> ERROR: Expected 12000 bytes, got {len(data)}")
        return False
    
    # PIL erwartet die Bits in einem bestimmten Format
    # 400x240 = 96000 bits = 12000 bytes
    image = Image.frombytes("1", (400, 240), bytes(data))
    display.image(image)
    display.show()
    print("  -> Raw buffer displayed!")
    return True

def handle_command(command, data):
    print(f"Command: {command}, Data length: {len(data)}")
    
    try:
        if command == b"CLEAR\x00\x00\x00":
            clear_display()
            return b"OK"
        
        elif command == b"TEXT\x00\x00\x00\x00":
            x = struct.unpack("i", data[0:4])[0]
            y = struct.unpack("i", data[4:8])[0]
            size = struct.unpack("i", data[8:12])[0]
            text = data[12:].decode('utf-8')
            draw_text(text, x, y, size)
            return b"OK"
        
        elif command == b"IMAGE\x00\x00\x00":
            x = struct.unpack("i", data[0:4])[0]
            y = struct.unpack("i", data[4:8])[0]
            path = data[8:].decode('utf-8')
            print(f"  -> Params: x={x}, y={y}, path={path}")
            
            if x == -1:
                x = None
            if y == -1:
                y = None
            
            if draw_image(path, x, y):
                return b"OK"
            return b"ERROR"
        
        elif command == b"RECT\x00\x00\x00\x00":
            x = struct.unpack("i", data[0:4])[0]
            y = struct.unpack("i", data[4:8])[0]
            w = struct.unpack("i", data[8:12])[0]
            h = struct.unpack("i", data[12:16])[0]
            fill = data[16] == 1
            draw_rect(x, y, w, h, fill)
            return b"OK"
        
        elif command == b"RAWBUF\x00\x00":
            if draw_raw_buffer(data):
                return b"OK"
            return b"ERROR"
        
        else:
            print(f"  -> UNKNOWN command: {command}")
            return b"UNKNOWN"
    
    except Exception as e:
        print(f"  -> ERROR: {e}")
        import traceback
        traceback.print_exc()
        return b"ERROR"

def main():
    try:
        os.unlink(SOCKET_PATH)
    except OSError:
        pass
    
    sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    sock.bind(SOCKET_PATH)
    os.chmod(SOCKET_PATH, 0o666)
    sock.listen(1)
    
    print(f"Display Server läuft auf {SOCKET_PATH}")
    clear_display()
    
    try:
        while True:
            print("\nWaiting for connection...")
            conn, _ = sock.accept()
            print("Client connected!")
            
            try:
                cmd = conn.recv(8)
                if not cmd:
                    print("No command received")
                    continue
                
                print(f"Received command: {cmd}")
                
                length_data = conn.recv(4)
                if not length_data:
                    print("No length received")
                    continue
                
                data_len = struct.unpack("i", length_data)[0]
                print(f"Data length: {data_len}")
                
                data = b""
                while len(data) < data_len:
                    chunk = conn.recv(min(4096, data_len - len(data)))
                    if not chunk:
                        break
                    data += chunk
                
                print(f"Received {len(data)} bytes of data")
                
                response = handle_command(cmd, data)
                conn.sendall(response)
                print(f"Sent response: {response}")
            
            finally:
                conn.close()
                print("Connection closed")
    
    except KeyboardInterrupt:
        print("\nShutdown...")
    finally:
        sock.close()
        os.unlink(SOCKET_PATH)

if __name__ == "__main__":
    main()
EOF