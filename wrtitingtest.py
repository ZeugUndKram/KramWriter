"""
Responsive Text Editor for Sharp Memory Display
Features: Line wrapping, scrolling, fast rendering
"""

import board
import busio
import digitalio
import time
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay
import os

# ============== CONFIGURATION ==============
# Display configuration - uncomment your display
# spi = busio.SPI(board.SCK, MOSI=board.MOSI)
# scs = digitalio.DigitalInOut(board.D6)

# For testing without hardware, use a simulated display
try:
    spi = busio.SPI(board.SCK, MOSI=board.MOSI)
    scs = digitalio.DigitalInOut(board.D6)
    display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 144, 168)
    HAS_DISPLAY = True
except (RuntimeError, ImportError):
    print("No display hardware detected - running in simulation mode")
    HAS_DISPLAY = False
    # Simulated display dimensions
    class SimulatedDisplay:
        width = 144
        height = 168
        def fill(self, *args): pass
        def show(self): pass
        def image(self, img): pass
    display = SimulatedDisplay()

# Display parameters
WIDTH = display.width
HEIGHT = display.height
CHAR_WIDTH = 6  # Approximate width of a character (will be measured)
CHAR_HEIGHT = 8  # Approximate height (will be measured)
MAX_LINES = 20  # Maximum lines to keep in buffer for scrolling

# Colors
BLACK = 0
WHITE = 255

# Font configuration
FONT_PATH = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
FONT_SIZE = 8  # Smaller font for more text

