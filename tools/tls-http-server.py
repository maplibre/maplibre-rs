from http.server import HTTPServer, SimpleHTTPRequestHandler
import ssl
import sys
import socketserver

class Handler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        SimpleHTTPRequestHandler.end_headers(self)

if __name__ == '__main__':
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(('0.0.0.0', 5555), Handler) as httpd:
        #httpd.socket = ssl.wrap_socket(httpd.socket, certfile='tools/server.pem', server_side=True)
        httpd.serve_forever()
