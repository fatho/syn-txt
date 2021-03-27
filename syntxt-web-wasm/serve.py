#!/usr/bin/env python3

import os
from http.server import HTTPServer, SimpleHTTPRequestHandler

ROOT = os.path.dirname(os.path.abspath(__file__))
MONACO_EDITOR_SRC = os.environ['MONACO_EDITOR_SRC']

class RequestHandler(SimpleHTTPRequestHandler):

    def do_GET(self):
        MONACO_ROOT = '/monaco-editor/'

        if self.path.startswith(MONACO_ROOT):
            self.path = self.path[len(MONACO_ROOT):]
            self.directory = MONACO_EDITOR_SRC
        elif self.path in ('/logo.png', '/favicon.ico'):
            self.directory = os.path.join(ROOT, '../doc')
        else:
            self.directory = ROOT
        super().do_GET()


if __name__ == '__main__':
    with HTTPServer(('localhost', 8080), RequestHandler) as server:
        print("Go to http://127.0.0.1:8080")
        server.serve_forever()
