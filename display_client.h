#ifndef DISPLAY_CLIENT_H
#define DISPLAY_CLIENT_H

#include <stdint.h>
#include <stdbool.h>

#define DISPLAY_WIDTH 400
#define DISPLAY_HEIGHT 240
#define DISPLAY_BUFFER_SIZE 12000

int display_init(void);
void display_close(void);

bool display_clear(void);
bool display_text(int x, int y, int font_size, const char *text);
bool display_image(const char *path, int x, int y);
bool display_rect(int x, int y, int w, int h, bool fill);
bool display_raw_buffer(const uint8_t *buffer, size_t size);

uint8_t* display_create_buffer(void);
void display_free_buffer(uint8_t *buffer);
void display_set_pixel(uint8_t *buffer, int x, int y, bool black);
bool display_get_pixel(const uint8_t *buffer, int x, int y);
void display_draw_rect_buffer(uint8_t *buffer, int x, int y, int w, int h, bool black, bool fill);
void display_draw_line_buffer(uint8_t *buffer, int x0, int y0, int x1, int y1, bool black);

#endif