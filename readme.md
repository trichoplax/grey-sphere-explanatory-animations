# Plain grey spheres explanatory animations
## :link: Webpage
[View the animations in your web browser](https://trichoplax.github.io/grey-sphere-explanatory-animations).

## :wrench: Development
### Local server
To access the webpage locally for development, you will need an http server as the `.wasm` file will be blocked from loading if the page is opened directly. For example, if you have Python installed, you can enter the following in the terminal from the `web` directory:

```text
python -m http.server 8080
```

You can then view the page at `localhost:8080` in your browser.

### Compiling Rust to WASM
There are 4 links on the homepage, each to a different animation which opens in a new tab. Each of these has a separate WASM file, which is compiled from a separate Rust project. The projects can be found in the [projects](projects) directory.

Instructions for compiling the Rust to WASM can be found in each of the project directories.
