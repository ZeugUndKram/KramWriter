#include "display_client.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <termios.h>
#include <unistd.h>

#define NUM_ITEMS 4
#define ARROW_WIDTH 30
#define ARROW_HEIGHT 30

// Menu items
const char *menu_items[] = {
    "NEW FILE",
    "OPEN FILE",
    "SETTINGS",
    "CREDITS"
};

// Terminal raw mode für sofortige Tasteneingabe
struct termios orig_termios;

void disable_raw_mode() {
    tcsetattr(STDIN_FILENO, TCSAFLUSH, &orig_termios);
}

void enable_raw_mode() {
    tcgetattr(STDIN_FILENO, &orig_termios);
    atexit(disable_raw_mode);
    
    struct termios raw = orig_termios;
    raw.c_lflag &= ~(ECHO | ICANON);
    raw.c_cc[VMIN] = 0;
    raw.c_cc[VTIME] = 1;
    
    tcsetattr(STDIN_FILENO, TCSAFLUSH, &raw);
}

// Zeichne einen Pfeil im Buffer
void draw_arrow(uint8_t *buffer, int x, int y) {
    // Einfacher Pfeil: >
    // Größerer Pfeil aus Linien
    int size = 15;
    
    // Pfeil spitze nach rechts
    display_draw_line_buffer(buffer, x, y, x + size, y + size, true);
    display_draw_line_buffer(buffer, x, y + size*2, x + size, y + size, true);
    
    // Dickerer Pfeil - mehrere Linien
    display_draw_line_buffer(buffer, x+1, y, x + size+1, y + size, true);
    display_draw_line_buffer(buffer, x+1, y + size*2, x + size+1, y + size, true);
    display_draw_line_buffer(buffer, x+2, y, x + size+2, y + size, true);
    display_draw_line_buffer(buffer, x+2, y + size*2, x + size+2, y + size, true);
}

// Zeichne einen einzelnen Buchstaben (sehr simpel, nur große Buchstaben)
void draw_char_simple(uint8_t *buffer, int x, int y, char c, int size) {
    // Vereinfachte Buchstaben - nur Striche
    int h = size;
    int w = size / 2;
    
    switch(c) {
        case 'N':
            display_draw_line_buffer(buffer, x, y, x, y+h, true);
            display_draw_line_buffer(buffer, x, y, x+w, y+h, true);
            display_draw_line_buffer(buffer, x+w, y, x+w, y+h, true);
            break;
        case 'E':
            display_draw_line_buffer(buffer, x, y, x, y+h, true);
            display_draw_line_buffer(buffer, x, y, x+w, y, true);
            display_draw_line_buffer(buffer, x, y+h/2, x+w-5, y+h/2, true);
            display_draw_line_buffer(buffer, x, y+h, x+w, y+h, true);
            break;
        case 'W':
            display_draw_line_buffer(buffer, x, y, x+w/3, y+h, true);
            display_draw_line_buffer(buffer, x+w/3, y+h, x+w/2, y+h/2, true);
            display_draw_line_buffer(buffer, x+w/2, y+h/2, x+2*w/3, y+h, true);
            display_draw_line_buffer(buffer, x+2*w/3, y+h, x+w, y, true);
            break;
        case 'F':
            display_draw_line_buffer(buffer, x, y, x, y+h, true);
            display_draw_line_buffer(buffer, x, y, x+w, y, true);
            display_draw_line_buffer(buffer, x, y+h/2, x+w-5, y+h/2, true);
            break;
        case 'I':
            display_draw_line_buffer(buffer, x, y, x+w, y, true);
            display_draw_line_buffer(buffer, x+w/2, y, x+w/2, y+h, true);
            display_draw_line_buffer(buffer, x, y+h, x+w, y+h, true);
            break;
        case 'L':
            display_draw_line_buffer(buffer, x, y, x, y+h, true);
            display_draw_line_buffer(buffer, x, y+h, x+w, y+h, true);
            break;
        case 'O':
            display_draw_rect_buffer(buffer, x, y, w, h, true, false);
            break;
        case 'P':
            display_draw_line_buffer(buffer, x, y, x, y+h, true);
            display_draw_line_buffer(buffer, x, y, x+w, y, true);
            display_draw_line_buffer(buffer, x+w, y, x+w, y+h/2, true);
            display_draw_line_buffer(buffer, x, y+h/2, x+w, y+h/2, true);
            break;
        case 'S':
            display_draw_line_buffer(buffer, x, y, x+w, y, true);
            display_draw_line_buffer(buffer, x, y, x, y+h/2, true);
            display_draw_line_buffer(buffer, x, y+h/2, x+w, y+h/2, true);
            display_draw_line_buffer(buffer, x+w, y+h/2, x+w, y+h, true);
            display_draw_line_buffer(buffer, x, y+h, x+w, y+h, true);
            break;
        case 'T':
            display_draw_line_buffer(buffer, x, y, x+w, y, true);
            display_draw_line_buffer(buffer, x+w/2, y, x+w/2, y+h, true);
            break;
        case 'G':
            display_draw_rect_buffer(buffer, x, y, w, h, true, false);
            display_draw_line_buffer(buffer, x+w/2, y+h/2, x+w, y+h/2, true);
            display_draw_line_buffer(buffer, x+w, y+h/2, x+w, y+h, true);
            break;
        case 'C':
            display_draw_line_buffer(buffer, x, y, x+w, y, true);
            display_draw_line_buffer(buffer, x, y, x, y+h, true);
            display_draw_line_buffer(buffer, x, y+h, x+w, y+h, true);
            break;
        case 'R':
            display_draw_line_buffer(buffer, x, y, x, y+h, true);
            display_draw_line_buffer(buffer, x, y, x+w, y, true);
            display_draw_line_buffer(buffer, x+w, y, x+w, y+h/2, true);
            display_draw_line_buffer(buffer, x, y+h/2, x+w, y+h/2, true);
            display_draw_line_buffer(buffer, x+w/2, y+h/2, x+w, y+h, true);
            break;
        case 'D':
            display_draw_line_buffer(buffer, x, y, x, y+h, true);
            display_draw_line_buffer(buffer, x, y, x+w-5, y+5, true);
            display_draw_line_buffer(buffer, x+w-5, y+5, x+w, y+h/2, true);
            display_draw_line_buffer(buffer, x+w, y+h/2, x+w-5, y+h-5, true);
            display_draw_line_buffer(buffer, x+w-5, y+h-5, x, y+h, true);
            break;
        case ' ':
            // Leerzeichen - nichts zeichnen
            break;
        default:
            // Unbekanntes Zeichen - Rechteck
            display_draw_rect_buffer(buffer, x+w/4, y+h/4, w/2, h/2, true, true);
            break;
    }
}

