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
            # Use default font - this is faster
            self.font = ImageFont.load_default()
            self.small_font = ImageFont.load_default()
        
        # Cached images for faster drawing
        self.cached_field = None
        self.field_changed = True
        self.last_full_draw = 0

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
        self.field_changed = True

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
        self.field_changed = True
        
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
        self.field_changed = True

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
        self.field_changed = True
        
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
        self.field_changed = True

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

    def update_animation(self, current_time):
        """Update line clearing animation"""
        if self.state == "clearing" and self.lines_to_clear:
            elapsed_time = current_time - self.line_clear_timer
            
            # Calculate blink state based on elapsed time
            blink_count = int(elapsed_time / self.line_blink_interval)
            new_blink_state = (blink_count % 2 == 0)
            
            if new_blink_state != self.line_blink_state:
                self.line_blink_state = new_blink_state
                self.field_changed = True
            
            # If animation duration has passed, clear the lines
            if elapsed_time >= self.line_clear_duration:
                self.clear_lines()


class OptimizedRenderer:
    """Optimized renderer that only updates changed parts of the display"""
    
    def __init__(self, tetris_game):
        self.game = tetris_game
        self.last_image = None
        self.last_fps = 0
        self.last_lag = 0
        
        # Create base image template
        self.base_image = Image.new("1", (display.width, display.height), BLACK)
        self.draw_base()
        
    def draw_base(self):
        """Draw static elements once"""
        draw = ImageDraw.Draw(self.base_image)
        
        # Draw main game border (static)
        border_left = self.game.x - 1
        border_top = self.game.y - 1
        border_right = self.game.x + self.game.width * self.game.zoom + 1
        border_bottom = self.game.y + self.game.height * self.game.zoom + 1
        
        draw.rectangle(
            (border_left, border_top, border_right, border_bottom),
            outline=WHITE, fill=BLACK
        )
        
        # Draw preview box (static)
        preview_width = 5 * self.game.preview_zoom
        preview_height = 5 * self.game.preview_zoom
        box_left = self.game.preview_x - 1
        box_top = self.game.preview_y - 1
        box_right = self.game.preview_x + preview_width + 1
        box_bottom = self.game.preview_y + preview_height + 1
        
        draw.rectangle(
            (box_left, box_top, box_right, box_bottom),
            outline=WHITE, fill=BLACK
        )
        
        # Draw "NEXT" label (static)
        next_text = "NEXT"
        next_width = draw.textlength(next_text, font=self.game.small_font)
        draw.text(
            (self.game.preview_x + preview_width // 2 - next_width // 2, 
             self.game.preview_y - 20),
            next_text, font=self.game.small_font, fill=WHITE
        )
        
        # Draw controls help (static)
        controls = "←→:Move  ↑:Rotate  Space:Drop  ↓:Fast"
        text_width = draw.textlength(controls, font=self.game.small_font)
        draw.text((display.width // 2 - text_width // 2, display.height - 25), 
                 controls, font=self.game.small_font, fill=WHITE)
        
        restart_hint = "R:Restart  Q:Quit"
        hint_width = draw.textlength(restart_hint, font=self.game.small_font)
        draw.text((display.width // 2 - hint_width // 2, display.height - 10), 
                 restart_hint, font=self.game.small_font, fill=WHITE)
    
    def draw_dynamic(self, draw, fps=0, input_lag=0):
        """Draw dynamic elements that change frequently"""
        # Draw score
        score_text = f"Score: {self.game.score}"
        draw.rectangle((10, 10, 150, 30), fill=BLACK)  # Clear area
        draw.text((10, 10), score_text, font=self.game.font, fill=WHITE)
        
        # Draw level
        level_text = f"Level: {self.game.level}"
        level_text_width = draw.textlength(level_text, font=self.game.font)
        draw.rectangle((display.width - 150, 10, display.width, 30), fill=BLACK)
        draw.text((display.width - level_text_width - 10, 10), level_text, 
                 font=self.game.font, fill=WHITE)
        
        # Draw FPS and lag
        fps_text = f"FPS: {fps:.1f}"
        fps_width = draw.textlength(fps_text, font=self.game.small_font)
        lag_text = f"Lag: {input_lag:.0f}ms"
        
        # Clear area for FPS/lag
        draw.rectangle((display.width // 2 - 100, 10, display.width // 2 + 100, 25), fill=BLACK)
        draw.text((display.width // 2 - fps_width - 5, 10), fps_text, 
                 font=self.game.small_font, fill=WHITE)
        draw.text((display.width // 2 + 5, 10), lag_text, 
                 font=self.game.small_font, fill=WHITE)
        
        # Draw placed blocks (except blinking lines if in clearing state)
        for i in range(self.game.height):
            # Skip drawing if this line is being cleared and it's in the "off" blink state
            if self.game.state == "clearing" and i in self.game.lines_to_clear and not self.game.line_blink_state:
                # Clear this line
                for j in range(self.game.width):
                    x = self.game.x + j * self.game.zoom
                    y = self.game.y + i * self.game.zoom
                    draw.rectangle([x, y, x + self.game.zoom - 1, y + self.game.zoom - 1], 
                                  outline=BLACK, fill=BLACK)
                continue
                
            for j in range(self.game.width):
                if self.game.field[i][j] > 0:
                    x = self.game.x + j * self.game.zoom
                    y = self.game.y + i * self.game.zoom
                    draw.rectangle([x, y, x + self.game.zoom - 1, y + self.game.zoom - 1], 
                                  outline=WHITE, fill=WHITE)
                else:
                    # Clear this block if it was previously filled
                    x = self.game.x + j * self.game.zoom
                    y = self.game.y + i * self.game.zoom
                    draw.rectangle([x, y, x + self.game.zoom - 1, y + self.game.zoom - 1], 
                                  outline=BLACK, fill=BLACK)
        
        # Draw current figure (if not in clearing state)
        if self.game.figure and self.game.state != "clearing":
            # First, clear the area where the figure was
            if hasattr(self.game.figure, 'last_positions'):
                for pos in self.game.figure.last_positions:
                    x = self.game.x + (pos[1] + self.game.figure.last_x) * self.game.zoom
                    y = self.game.y + (pos[0] + self.game.figure.last_y) * self.game.zoom
                    draw.rectangle([x, y, x + self.game.zoom - 1, y + self.game.zoom - 1], 
                                  outline=BLACK, fill=BLACK)
            
            # Store current positions for next frame
            self.game.figure.last_positions = []
            self.game.figure.last_x = self.game.figure.x
            self.game.figure.last_y = self.game.figure.y
            
            # Draw new figure
            for i in range(4):
                for j in range(4):
                    if i * 4 + j in self.game.figure.image():
                        x = self.game.x + (j + self.game.figure.x) * self.game.zoom
                        y = self.game.y + (i + self.game.figure.y) * self.game.zoom
                        draw.rectangle([x, y, x + self.game.zoom - 1, y + self.game.zoom - 1], 
                                      outline=WHITE, fill=WHITE)
                        self.game.figure.last_positions.append((i, j))
        
        # Draw next piece preview
        if self.game.next_figure:
            # Clear preview area
            preview_width = 5 * self.game.preview_zoom
            preview_height = 5 * self.game.preview_zoom
            draw.rectangle(
                (self.game.preview_x, self.game.preview_y,
                 self.game.preview_x + preview_width,
                 self.game.preview_y + preview_height),
                fill=BLACK
            )
            
            # Calculate piece dimensions
            min_x = min([j % 4 for j in self.game.next_figure.image()])
            max_x = max([j % 4 for j in self.game.next_figure.image()])
            min_y = min([j // 4 for j in self.game.next_figure.image()])
            max_y = max([j // 4 for j in self.game.next_figure.image()])
            
            piece_width = (max_x - min_x + 1) * self.game.preview_zoom
            piece_height = (max_y - min_y + 1) * self.game.preview_zoom
            
            # Center position
            piece_x = self.game.preview_x + (preview_width - piece_width) // 2
            piece_y = self.game.preview_y + (preview_height - piece_height) // 2
            
            # Draw each block of the preview piece
            for j in self.game.next_figure.image():
                block_x = j % 4 - min_x
                block_y = j // 4 - min_y
                x = piece_x + block_x * self.game.preview_zoom
                y = piece_y + block_y * self.game.preview_zoom
                draw.rectangle(
                    [x, y, x + self.game.preview_zoom - 1, y + self.game.preview_zoom - 1],
                    outline=WHITE, fill=WHITE
                )
        
        # Game over screen
        if self.game.state == "gameover":
            # Semi-transparent overlay
            for i in range(0, display.height, 2):
                draw.line([(0, i), (display.width, i)], fill=WHITE, width=1)
            
            game_over = "GAME OVER"
            game_over_width = draw.textlength(game_over, font=self.game.font)
            draw.text((display.width // 2 - game_over_width // 2, display.height // 2 - 30), 
                     game_over, font=self.game.font, fill=BLACK)
            
            final_score = f"Score: {self.game.score}"
            final_score_width = draw.textlength(final_score, font=self.game.font)
            draw.text((display.width // 2 - final_score_width // 2, display.height // 2), 
                     final_score, font=self.game.font, fill=BLACK)
            
            restart = "Press R to restart"
            restart_width = draw.textlength(restart, font=self.game.font)
            draw.text((display.width // 2 - restart_width // 2, display.height // 2 + 30), 
                     restart, font=self.game.font, fill=BLACK)
    
    def render(self, fps=0, input_lag=0):
        """Render the current game state"""
        # Start with base image
        image = self.base_image.copy()
        draw = ImageDraw.Draw(image)
        
        # Draw dynamic elements
        self.draw_dynamic(draw, fps, input_lag)
        
        return image


class SimpleInputHandler:
    """Simpler, more responsive input handler without curses overhead"""
    
    def __init__(self):
        self.key_queue = deque(maxlen=20)
        self.running = True
        self.thread = None
        
    def start(self):
        """Start input thread using curses in separate thread"""
        self.thread = threading.Thread(target=self._curses_input_loop, daemon=True)
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
        
    def _curses_input_loop(self):
        """Simple curses input loop"""
        try:
            # Initialize curses in this thread
            stdscr = curses.initscr()
            curses.curs_set(0)
            curses.noecho()
            curses.cbreak()
            stdscr.nodelay(True)
            stdscr.keypad(True)
            
            # Key state tracking for repeat
            key_states = {}
            last_key_time = {}
            repeat_start = 0.2  # 200ms before repeat starts
            repeat_interval = 0.05  # 50ms repeat rate
            
            while self.running:
                current_time = time.time()
                key = stdscr.getch()
                
                if key != -1:
                    # Process key press
                    if key in [curses.KEY_UP, curses.KEY_DOWN, curses.KEY_LEFT, curses.KEY_RIGHT,
                              ord(' '), ord('r'), ord('R'), ord('q'), ord('Q')]:
                        
                        if key in key_states:
                            # Key is held - check for repeat
                            elapsed = current_time - last_key_time[key]
                            if elapsed > repeat_start:
                                # In repeat mode
                                repeat_elapsed = elapsed - repeat_start
                                if repeat_elapsed > repeat_interval:
                                    self.key_queue.append(key)
                                    last_key_time[key] = current_time - (repeat_start - repeat_interval)
                            else:
                                # Still in initial delay
                                self.key_queue.append(key)
                                last_key_time[key] = current_time
                        else:
                            # New key press
                            key_states[key] = True
                            last_key_time[key] = current_time
                            self.key_queue.append(key)
                else:
                    # No key pressed - clear states after a delay
                    to_remove = []
                    for key, press_time in list(last_key_time.items()):
                        if current_time - press_time > 0.3:  # 300ms delay
                            to_remove.append(key)
                    for key in to_remove:
                        if key in key_states:
                            del key_states[key]
                        if key in last_key_time:
                            del last_key_time[key]
                
                # Small sleep to prevent CPU hogging
                time.sleep(0.001)  # 1ms
                
        except Exception as e:
            print(f"Input error: {e}")
        finally:
            try:
                curses.nocbreak()
                curses.echo()
                curses.endwin()
            except:
                pass


def main():
    """Main game loop with optimized rendering"""
    
    print("\n" + "=" * 50)
    print("TETRIS - Optimized Version")
    print("Controls:")
    print("  Arrow keys: Move/Rotate/Fast drop")
    print("  Spacebar: Hard drop (instant)")
    print("  R: Restart game")
    print("  Q: Quit game")
    print("=" * 50)
    print("Using optimized rendering for better performance")
    print("=" * 50)
    
    # Create game
    game = Tetris(20, 10)
    game.new_figure()
    
    # Create optimized renderer
    renderer = OptimizedRenderer(game)
    
    # Start simple input handler
    input_handler = SimpleInputHandler()
    input_handler.start()
    
    # Game timing
    last_drop_time = time.time()
    drop_interval = 0.5
    
    # Performance tracking
    frame_times = deque(maxlen=30)
    fps = 0
    input_lag = 0
    input_timestamps = deque(maxlen=30)
    
    # Display update rate limiting
    last_display_update = 0
    display_update_interval = 1.0 / 30.0  # 30 FPS max for display
    
    try:
        while True:
            frame_start = time.time()
            
            # Process input immediately
            input_keys = input_handler.get_keys()
            
            if input_keys:
                input_timestamps.append(frame_start)
                current_time = time.time()
                
                for key in input_keys:
                    if key == ord(' '):  # Spacebar
                        if game.state == "start":
                            game.hard_drop()
                    elif key == ord('r') or key == ord('R'):
                        if game.state == "gameover" or game.state == "start":
                            game = Tetris(20, 10)
                            game.new_figure()
                            renderer = OptimizedRenderer(game)
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
            
            # Update game state
            current_time = time.time()
            
            # Update animation
            if game.state == "clearing":
                game.update_animation(current_time)
            
            # Auto-drop
            if game.state == "start" and current_time - last_drop_time > drop_interval:
                game.go_down()
                last_drop_time = current_time
                # Speed up gradually
                drop_interval = max(0.1, 0.5 - (game.score / 2000) * 0.1)
            
            # Calculate input lag
            if input_timestamps:
                avg_time = sum(input_timestamps) / len(input_timestamps)
                input_lag = (current_time - avg_time) * 1000  # ms
            
            # Calculate FPS
            frame_end = time.time()
            frame_time = frame_end - frame_start
            frame_times.append(frame_time)
            
            if len(frame_times) > 0:
                avg_frame_time = sum(frame_times) / len(frame_times)
                fps = 1.0 / avg_frame_time if avg_frame_time > 0 else 0
            
            # Update display at limited rate (to prevent display bottleneck)
            if current_time - last_display_update >= display_update_interval:
                # Render and update display
                image = renderer.render(fps, input_lag)
                
                # Update display (this is the slow part)
                try:
                    display.image(image)
                    display.show()
                except Exception as e:
                    print(f"Display error: {e}")
                
                last_display_update = current_time
            
            # Adaptive sleep to maintain ~60Hz game loop
            target_frame_time = 1.0 / 60.0  # 16.67ms
            elapsed = time.time() - frame_start
            
            if elapsed < target_frame_time:
                sleep_time = target_frame_time - elapsed
                if sleep_time > 0.002:  # Only sleep if we have >2ms
                    time.sleep(min(sleep_time, 0.01))  # Max 10ms sleep
            
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


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nGame interrupted")
    finally:
        print("\nCleaning up...")
        display.fill(1)
        display.show()
        print("Goodbye!")