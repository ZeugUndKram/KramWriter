import random
import time
import curses
import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay
import threading
import queue
from collections import deque

# Initialize the Sharp Memory Display
spi = busio.SPI(board.SCK, MOSI=board.MOSI)
scs = digitalio.DigitalInOut(board.D6)
display = adafruit_sharpmemorydisplay.SharpMemoryDisplay(spi, scs, 400, 240)

# Tetris colors (converted to 1-bit)
BLACK = 0
WHITE = 255

class Figure:
    figures = [
        [[1, 5, 9, 13], [4, 5, 6, 7]],  # I
        [[4, 5, 9, 10], [2, 6, 5, 9]],  # Z
        [[6, 7, 9, 10], [1, 5, 6, 10]], # S
        [[1, 2, 5, 9], [0, 4, 5, 6], [1, 5, 9, 8], [4, 5, 6, 10]], # J
        [[1, 2, 6, 10], [5, 6, 7, 9], [2, 6, 10, 11], [3, 5, 6, 7]], # L
        [[1, 4, 5, 6], [1, 4, 5, 9], [4, 5, 6, 9], [1, 5, 6, 9]], # T
        [[1, 2, 5, 6]], # O
    ]
    
    def __init__(self, x, y):
        self.x = x
        self.y = y
        self.type = random.randint(0, len(self.figures) - 1)
        self.color = 1  # All blocks are white in monochrome
        self.rotation = 0

    def image(self):
        return self.figures[self.type][self.rotation]

    def rotate(self):
        self.rotation = (self.rotation + 1) % len(self.figures[self.type])
    
    def copy(self):
        """Create a copy of this figure"""
        new_figure = Figure(self.x, self.y)
        new_figure.type = self.type
        new_figure.rotation = self.rotation
        return new_figure