// Zeichne Text (ohne Font-Dateien, rein programmatisch)
void draw_text_simple(uint8_t *buffer, int x, int y, const char *text, int size) {
    int offset = 0;
    int spacing = size / 2 + 5;
    
    for(int i = 0; text[i] != '\0'; i++) {
        draw_char_simple(buffer, x + offset, y, text[i], size);
        offset += spacing;
    }
}

void display_menu_frame(int selected) {
    uint8_t *buffer = display_create_buffer();
    
    // Berechne Positionen
    int item_height = 45;
    int total_height = NUM_ITEMS * item_height;
    int start_y = (DISPLAY_HEIGHT - total_height) / 2;
    
    // Zeichne jeden Menü-Eintrag
    for(int i = 0; i < NUM_ITEMS; i++) {
        int y_pos = start_y + (i * item_height);
        
        // Berechne ungefähre Textbreite für Zentrierung
        int text_len = strlen(menu_items[i]);
        int char_width = 20;  // Ungefähre Breite pro Buchstabe
        int text_width = text_len * char_width;
        int x_pos = (DISPLAY_WIDTH - text_width) / 2;
        
        // Zeichne Text
        draw_text_simple(buffer, x_pos, y_pos, menu_items[i], 30);
        
        // Zeichne Pfeil wenn ausgewählt
        if(i == selected) {
            int arrow_x = x_pos - 40;
            int arrow_y = y_pos + 7;
            draw_arrow(buffer, arrow_x, arrow_y);
        }
    }
    
    // Sende zum Display
    display_raw_buffer(buffer, DISPLAY_BUFFER_SIZE);
    display_free_buffer(buffer);
}

int main() {
    printf("Connecting to display server...\n");
    
    if(display_init() < 0) {
        fprintf(stderr, "Failed to connect! Is display_server.py running?\n");
        return 1;
    }
    
    printf("Menu started!\n");
    printf("Use UP/DOWN arrow keys, ENTER to select, Q to quit\n");
    
    int selected = 0;
    enable_raw_mode();
    
    // Initial display
    display_menu_frame(selected);
    
    while(1) {
        char c = getchar();
        
        if(c == '\033') {  // ESC sequence
            getchar();  // Skip '['
            switch(getchar()) {
                case 'A':  // Up arrow
                    selected = (selected - 1 + NUM_ITEMS) % NUM_ITEMS;
                    display_menu_frame(selected);
                    printf("↑ Selected: %s\n", menu_items[selected]);
                    break;
                case 'B':  // Down arrow
                    selected = (selected + 1) % NUM_ITEMS;
                    display_menu_frame(selected);
                    printf("↓ Selected: %s\n", menu_items[selected]);
                    break;
            }
        }
        else if(c == '\r' || c == '\n') {  // Enter
            printf("✓ Selected: %s\n", menu_items[selected]);
            
            // Führe Aktion aus basierend auf Auswahl
            switch(selected) {
                case 0:
                    printf("Opening NEW FILE...\n");
                    // Hier dein Code für NEW FILE
                    break;
                case 1:
                    printf("Opening OPEN FILE...\n");
                    // Hier dein Code für OPEN FILE
                    break;
                case 2:
                    printf("Opening SETTINGS...\n");
                    // Hier dein Code für SETTINGS
                    break;
                case 3:
                    printf("Opening CREDITS...\n");
                    // Hier dein Code für CREDITS
                    break;
            }
            break;
        }
        else if(c == 'q' || c == 'Q') {  // Quit
            printf("Exiting menu...\n");
            break;
        }
    }
    
    disable_raw_mode();
    display_clear();
    display_close();
    
    return 0;
}