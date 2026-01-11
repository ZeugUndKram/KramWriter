import threading
import time
import curses
from collections import deque

class InputHandler:
    def __init__(self):
        self.key_queue = deque(maxlen=50)
        self.running = False
        self.thread = None
        
        # Key repeat settings
        self.repeat_initial = 0.15  # 150ms before repeat starts
        self.repeat_interval = 0.05  # 50ms repeat rate
        
        # Key state tracking
        self.key_states = {}
        self.last_key_time = {}
        
    def start(self):
        """Start input thread"""
        self.running = True
        self.thread = threading.Thread(target=self._input_loop, daemon=True)
        self.thread.start()
        print("Input handler started")
        
    def stop(self):
        """Stop input thread"""
        self.running = False
        if self.thread:
            self.thread.join(timeout=0.5)
        print("Input handler stopped")
        
    def get_keys(self):
        """Get all pending keys (non-blocking)"""
        keys = []
        while self.key_queue:
            keys.append(self.key_queue.popleft())
        return keys
        
    def _input_loop(self):
        """Main input loop - runs in separate thread"""
        try:
            # Initialize curses in this thread
            stdscr = curses.initscr()
            curses.curs_set(0)
            curses.noecho()
            curses.cbreak()
            stdscr.nodelay(True)  # Non-blocking
            stdscr.keypad(True)   # Enable special keys
            
            # Set a very short timeout for getch
            stdscr.timeout(1)  # 1ms timeout
            
            last_cleanup = time.time()
            
            while self.running:
                current_time = time.time()
                
                # Read all available keys
                while True:
                    key = stdscr.getch()
                    if key == -1:  # No more keys
                        break
                    
                    # Process valid keys
                    if key in [curses.KEY_UP, curses.KEY_DOWN, curses.KEY_LEFT, curses.KEY_RIGHT,
                              ord(' '), ord('r'), ord('R'), ord('q'), ord('Q')]:
                        
                        # Handle key repeat
                        if key in self.key_states:
                            elapsed = current_time - self.last_key_time[key]
                            if elapsed > self.repeat_initial:
                                # In repeat mode
                                repeat_elapsed = elapsed - self.repeat_initial
                                if repeat_elapsed > self.repeat_interval:
                                    self.key_queue.append(key)
                                    self.last_key_time[key] = current_time - (self.repeat_initial - self.repeat_interval)
                            else:
                                # Still in initial delay, send once
                                self.key_queue.append(key)
                                self.last_key_time[key] = current_time
                        else:
                            # New key press
                            self.key_states[key] = True
                            self.last_key_time[key] = current_time
                            self.key_queue.append(key)
                
                # Clean up old key states every second
                if current_time - last_cleanup > 1.0:
                    self._cleanup_key_states(current_time)
                    last_cleanup = current_time
                
                # Tiny sleep to prevent CPU hogging
                time.sleep(0.001)  # 1ms
                
        except Exception as e:
            print(f"Input error: {e}")
        finally:
            # Clean up curses
            try:
                curses.nocbreak()
                curses.echo()
                curses.endwin()
            except:
                pass
    
    def _cleanup_key_states(self, current_time):
        """Remove keys that haven't been pressed recently"""
        to_remove = []
        for key, press_time in list(self.last_key_time.items()):
            if current_time - press_time > 0.5:  # 500ms threshold
                to_remove.append(key)
        
        for key in to_remove:
            if key in self.key_states:
                del self.key_states[key]
            if key in self.last_key_time:
                del self.last_key_time[key]