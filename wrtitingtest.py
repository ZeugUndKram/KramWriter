"""
Responsive Text Editor for Sharp Memory Display (400x240)
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
# Display configuration - 400x240 display
try:
    spi = busio.SPI(board.SCK, MOSI=board.MOSI)
    scs = digitalio.DigitalInOut(board.D6)
    display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)
    HAS_DISPLAY = True
    print(f"Display detected: {display.width}x{display.height}")
except (RuntimeError, ImportError):
    print("No display hardware detected - running in simulation mode")
    HAS_DISPLAY = False
    # Simulated display dimensions - 400x240
    class SimulatedDisplay:
        width = 400
        height = 240
        def fill(self, *args): pass
        def show(self): pass
        def image(self, img): pass
    display = SimulatedDisplay()

# Display parameters
WIDTH = display.width  # 400
HEIGHT = display.height  # 240
BORDER = 5
MAX_LINES = 100  # Maximum lines to keep in buffer for scrolling

# Colors
BLACK = 0
WHITE = 255

# Font configuration (adjust for larger display)
FONT_PATHS = [
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/truetype/droid/DroidSans.ttf",
    "arial.ttf"  # Windows fallback
]
FONT_SIZE = 12  # Larger font for better readability on big display

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
        self.max_visible_lines = (HEIGHT - 20) // char_height  # Reserve space for status bar
        self.max_chars_per_line = (display_width - BORDER * 2) // char_width
        
        print(f"Display: {display_width}x{HEIGHT}")
        print(f"Char size: {char_width}x{char_height}")
        print(f"Visible lines: {self.max_visible_lines}")
        print(f"Chars per line: {self.max_chars_per_line}")
        
    def add_char(self, char):
        """Add a character at current position with word wrapping"""
        if char == '\n':
            self._insert_newline()
            return
            
        if self.current_pos >= len(self.lines[self.current_line]):
            # Append to end of line
            self.lines[self.current_line] += char
            self.current_pos += 1
            
            # Check if we need to wrap (with margin)
            text_width = len(self.lines[self.current_line]) * self.char_width
            if text_width > self.display_width - BORDER * 2 - 10:
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
        
    def move_to_line_start(self):
        """Move cursor to start of current line"""
        self.current_pos = 0
        
    def move_to_line_end(self):
        """Move cursor to end of current line"""
        self.current_pos = len(self.lines[self.current_line])
        
    def move_to_document_start(self):
        """Move cursor to start of document"""
        self.current_line = 0
        self.current_pos = 0
        self.scroll_offset = 0
        
    def move_to_document_end(self):
        """Move cursor to end of document"""
        self.current_line = len(self.lines) - 1
        self.current_pos = len(self.lines[self.current_line])
        self._auto_scroll()
        
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
        max_chars = self.max_chars_per_line
        for i in range(min(len(line) - 1, max_chars), -1, -1):
            if line[i] == ' ':
                wrap_pos = i + 1  # Include the space in the first line
                break
                
        # If no space found, wrap at max chars
        if wrap_pos == len(line) or wrap_pos == 0:
            wrap_pos = min(max_chars, len(line))
            
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
                # Insert cursor indicator (blinking would be better but requires timing)
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
            print(f"Saved {len(self.lines)} lines to {filename}")
            return True
        except Exception as e:
            print(f"Error saving: {e}")
            return False
            
    def load_from_file(self, filename="text_output.txt"):
        """Load buffer from file"""
        try:
            if os.path.exists(filename):
                with open(filename, "r") as f:
                    self.lines = [line.rstrip('\n') for line in f.readlines()] or [""]
                self.current_line = min(self.current_line, len(self.lines) - 1)
                self.current_pos = min(self.current_pos, len(self.lines[self.current_line]))
                self.scroll_offset = 0
                print(f"Loaded {len(self.lines)} lines from {filename}")
                return True
        except Exception as e:
            print(f"Error loading: {e}")
        return False

# ============== RENDERER CLASS ==============
class DisplayRenderer:
    def __init__(self, display, font_paths, font_size):
        self.display = display
        self.image = Image.new("1", (WIDTH, HEIGHT))
        self.draw = ImageDraw.Draw(self.image)
        
        # Try to load font from multiple paths
        self.font = None
        for font_path in font_paths:
            try:
                self.font = ImageFont.truetype(font_path, font_size)
                print(f"Loaded font: {font_path}")
                break
            except:
                continue
                
        if self.font is None:
            print("Using default font")
            self.font = ImageFont.load_default()
            
        # Measure actual character dimensions
        bbox = self.font.getbbox("M")
        self.char_width = bbox[2] - bbox[0]
        self.char_height = bbox[3] - bbox[1] + 2  # Add some spacing
        
        print(f"Character dimensions: {self.char_width}x{self.char_height}")
        
        self.last_render = None
        self.cursor_blink = False
        self.last_blink_time = time.time()
        
    def render_text(self, text_buffer):
        """Render text buffer to display efficiently"""
        # Clear display
        self.draw.rectangle((0, 0, WIDTH, HEIGHT), fill=WHITE)
        
        # Draw border
        self.draw.rectangle(
            (BORDER, BORDER, WIDTH - BORDER - 1, HEIGHT - BORDER - 1),
            outline=BLACK,
            fill=WHITE,
        )
        
        # Get visible text
        visible_lines = text_buffer.get_visible_text()
        
        # Draw each line
        y = BORDER + 2
        text_area_width = WIDTH - BORDER * 2 - 4
        
        for line in visible_lines:
            # Truncate line if too long (shouldn't happen with wrapping, but just in case)
            if len(line) * self.char_width > text_area_width:
                max_chars = text_area_width // self.char_width
                line = line[:max_chars]
            
            self.draw.text((BORDER + 2, y), line, font=self.font, fill=BLACK)
            y += self.char_height
            if y >= HEIGHT - self.char_height - BORDER:
                break
                
        # Draw status bar with better visibility
        status_height = self.char_height + 4
        self.draw.rectangle((0, HEIGHT - status_height, WIDTH, HEIGHT), fill=BLACK)
        
        # Status information
        status_text = f"Ln:{text_buffer.current_line+1}/{len(text_buffer.lines)} Col:{text_buffer.current_pos+1}"
        self.draw.text((BORDER + 2, HEIGHT - status_height + 2), status_text, 
                      font=self.font, fill=WHITE)
        
        # File indicator
        file_indicator = "SAVED" if text_buffer.save_to_file("temp_check") else "UNSAVED"
        file_width = self.font.getbbox(file_indicator)[2]
        self.draw.text((WIDTH - file_width - BORDER - 2, HEIGHT - status_height + 2), 
                      file_indicator, font=self.font, fill=WHITE)
        
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
        print("=" * 80)
        print(f"DISPLAY SIMULATION: {WIDTH}x{HEIGHT} (showing {len(lines)} of {len(buffer.lines)} lines)")
        print("=" * 80)
        for i, line in enumerate(lines):
            line_num = buffer.scroll_offset + i + 1
            print(f"{line_num:3d}: {line}")
        print("=" * 80)
        print(f"Cursor: Ln {buffer.current_line+1}, Col {buffer.current_pos+1} | "
              f"Scroll: {buffer.scroll_offset}")

# ============== INPUT HANDLER ==============
class InputHandler:
    def __init__(self):
        self.has_keyboard = False
        try:
            import keyboard
            self.keyboard = keyboard
            self.has_keyboard = True
            print("Keyboard module loaded")
        except ImportError:
            print("Keyboard module not available - using simple input")
            
        self.last_key_time = 0
        self.key_repeat_delay = 0.1
        self.active_keys = set()
        
    def get_input(self):
        """Get keyboard input efficiently"""
        if not self.has_keyboard:
            return self._get_simple_input()
            
        inputs = []
        current_time = time.time()
        
        # Check for modifier combinations first
        if self.keyboard.is_pressed('ctrl'):
            if self.keyboard.is_pressed('s') and current_time - self.last_key_time > 0.5:
                inputs.append('ctrl+s')
                self.last_key_time = current_time
            elif self.keyboard.is_pressed('l') and current_time - self.last_key_time > 0.5:
                inputs.append('ctrl+l')
                self.last_key_time = current_time
            elif self.keyboard.is_pressed('x') and current_time - self.last_key_time > 0.5:
                inputs.append('ctrl+x')
                self.last_key_time = current_time
            elif self.keyboard.is_pressed('home') and current_time - self.last_key_time > 0.3:
                inputs.append('ctrl+home')
                self.last_key_time = current_time
            elif self.keyboard.is_pressed('end') and current_time - self.last_key_time > 0.3:
                inputs.append('ctrl+end')
                self.last_key_time = current_time
                
        # Check for regular key presses with debouncing
        for event in self.keyboard.get_typed_strings(self.keyboard.record()):
            for char in event:
                if current_time - self.last_key_time > 0.05:  # Basic debounce
                    inputs.append(char)
                    self.last_key_time = current_time
                    
        # Check for special keys
        special_keys = ['backspace', 'delete', 'enter', 'left', 'right', 'up', 'down', 'home', 'end']
        for key in special_keys:
            if self.keyboard.is_pressed(key):
                if key not in self.active_keys or current_time - self.last_key_time > self.key_repeat_delay:
                    inputs.append(key)
                    self.active_keys.add(key)
                    self.last_key_time = current_time
            else:
                self.active_keys.discard(key)
                
        return inputs
        
    def _get_simple_input(self):
        """Simple input fallback for platforms without keyboard module"""
        try:
            import sys
            import select
            import termios
            import tty
            
            # Set non-blocking input
            fd = sys.stdin.fileno()
            old_settings = termios.tcgetattr(fd)
            
            try:
                tty.setraw(fd)
                if sys.stdin in select.select([sys.stdin], [], [], 0.01)[0]:
                    char = sys.stdin.read(1)
                    
                    # Map special characters
                    if ord(char) == 3:  # Ctrl+C
                        return ['ctrl+x']
                    elif ord(char) == 19:  # Ctrl+S
                        return ['ctrl+s']
                    elif ord(char) == 12:  # Ctrl+L
                        return ['ctrl+l']
                    elif ord(char) == 127:  # Backspace
                        return ['backspace']
                    elif char == '\n':
                        return ['enter']
                    elif ord(char) == 27:  # Escape sequence
                        next_chars = sys.stdin.read(2)
                        if next_chars == '[A':
                            return ['up']
                        elif next_chars == '[B':
                            return ['down']
                        elif next_chars == '[C':
                            return ['right']
                        elif next_chars == '[D':
                            return ['left']
                        elif next_chars == '[H':
                            return ['home']
                        elif next_chars == '[F':
                            return ['end']
                    else:
                        return [char]
            finally:
                termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
        except:
            pass
            
        return []

# ============== MAIN APPLICATION ==============
def main():
    print(f"Starting Text Editor for {WIDTH}x{HEIGHT} display...")
    print("=" * 60)
    print("Controls:")
    print("  Type normally to enter text")
    print("  Arrow keys: Navigate")
    print("  Home/End: Move to line start/end")
    print("  Ctrl+Home/Ctrl+End: Move to document start/end")
    print("  Backspace/Delete: Erase")
    print("  Enter: New line")
    print("  Ctrl+S: Save to file")
    print("  Ctrl+L: Load from file")
    print("  Ctrl+X: Exit")
    print("=" * 60)
    
    # Initialize renderer
    renderer = DisplayRenderer(display, FONT_PATHS, FONT_SIZE)
    
    # Initialize text buffer with measured character dimensions
    text_buffer = TextBuffer(WIDTH - BORDER * 2, renderer.char_width, renderer.char_height)
    
    # Initialize input handler
    input_handler = InputHandler()
    
    # Try to load existing text
    text_buffer.load_from_file()
    
    # Initial render
    renderer.render_text(text_buffer)
    
    running = True
    last_update = time.time()
    update_interval = 0.033  # ~30 FPS
    
    print("Ready for input...")
    
    while running:
        current_time = time.time()
        
        # Process input
        inputs = input_handler.get_input()
        
        for input_char in inputs:
            if input_char == 'ctrl+x':
                running = False
                break
            elif input_char == 'ctrl+s':
                text_buffer.save_to_file()
                time.sleep(0.2)  # Brief pause after save
            elif input_char == 'ctrl+l':
                text_buffer.load_from_file()
                time.sleep(0.2)
            elif input_char == 'ctrl+home':
                text_buffer.move_to_document_start()
            elif input_char == 'ctrl+end':
                text_buffer.move_to_document_end()
            elif input_char == 'home':
                text_buffer.move_to_line_start()
            elif input_char == 'end':
                text_buffer.move_to_line_end()
            elif input_char == 'backspace':
                text_buffer.backspace()
            elif input_char == 'delete':
                text_buffer.delete()
            elif input_char == 'enter':
                text_buffer.add_char('\n')
            elif input_char == 'left':
                text_buffer.move_cursor(-1, 0)
            elif input_char == 'right':
                text_buffer.move_cursor(1, 0)
            elif input_char == 'up':
                text_buffer.move_cursor(0, -1)
            elif input_char == 'down':
                text_buffer.move_cursor(0, 1)
            elif len(input_char) == 1:
                # Regular character
                text_buffer.add_char(input_char)
        
        # Update display at controlled rate
        if current_time - last_update >= update_interval:
            renderer.render_text(text_buffer)
            last_update = current_time
            
        # Small sleep to prevent CPU hogging
        time.sleep(0.001)
        
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
        import traceback
        traceback.print_exc()