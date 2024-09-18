# Beach ball animation
## :link: Webpage
[View the animation in your web browser](https://trichoplax.github.io/grey-sphere-explanatory-animations/beachball).

## :wrench: Development
See the [main readme file](../../readme.md) for how to view the pages in a local development environment.

If you modify the Rust source code, you can compile it to WASM by entering the following in the terminal from the `projects/beachball` directory:

```
cargo build --release --target=wasm32-unknown-unknown
```

You can then add this WASM to the `web/beachball` directory ready for viewing locally by entering the following in the terminal from the `projects/beachball` directory:

```
wasm-bindgen target/wasm32-unknown-unknown/release/beachball.wasm --out-dir ../../web/beachball --target web --no-typescript
```

The resulting animation should then show up in your web browser when viewed locally.