class Tetris:
    def __init__(self, height, width):
        self.level = 1
        self.score = 0
        self.state = "start"
        self.height = height
        self.width = width
        self.zoom = 8  # Block size
        self.preview_zoom = 6  # Smaller zoom for preview
        
        # Center the playing field vertically and horizontally
        field_width = width * self.zoom
        field_height = height * self.zoom
        
        # Space for preview on the right (preview area: 80 pixels)
        self.x = (display.width - field_width - 100) // 2  # Leave space for preview
        self.y = (display.height - field_height) // 2  # Center vertically
        
        # Preview position (to the right of main field)
        self.preview_x = self.x + field_width + 20
        self.preview_y = self.y + 30
        
        self.figure = None
        self.next_figure = None  # Next piece preview
        
        # Initialize field
        self.field = [[0 for _ in range(width)] for _ in range(height)]
        
        # Line clearing animation
        self.lines_to_clear = []  # Stores lines that need to be cleared
        self.line_clear_timer = 0  # Timer for blinking animation
        self.line_blink_state = True  # Current blink state (True = visible, False = invisible)
        self.line_clear_duration = 0.5  # Total duration of clearing animation in seconds
        self.line_blink_interval = 0.1  # Time between blinks in seconds
        
        # Load font
        try:
            self.font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 14)
            self.small_font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 12)
        except:
            self.font = ImageFont.load_default()
            self.small_font = ImageFont.load_default()

    def new_figure(self):
        # If we have a next figure, use it as current
        if self.next_figure:
            self.figure = self.next_figure
            self.figure.x = self.width // 2 - 2
            self.figure.y = 0
        else:
            self.figure = Figure(self.width // 2 - 2, 0)
        
        # Create new next figure for preview
        self.next_figure = Figure(0, 0)
        # Position the preview figure in the center of preview area
        self.next_figure.x = 0
        self.next_figure.y = 0

    def intersects(self):
        for i in range(4):
            for j in range(4):
                if i * 4 + j in self.figure.image():
                    if (i + self.figure.y >= self.height or
                        j + self.figure.x >= self.width or
                        j + self.figure.x < 0 or
                        self.field[i + self.figure.y][j + self.figure.x] > 0):
                        return True
        return False

    def freeze(self):
        for i in range(4):
            for j in range(4):
                if i * 4 + j in self.figure.image():
                    self.field[i + self.figure.y][j + self.figure.x] = self.figure.color
        
        # Check for completed lines
        self.lines_to_clear = []
        for i in range(self.height):
            if all(cell > 0 for cell in self.field[i]):
                self.lines_to_clear.append(i)
        
        # Start blinking animation if there are lines to clear
        if self.lines_to_clear:
            self.line_clear_timer = time.time()
            self.state = "clearing"
            return
        
        # Create new figure
        self.new_figure()
        if self.intersects():
            self.state = "gameover"

    def hard_drop(self):
        """Hard drop - instantly drop piece to bottom"""
        if not self.figure or self.state != "start":
            return
            
        # Move down until collision
        while True:
            self.figure.y += 1
            if self.intersects():
                self.figure.y -= 1
                self.freeze()
                break

    def clear_lines(self):
        """Remove completed lines and update score"""
        lines_cleared = len(self.lines_to_clear)
        
        # Remove lines and add to score
        for line in reversed(self.lines_to_clear):
            del self.field[line]
            self.field.insert(0, [0 for _ in range(self.width)])
        
        # Add score based on number of lines cleared
        if lines_cleared > 0:
            # Standard Tetris scoring: more points for multiple lines
            if lines_cleared == 1:
                self.score += 100 * self.level
            elif lines_cleared == 2:
                self.score += 300 * self.level
            elif lines_cleared == 3:
                self.score += 500 * self.level
            elif lines_cleared >= 4:
                self.score += 800 * self.level  # Tetris!
        
        # Reset line clearing state
        self.lines_to_clear = []
        
        # Create new figure
        self.new_figure()
        if self.intersects():
            self.state = "gameover"
        else:
            self.state = "start"

    def go_down(self):
        self.figure.y += 1
        if self.intersects():
            self.figure.y -= 1
            self.freeze()

    def go_side(self, dx):
        old_x = self.figure.x
        self.figure.x += dx
        if self.intersects():
            self.figure.x = old_x

    def rotate(self):
        old_rotation = self.figure.rotation
        self.figure.rotate()
        if self.intersects():
            self.figure.rotation = old_rotation

    def draw_preview(self, draw):
        """Draw the next piece preview"""
        if not self.next_figure:
            return
            
        # Draw preview box
        preview_width = 5 * self.preview_zoom  # 5 blocks wide
        preview_height = 5 * self.preview_zoom  # 5 blocks high
        box_left = self.preview_x - 1
        box_top = self.preview_y - 1
        box_right = self.preview_x + preview_width + 1
        box_bottom = self.preview_y + preview_height + 1
        
        # Draw preview border
        draw.rectangle(
            (box_left, box_top, box_right, box_bottom),
            outline=WHITE, fill=BLACK
        )
        
        # Draw "NEXT" label
        next_text = "NEXT"
        next_width = draw.textlength(next_text, font=self.small_font)
        draw.text(
            (self.preview_x + preview_width // 2 - next_width // 2, self.preview_y - 20),
            next_text, font=self.small_font, fill=WHITE
        )
        
        # Draw the preview piece
        # Center the preview piece in the preview box
        # Calculate piece dimensions
        min_x = min([j % 4 for j in self.next_figure.image()])
        max_x = max([j % 4 for j in self.next_figure.image()])
        min_y = min([j // 4 for j in self.next_figure.image()])
        max_y = max([j // 4 for j in self.next_figure.image()])
        
        piece_width = (max_x - min_x + 1) * self.preview_zoom
        piece_height = (max_y - min_y + 1) * self.preview_zoom
        
        # Center position
        piece_x = self.preview_x + (preview_width - piece_width) // 2
        piece_y = self.preview_y + (preview_height - piece_height) // 2
        
        # Draw each block of the preview piece
        for j in self.next_figure.image():
            block_x = j % 4 - min_x
            block_y = j // 4 - min_y
            x = piece_x + block_x * self.preview_zoom
            y = piece_y + block_y * self.preview_zoom
            draw.rectangle(
                [x, y, x + self.preview_zoom - 1, y + self.preview_zoom - 1],
                outline=WHITE, fill=WHITE
            )

    def draw(self, draw, fps=0, input_lag=0):
        """Draw game on display"""
        # Clear display
        draw.rectangle((0, 0, display.width, display.height), fill=BLACK)
        
        # Draw main game border
        border_left = self.x - 1
        border_top = self.y - 1
        border_right = self.x + self.width * self.zoom + 1
        border_bottom = self.y + self.height * self.zoom + 1
        
        draw.rectangle(
            (border_left, border_top, border_right, border_bottom),
            outline=WHITE, fill=BLACK
        )
        
        # Draw placed blocks (except blinking lines if in clearing state)
        for i in range(self.height):
            # Skip drawing if this line is being cleared and it's in the "off" blink state
            if self.state == "clearing" and i in self.lines_to_clear and not self.line_blink_state:
                continue
                
            for j in range(self.width):
                if self.field[i][j] > 0:
                    x = self.x + j * self.zoom
                    y = self.y + i * self.zoom
                    draw.rectangle([x, y, x + self.zoom - 1, y + self.zoom - 1], 
                                  outline=WHITE, fill=WHITE)
        
        # Draw current figure (if not in clearing state)
        if self.figure and self.state != "clearing":
            for i in range(4):
                for j in range(4):
                    if i * 4 + j in self.figure.image():
                        x = self.x + (j + self.figure.x) * self.zoom
                        y = self.y + (i + self.figure.y) * self.zoom
                        draw.rectangle([x, y, x + self.zoom - 1, y + self.zoom - 1], 
                                      outline=WHITE, fill=WHITE)
        
        # Draw next piece preview
        self.draw_preview(draw)
        
        # Draw score (top left)
        score_text = f"Score: {self.score}"
        draw.text((10, 10), score_text, font=self.font, fill=WHITE)
        
        # Draw level (top right)
        level_text = f"Level: {self.level}"
        level_text_width = draw.textlength(level_text, font=self.font)
        draw.text((display.width - level_text_width - 10, 10), level_text, font=self.font, fill=WHITE)
        
        # Draw FPS counter (top middle)
        fps_text = f"FPS: {fps:.1f}"
        fps_width = draw.textlength(fps_text, font=self.small_font)
        
        # Draw input lag (next to FPS)
        lag_text = f"Lag: {input_lag:.0f}ms"
        lag_width = draw.textlength(lag_text, font=self.small_font)
        
        draw.text((display.width // 2 - fps_width - 5, 10), fps_text, font=self.small_font, fill=WHITE)
        draw.text((display.width // 2 + 5, 10), lag_text, font=self.small_font, fill=WHITE)
        
        # Draw lines cleared indicator during animation
        if self.state == "clearing" and self.lines_to_clear:
            lines_text = f"Lines: {len(self.lines_to_clear)}"
            text_width = draw.textlength(lines_text, font=self.font)
            draw.text((display.width // 2 - text_width // 2, self.y - 30), 
                     lines_text, font=self.font, fill=WHITE)
        
        # Draw controls help
        if self.state == "start":
            controls = "←→:Move  ↑:Rotate  Space:Drop  ↓:Fast"
            text_width = draw.textlength(controls, font=self.small_font)
            draw.text((display.width // 2 - text_width // 2, display.height - 25), 
                     controls, font=self.small_font, fill=WHITE)
            
            # Add hint for restart/quit
            restart_hint = "R:Restart  Q:Quit"
            hint_width = draw.textlength(restart_hint, font=self.small_font)
            draw.text((display.width // 2 - hint_width // 2, display.height - 10), 
                     restart_hint, font=self.small_font, fill=WHITE)
        
        # Game over screen
        if self.state == "gameover":
            # Semi-transparent overlay
            for i in range(0, display.height, 2):
                draw.line([(0, i), (display.width, i)], fill=WHITE, width=1)
            
            game_over = "GAME OVER"
            game_over_width = draw.textlength(game_over, font=self.font)
            draw.text((display.width // 2 - game_over_width // 2, display.height // 2 - 30), 
                     game_over, font=self.font, fill=BLACK)
            
            final_score = f"Score: {self.score}"
            final_score_width = draw.textlength(final_score, font=self.font)
            draw.text((display.width // 2 - final_score_width // 2, display.height // 2), 
                     final_score, font=self.font, fill=BLACK)
            
            restart = "Press R to restart"
            restart_width = draw.textlength(restart, font=self.font)
            draw.text((display.width // 2 - restart_width // 2, display.height // 2 + 30), 
                     restart, font=self.font, fill=BLACK)

    def update_animation(self, current_time):
        """Update line clearing animation"""
        if self.state == "clearing" and self.lines_to_clear:
            elapsed_time = current_time - self.line_clear_timer
            
            # Calculate blink state based on elapsed time
            blink_count = int(elapsed_time / self.line_blink_interval)
            self.line_blink_state = (blink_count % 2 == 0)
            
            # If animation duration has passed, clear the lines
            if elapsed_time >= self.line_clear_duration:
                self.clear_lines()


class InputHandler:
    """High-performance input handler with minimal latency"""
    
    def __init__(self):
        self.key_queue = deque(maxlen=10)  # Buffer for key presses
        self.last_key_time = {}
        self.key_repeat_initial = 0.15  # 150ms initial delay
        self.key_repeat_interval = 0.05  # 50ms repeat interval
        self.running = False
        self.thread = None
        
    def start(self):
        """Start input thread"""
        self.running = True
        self.thread = threading.Thread(target=self._input_loop, daemon=True)
        self.thread.start()
        
    def stop(self):
        """Stop input thread"""
        self.running = False
        if self.thread:
            self.thread.join(timeout=0.5)
            
    def get_keys(self):
        """Get all pending keys (non-blocking)"""
        keys = []
        while self.key_queue:
            keys.append(self.key_queue.popleft())
        return keys
        
    def _input_loop(self):
        """Main input loop running in separate thread"""
        # Initialize curses in this thread
        stdscr = curses.initscr()
        curses.curs_set(0)
        curses.noecho()
        curses.cbreak()
        stdscr.nodelay(True)  # Non-blocking
        stdscr.keypad(True)   # Enable special keys
        
        # Disable timeout - we'll handle timing ourselves
        stdscr.timeout(0)
        
        print("Input thread started")
        
        try:
            while self.running:
                current_time = time.time()
                
                # Read all available keys
                while True:
                    key = stdscr.getch()
                    if key == -1:
                        break  # No more keys
                        
                    # Handle key press
                    if key in [curses.KEY_UP, curses.KEY_DOWN, curses.KEY_LEFT, curses.KEY_RIGHT,
                              ord(' '), ord('r'), ord('R'), ord('q'), ord('Q')]:
                        
                        if key in self.last_key_time:
                            # Key is being held
                            elapsed = current_time - self.last_key_time[key]
                            if elapsed > self.key_repeat_initial:
                                # In repeat mode, check interval
                                if elapsed - self.key_repeat_initial > self.key_repeat_interval:
                                    self.key_queue.append(key)
                                    self.last_key_time[key] = current_time - (self.key_repeat_initial - self.key_repeat_interval)
                            else:
                                # Still in initial delay, process once
                                self.key_queue.append(key)
                                self.last_key_time[key] = current_time
                        else:
                            # New key press
                            self.key_queue.append(key)
                            self.last_key_time[key] = current_time
                    
                # Clear key states for keys that are no longer pressed
                keys_to_remove = []
                for key, press_time in list(self.last_key_time.items()):
                    if current_time - press_time > 0.5:  # 500ms since last press
                        keys_to_remove.append(key)
                for key in keys_to_remove:
                    del self.last_key_time[key]
                
                # Very short sleep to prevent CPU hogging
                time.sleep(0.001)  # 1ms sleep
                
        except Exception as e:
            print(f"Input error: {e}")
        finally:
            # Clean up curses
            curses.nocbreak()
            curses.echo()
            curses.endwin()
            print("Input thread stopped")


def main_game_loop():
    """Main game loop with separate input handling"""
    
    print("\n" + "=" * 50)
    print("TETRIS on Sharp Memory Display")
    print("Controls:")
    print("  Arrow keys: Move/Rotate/Fast drop")
    print("  Spacebar: Hard drop (instant)")
    print("  R: Restart game")
    print("  Q: Quit game")
    print("=" * 50)
    print("Input handling in separate thread for minimal latency")
    print("=" * 50)
    
    # Create game
    game = Tetris(20, 10)
    game.new_figure()
    
    # Start high-performance input handler
    input_handler = InputHandler()
    input_handler.start()
    
    # Game loop variables
    last_drop_time = time.time()
    drop_interval = 0.5
    
    # FPS calculation
    frame_count = 0
    fps = 0
    fps_last_time = time.time()
    
    # Input lag measurement
    input_timestamps = deque(maxlen=60)
    avg_input_lag = 0
    
    try:
        while True:
            frame_start = time.time()
            frame_count += 1
            
            # Calculate FPS every second
            current_time = time.time()
            if current_time - fps_last_time >= 0.5:  # Update every 0.5s for smoother display
                fps = frame_count / (current_time - fps_last_time)
                frame_count = 0
                fps_last_time = current_time
            
            # Update animation if in clearing state
            if game.state == "clearing":
                game.update_animation(current_time)
            
            # Process input IMMEDIATELY
            input_keys = input_handler.get_keys()
            
            if input_keys:
                # Record input timestamp for lag measurement
                input_timestamps.append(frame_start)
                
                # Process all queued keys
                for key in input_keys:
                    if key == ord(' '):  # Spacebar for hard drop
                        if game.state == "start":
                            game.hard_drop()
                    elif key == ord('r') or key == ord('R'):
                        if game.state == "gameover" or game.state == "start":
                            game = Tetris(20, 10)
                            game.new_figure()
                    elif key == ord('q') or key == ord('Q'):
                        print("\nQuitting game...")
                        return
                    elif game.state == "start":
                        if key == curses.KEY_UP:
                            game.rotate()
                        elif key == curses.KEY_DOWN:
                            game.go_down()
                        elif key == curses.KEY_LEFT:
                            game.go_side(-1)
                        elif key == curses.KEY_RIGHT:
                            game.go_side(1)
            
            # Calculate average input lag (time from key press to processing)
            if input_timestamps:
                avg_input_lag = sum(input_timestamps) / len(input_timestamps)
                avg_input_lag = (current_time - avg_input_lag) * 1000  # Convert to ms
            
            # Auto-drop (only if in start state)
            if game.state == "start" and current_time - last_drop_time > drop_interval:
                game.go_down()
                last_drop_time = current_time
                # Speed up as score increases
                drop_interval = max(0.1, 0.5 - (game.score / 1000) * 0.1)
            
            # Render game
            image = Image.new("1", (display.width, display.height))
            draw = ImageDraw.Draw(image)
            game.draw(draw, fps, avg_input_lag)
            
            # Update display
            display.image(image)
            display.show()
            
            # Calculate frame time and adaptive sleep
            frame_end = time.time()
            frame_time = frame_end - frame_start
            
            # Target 60 FPS (16.67ms per frame)
            target_frame_time = 1.0 / 60.0
            
            if frame_time < target_frame_time:
                sleep_time = target_frame_time - frame_time
                # Use time.sleep for small sleeps, busy wait for very small sleeps
                if sleep_time > 0.001:  # 1ms
                    time.sleep(sleep_time)
                elif sleep_time > 0.0001:  # 100µs
                    # Busy wait for very short sleeps (more accurate)
                    end_time = time.perf_counter() + sleep_time
                    while time.perf_counter() < end_time:
                        pass
            
    except KeyboardInterrupt:
        print("\nGame interrupted")
    except Exception as e:
        print(f"Error: {e}")
        import traceback
        traceback.print_exc()
    finally:
        # Clean up
        input_handler.stop()
        
        # Clear display
        display.fill(1)
        display.show()
        
        print("\nGame ended")


# Alternative: Even simpler input without curses (using keyboard polling)
def simple_input_handler():
    """Alternative input handler that might be faster"""
    import select
    import sys
    import tty
    import termios
    
    # Save terminal settings
    old_settings = termios.tcgetattr(sys.stdin)
    
    try:
        tty.setcbreak(sys.stdin.fileno())
        
        key_map = {
            'A': curses.KEY_UP,
            'B': curses.KEY_DOWN,
            'C': curses.KEY_RIGHT,
            'D': curses.KEY_LEFT,
            ' ': ord(' '),
            'r': ord('r'),
            'R': ord('R'),
            'q': ord('q'),
            'Q': ord('Q'),
        }
        
        while True:
            if select.select([sys.stdin], [], [], 0.001)[0]:
                key = sys.stdin.read(1)
                if key == '\x1b':  # Escape sequence
                    # Read the rest of the escape sequence
                    if select.select([sys.stdin], [], [], 0.01)[0]:
                        next_key = sys.stdin.read(2)
                        if next_key[0] == '[':
                            arrow_key = next_key[1]
                            if arrow_key in key_map:
                                yield key_map[arrow_key]
                elif key in key_map:
                    yield key_map[key]
            else:
                yield None
                
    finally:
        termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)


if __name__ == "__main__":
    try:
        main_game_loop()
    except KeyboardInterrupt:
        print("\nGame interrupted")
    finally:
        print("\nCleaning up...")
        display.fill(1)
        display.show()
        print("Goodbye!")