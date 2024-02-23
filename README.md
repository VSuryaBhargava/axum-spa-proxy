# axum-spa-proxy
This is a simple server which serves static files for a single page application and proxies api calls to a backend server.

A json config is needed to start the server, below is a sample json

```JS
﻿﻿{
  "port": 3001,
  "httpsConfig": {                                 // `httpsConfig` is optional
    "key": "./.cert/server.key",
    "cert": "./.cert/server.crt"
  },
  "fileServer": {                                  // `fileServer` is optional. File server is the fallback so if none of the proxies match the route
    "filePath": "./public",                        // the server will try to serve a file at the path. If no file is found at that path the server
    "routePath": "/",                              // will serve the `fallbackFile`
    "fallbackFile": "./public/index.html"          // root html file.
  },
  "proxies": [                                     // `proxies` is optional. Multiple proxies can be provided
    {
      "route": "/api/*path",
      "target": "https://some-secure-domain"       //  `localhost:port/api/any/path` calls `https://some-secure-domain/api/any/path` 
    },
    {
      "route": "/some-path",                       //  `localhost:port/some-path` calls `http://some-domain/some-path` 
      "target": "http://some-domain"               //  `localhost:port/some-other-path` doesn't call `http://some-domain/some-other-path` 
    }
  ]
}
```
