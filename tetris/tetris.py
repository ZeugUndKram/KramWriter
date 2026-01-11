import random
import time

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
        self.last_positions = []
        self.last_x = x
        self.last_y = y

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
    def __init__(self, height, width, display_width=400, display_height=240):
        self.level = 1
        self.score = 0
        self.state = "start"
        self.height = height
        self.width = width
        self.display_width = display_width
        self.display_height = display_height
        
        # Block size - make it smaller to fit better
        self.zoom = 10  # Smaller blocks
        self.preview_zoom = 6  # Smaller zoom for preview
        
        # Calculate centered position for the game board
        field_width = width * self.zoom
        field_height = height * self.zoom
        
        # Center the playing field - simple calculation
        self.x = (display_width - field_width) // 2
        self.y = (display_height - field_height) // 2
        
        # Preview position (to the right of main field, but centered vertically)
        self.preview_x = self.x + field_width + 20
        self.preview_y = self.y
        
        self.figure = None
        self.next_figure = None
        
        # Initialize field
        self.field = [[0 for _ in range(width)] for _ in range(height)]
        
        # Line clearing animation
        self.lines_to_clear = []
        self.line_clear_timer = 0
        self.line_blink_state = True
        self.line_clear_duration = 0.5
        self.line_blink_interval = 0.1
        
        # Track changes for optimized rendering
        self.field_changed = True
        self.score_changed = True
        self.level_changed = True

    def new_figure(self):
        # If we have a next figure, use it as current
        if self.next_figure:
            self.figure = self.next_figure
            self.figure.x = self.width // 2 - 2
            self.figure.y = 0
            self.figure.last_x = self.figure.x
            self.figure.last_y = self.figure.y
            self.figure.last_positions = []
        else:
            self.figure = Figure(self.width // 2 - 2, 0)
        
        # Create new next figure for preview
        self.next_figure = Figure(0, 0)
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
            if lines_cleared == 1:
                self.score += 100 * self.level
            elif lines_cleared == 2:
                self.score += 300 * self.level
            elif lines_cleared == 3:
                self.score += 500 * self.level
            elif lines_cleared >= 4:
                self.score += 800 * self.level  # Tetris!
            self.score_changed = True
        
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
        else:
            self.field_changed = True

    def go_side(self, dx):
        old_x = self.figure.x
        self.figure.x += dx
        if self.intersects():
            self.figure.x = old_x
        else:
            self.field_changed = True

    def rotate(self):
        old_rotation = self.figure.rotation
        self.figure.rotate()
        if self.intersects():
            self.figure.rotation = old_rotation
        else:
            self.field_changed = True

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