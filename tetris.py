import random
import time
import curses
import board
import busio
import digitalio
from PIL import Image, ImageDraw, ImageFont
import adafruit_sharpmemorydisplay

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


class Tetris:
    def __init__(self, height, width):
        self.level = 1
        self.score = 0
        self.state = "start"
        self.height = height
        self.width = width
        self.x = 50  # X offset
        self.y = 30  # Y offset
        self.zoom = 8  # Block size
        self.figure = None
        
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
        except:
            self.font = ImageFont.load_default()

    def new_figure(self):
        self.figure = Figure(self.width // 2 - 2, 0)

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

    def draw(self, draw):
        """Draw game on display"""
        # Clear display
        draw.rectangle((0, 0, display.width, display.height), fill=BLACK)
        
        # Draw border
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
        
        # Draw score
        score_text = f"Score: {self.score}"
        draw.text((10, 10), score_text, font=self.font, fill=WHITE)
        
        # Draw level
        level_text = f"Level: {self.level}"
        draw.text((display.width - 80, 10), level_text, font=self.font, fill=WHITE)
        
        # Draw lines cleared indicator during animation
        if self.state == "clearing" and self.lines_to_clear:
            lines_text = f"Lines: {len(self.lines_to_clear)}"
            text_width = draw.textlength(lines_text, font=self.font)
            draw.text((display.width // 2 - text_width // 2, self.y + self.height * self.zoom + 10), 
                     lines_text, font=self.font, fill=WHITE)
        
        # Draw controls help
        if self.state == "start":
            controls = "←→:Move  ↑:Rotate  ↓:Fast"
            draw.text((display.width // 2 - 70, display.height - 20), 
                     controls, font=self.font, fill=WHITE)
        
        # Game over screen
        if self.state == "gameover":
            draw.rectangle((0, 0, display.width, display.height), fill=BLACK)
            game_over = "GAME OVER"
            draw.text((display.width // 2 - 40, display.height // 2 - 20), 
                     game_over, font=self.font, fill=WHITE)
            final_score = f"Score: {self.score}"
            draw.text((display.width // 2 - 30, display.height // 2), 
                     final_score, font=self.font, fill=WHITE)
            restart = "Press R to restart"
            draw.text((display.width // 2 - 50, display.height // 2 + 20), 
                     restart, font=self.font, fill=WHITE)

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


def curses_main(stdscr):
    # Setup curses
    curses.curs_set(0)
    stdscr.nodelay(True)
    stdscr.timeout(50)  # 50ms timeout for smoother animation
    
    # Create game
    game = Tetris(20, 10)
    game.new_figure()
    
    # Game loop
    counter = 0
    last_drop_time = time.time()
    drop_interval = 0.5  # Start with 0.5 seconds per drop
    last_animation_update = time.time()
    
    print("\n" + "=" * 50)
    print("TETRIS on Sharp Memory Display")
    print("Controls: Arrow keys to move, R to restart, Q to quit")
    print("=" * 50)
    
    try:
        while True:
            current_time = time.time()
            
            # Update animation if in clearing state
            if game.state == "clearing":
                game.update_animation(current_time)
            
            # Handle input (only if not in clearing state)
            if game.state != "clearing":
                key = stdscr.getch()
                
                if key != -1:
                    if key == curses.KEY_UP:
                        game.rotate()
                    elif key == curses.KEY_DOWN:
                        game.go_down()
                    elif key == curses.KEY_LEFT:
                        game.go_side(-1)
                    elif key == curses.KEY_RIGHT:
                        game.go_side(1)
                    elif key == ord('r') or key == ord('R'):
                        if game.state == "gameover":
                            game = Tetris(20, 10)
                            game.new_figure()
                    elif key == ord('q') or key == ord('Q'):
                        break
            
            # Auto-drop (only if in start state)
            if game.state == "start" and current_time - last_drop_time > drop_interval:
                game.go_down()
                last_drop_time = current_time
                # Speed up as score increases
                drop_interval = max(0.1, 0.5 - (game.score / 1000) * 0.1)
            
            # Update display
            image = Image.new("1", (display.width, display.height))
            draw = ImageDraw.Draw(image)
            game.draw(draw)
            display.image(image)
            display.show()
            
            counter += 1
            if counter % 100 == 0:
                # Increase level every 100 updates
                game.level = 1 + game.score // 100
            
            # Small delay for smoother animation
            time.sleep(0.01)
            
    except KeyboardInterrupt:
        pass
    
    except Exception as e:
        print(f"Error: {e}")
    
    finally:
        display.fill(1)
        display.show()


if __name__ == "__main__":
    try:
        curses.wrapper(curses_main)
    except KeyboardInterrupt:
        print("\nGame interrupted")
    finally:
        print("\nCleaning up...")
        display.fill(1)
        display.show()
        print("Goodbye!")