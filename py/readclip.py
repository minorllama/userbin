#!/bin/env python3

import os
import sys
import tkinter as tk


self_verbose = True

def logger(a):
    if self_verbose:
        print(a)

def safedict(es):
    hashed = dict(es)
    assert len(es) == len(hashed)
    return hashed


class Cfg:
    def __init__(self, vals, delim="="):
        flags = [e for e in vals if e[0] in "-"]
        self.values = [e for e in vals if not e[0] in "-"]
        self.keys = safedict([e.split(delim) for e in flags if e.find(delim) != -1])
        self.flags = [e.split(delim)[0] for e in flags]
        self.usage = list() 
    def __getitem__(self, k):
        if type(k)==int:
            return self.values[k] if k < len(self.values) else None
        raise NotImplementedError("__geitem__")
    @staticmethod
    def sys():
        return Cfg(sys.argv[1:]) 

class ClipboardAccess(tk.Tk):
    def __init__(self):
        super().__init__()
        self.verbose = True
    def fromclip(self):
        #self.clipboard_clear()
        txt = self.clipboard_get()
        return txt
    def toclip(self, txt, hold=True):
        self.clipboard_clear()
        self.clipboard_append(txt)
        self.update() # keep on the clipboard after the window is closed
        if hold: 
            return input("..on clipbord..")

def clipper(cfg):
    clip = ClipboardAccess()
    clip.withdraw()
    if "-env" in cfg.keys:
        txt = os.environ[cfg.keys["-env"]] 
    else:
        txt = " ".join(cfg.values)
    if "-from" in cfg.flags:
        try:
            txt = clip.fromclip()
            print(txt)
        except Exception as e:
            logger("__empty__:{0}".format(e))
    else:
        assert "-to" in cfg.flags
        clip.toclip(txt, hold = "-hold" in cfg.flags)
    if "-v" in cfg.flags:
        logger("__clipped__:{0}".format(txt))
    else:
        logger("__clipped__:{0}...".format(txt[0:15]))
                
if __name__ == "__main__":
    clipper(Cfg.sys())
