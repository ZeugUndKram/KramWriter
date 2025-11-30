import tkinter as tk

win = tk.Tk()
win.geometry("400x240")
win.resizable(False, False)

c = tk.Canvas(win, width=400, height=240, bg="grey")
c.pack()

y = 240 - 16
c.create_line(0, y, 400, y, width=1)

win.mainloop()