# ============== TEXT BUFFER CLASS ==============
class TextBuffer:
    def __init__(self, display_width, char_width, char_height):
        self.display_width = display_width
        self.char_width = char_width
        self.char_height = char_height
        self.lines = [""]
        self.current_line = 0
        self.current_pos = 0
        self.scroll_offset = 0
        self.max_visible_lines = HEIGHT // char_height
        self.max_chars_per_line = display_width // char_width
        
    def add_char(self, char):
        """Add a character at current position with word wrapping"""
        if char == '\n':
            self._insert_newline()
            return
            
        if self.current_pos >= len(self.lines[self.current_line]):
            # Append to end of line
            self.lines[self.current_line] += char
            self.current_pos += 1
            
            # Check if we need to wrap
            if len(self.lines[self.current_line]) * self.char_width > self.display_width - 10:
                self._wrap_line()
        else:
            # Insert in the middle
            line = self.lines[self.current_line]
            self.lines[self.current_line] = line[:self.current_pos] + char + line[self.current_pos:]
            self.current_pos += 1
            
        # Auto-scroll if needed
        self._auto_scroll()
        
    def backspace(self):
        """Handle backspace at current position"""
        if self.current_pos > 0:
            # Remove character before cursor
            line = self.lines[self.current_line]
            self.lines[self.current_line] = line[:self.current_pos-1] + line[self.current_pos:]
            self.current_pos -= 1
        elif self.current_line > 0:
            # Join with previous line
            prev_line = self.lines.pop(self.current_line)
            self.current_line -= 1
            self.current_pos = len(self.lines[self.current_line])
            self.lines[self.current_line] += prev_line
            
        self._auto_scroll()
        
    def delete(self):
        """Handle delete at current position"""
        if self.current_pos < len(self.lines[self.current_line]):
            line = self.lines[self.current_line]
            self.lines[self.current_line] = line[:self.current_pos] + line[self.current_pos+1:]
        elif self.current_line < len(self.lines) - 1:
            # Merge with next line
            next_line = self.lines.pop(self.current_line + 1)
            self.lines[self.current_line] += next_line
            
        self._auto_scroll()
        
    def move_cursor(self, dx, dy):
        """Move cursor by dx characters and dy lines"""
        new_line = max(0, min(len(self.lines) - 1, self.current_line + dy))
        
        if new_line != self.current_line:
            self.current_line = new_line
            self.current_pos = min(self.current_pos, len(self.lines[self.current_line]))
            self._auto_scroll()
            return
            
        self.current_pos = max(0, min(len(self.lines[self.current_line]), self.current_pos + dx))
        
    def _insert_newline(self):
        """Insert a newline and split the line if needed"""
        current = self.lines[self.current_line]
        before = current[:self.current_pos]
        after = current[self.current_pos:]
        
        self.lines[self.current_line] = before
        self.lines.insert(self.current_line + 1, after)
        self.current_line += 1
        self.current_pos = 0
        
        # Limit total lines for memory efficiency
        if len(self.lines) > MAX_LINES:
            self.lines = self.lines[-MAX_LINES:]
            self.current_line = min(self.current_line, len(self.lines) - 1)
            self.scroll_offset = max(0, self.scroll_offset - 1)
            
        self._auto_scroll()
        
    def _wrap_line(self):
        """Wrap the current line to next line"""
        line = self.lines[self.current_line]
        
        # Find the last space before overflow
        wrap_pos = len(line)
        for i in range(len(line) - 1, -1, -1):
            if line[i] == ' ' and i * self.char_width < self.display_width - 10:
                wrap_pos = i + 1  # Include the space in the first line
                break
                
        # If no space found, wrap at max chars
        if wrap_pos == len(line):
            wrap_pos = self.max_chars_per_line
            
        # Split the line
        self.lines[self.current_line] = line[:wrap_pos].rstrip()
        new_line = line[wrap_pos:].lstrip()
        
        # Insert new line
        self.lines.insert(self.current_line + 1, new_line)
        self.current_line += 1
        self.current_pos = len(new_line)
        
    def _auto_scroll(self):
        """Auto-scroll to keep cursor visible"""
        # Calculate visible line range
        visible_start = self.scroll_offset
        visible_end = visible_start + self.max_visible_lines
        
        # Adjust scroll if cursor is outside visible area
        if self.current_line < visible_start:
            self.scroll_offset = self.current_line
        elif self.current_line >= visible_end:
            self.scroll_offset = self.current_line - self.max_visible_lines + 1
            
    def get_visible_text(self):
        """Get text to display (with cursor indicator)"""
        visible_lines = []
        start = self.scroll_offset
        end = min(start + self.max_visible_lines, len(self.lines))
        
        for i in range(start, end):
            line = self.lines[i]
            if i == self.current_line:
                # Insert cursor indicator
                if self.current_pos >= len(line):
                    line_with_cursor = line + "█"
                else:
                    line_with_cursor = line[:self.current_pos] + "█" + line[self.current_pos:]
                visible_lines.append(line_with_cursor)
            else:
                visible_lines.append(line)
                
        return visible_lines
        
    def save_to_file(self, filename="text_output.txt"):
        """Save buffer to file"""
        try:
            with open(filename, "w") as f:
                f.write("\n".join(self.lines))
            return True
        except Exception as e:
            print(f"Error saving: {e}")
            return False
            
    def load_from_file(self, filename="text_output.txt"):
        """Load buffer from file"""
        try:
            if os.path.exists(filename):
                with open(filename, "r") as f:
                    self.lines = [line.rstrip('\n') for line in f.readlines()]
                self.current_line = min(self.current_line, len(self.lines) - 1)
                self.current_pos = min(self.current_pos, len(self.lines[self.current_line]))
                self.scroll_offset = 0
                return True
        except Exception as e:
            print(f"Error loading: {e}")
        return False

# ============== RENDERER CLASS ==============
class DisplayRenderer:
    def __init__(self, display, font_path, font_size):
        self.display = display
        self.image = Image.new("1", (WIDTH, HEIGHT))
        self.draw = ImageDraw.Draw(self.image)
        
        # Try to load font, fall back to default
        try:
            self.font = ImageFont.truetype(font_path, font_size)
            # Measure actual character dimensions
            bbox = self.font.getbbox("M")
            self.char_width = bbox[2] - bbox[0]
            self.char_height = bbox[3] - bbox[1] + 2  # Add some spacing
        except:
            print("Using default font")
            self.font = ImageFont.load_default()
            self.char_width = 8
            self.char_height = 10
            
        self.last_render = None
        
    def render_text(self, text_buffer):
        """Render text buffer to display efficiently"""
        # Clear display
        self.draw.rectangle((0, 0, WIDTH, HEIGHT), fill=WHITE)
        
        # Get visible text
        visible_lines = text_buffer.get_visible_text()
        
        # Draw each line
        y = 0
        for line in visible_lines:
            self.draw.text((2, y), line, font=self.font, fill=BLACK)
            y += self.char_height
            if y >= HEIGHT:
                break
                
        # Draw status bar
        self.draw.rectangle((0, HEIGHT - self.char_height, WIDTH, HEIGHT), fill=BLACK)
        status = f"Ln:{text_buffer.current_line+1} Col:{text_buffer.current_pos+1}"
        self.draw.text((2, HEIGHT - self.char_height), status, font=self.font, fill=WHITE)
        
        # Only update display if image changed
        current_image = self.image.tobytes()
        if current_image != self.last_render:
            if HAS_DISPLAY:
                self.display.image(self.image)
                self.display.show()
            else:
                # Simulate display update
                self._print_simulation(visible_lines, text_buffer)
            self.last_render = current_image
            
    def _print_simulation(self, lines, buffer):
        """Print simulation to console"""
        os.system('clear' if os.name == 'posix' else 'cls')
        print("=" * 60)
        for line in lines:
            print(line)
        print("=" * 60)
        print(f"Lines: {len(buffer.lines)} | Scroll: {buffer.scroll_offset}")

