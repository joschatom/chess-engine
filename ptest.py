
# -*- coding: utf-8 -*-
# Automatically test the move generator against stockfish.

import subprocess

class Stockfish:
    
    def __init__(self, exe):
        self.ps = subprocess.run(exe)

    def set_position(self, pos):
        self.ps