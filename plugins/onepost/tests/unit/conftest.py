import os
import sys

_LIB_PATH = os.path.join(os.path.dirname(__file__), "..", "..", "lib")
if _LIB_PATH not in sys.path:
    sys.path.insert(0, os.path.abspath(_LIB_PATH))