# ============== MAIN APPLICATION ==============
def main():
    print("Starting Text Editor...")
    print("Controls: Type normally, Ctrl+S to save, Ctrl+L to load")
    print("          Arrow keys to navigate, Backspace/Delete to erase")
    print("          Ctrl+X to exit")
    
    # Initialize renderer
    renderer = DisplayRenderer(display, FONT_PATH, FONT_SIZE)
    
    # Initialize text buffer with measured character dimensions
    text_buffer = TextBuffer(WIDTH, renderer.char_width, renderer.char_height)
    
    # Try to load existing text
    text_buffer.load_from_file()
    
    # Initial render
    renderer.render_text(text_buffer)
    
    # For platforms without keyboard module, use simple input
    try:
        import keyboard
        HAS_KEYBOARD = True
        print("Keyboard module available - using advanced input")
    except ImportError:
        HAS_KEYBOARD = False
        print("Keyboard module not available - using simple input")
        print("Enter text (press Enter twice to exit):")
    
    running = True
    last_update = time.time()
    update_interval = 0.05  # 20 FPS max
    
    while running:
        current_time = time.time()
        
        if HAS_KEYBOARD:
            # Using keyboard module for better control
            if keyboard.is_pressed('ctrl+x'):
                running = False
                break
                
            if keyboard.is_pressed('ctrl+s'):
                if text_buffer.save_to_file():
                    print("Saved!")
                time.sleep(0.5)  # Debounce
                
            if keyboard.is_pressed('ctrl+l'):
                if text_buffer.load_from_file():
                    print("Loaded!")
                time.sleep(0.5)
            
            # Check for individual key presses
            for key_event in keyboard.get_typed_strings(keyboard.record()):
                for char in key_event:
                    text_buffer.add_char(char)
                    
            # Check for special keys
            if keyboard.is_pressed('backspace'):
                text_buffer.backspace()
                time.sleep(0.05)  # Delay for repeat
                
            if keyboard.is_pressed('delete'):
                text_buffer.delete()
                time.sleep(0.05)
                
            if keyboard.is_pressed('enter'):
                text_buffer.add_char('\n')
                time.sleep(0.1)
                
            # Arrow keys
            if keyboard.is_pressed('left'):
                text_buffer.move_cursor(-1, 0)
                time.sleep(0.1)
            if keyboard.is_pressed('right'):
                text_buffer.move_cursor(1, 0)
                time.sleep(0.1)
            if keyboard.is_pressed('up'):
                text_buffer.move_cursor(0, -1)
                time.sleep(0.1)
            if keyboard.is_pressed('down'):
                text_buffer.move_cursor(0, 1)
                time.sleep(0.1)
                
        else:
            # Simple console input fallback
            try:
                import sys
                import select
                
                if sys.stdin in select.select([sys.stdin], [], [], 0)[0]:
                    line = sys.stdin.readline()
                    if line == '\n\n':  # Double newline to exit
                        running = False
                    else:
                        for char in line:
                            text_buffer.add_char(char)
            except:
                pass
        
        # Throttle updates for performance
        if current_time - last_update >= update_interval:
            renderer.render_text(text_buffer)
            last_update = current_time
            
        time.sleep(0.01)  # Small sleep to prevent CPU hogging
        
    print("\nExiting...")
    text_buffer.save_to_file()
    print("Text saved to text_output.txt")

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nInterrupted by user")
    except Exception as e:
        print(f"Error: {e}")