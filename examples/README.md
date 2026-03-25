# Examples

These examples assume a php-fpm server is listening on `127.0.0.1:9000` and that the
repository is mounted into the container so php-fpm can read the PHP fixtures under
`tests/php`.

One way to start the same container used by this repository's CI is:

```shell
docker run --rm --name php-fpm -v "$PWD:$PWD" -p 9000:9000 php:7.1.30-fpm -c /usr/local/etc/php/php.ini-development
```

From the repository root, run the examples as follows:

1. `tokio_short_connection`

   This example uses Tokio short-connection mode, sends a single FastCGI request to `tests/php/index.php`, and prints the full stdout payload returned by php-fpm.

   Run it with:

   ```shell
   cargo run --example tokio_short_connection --features runtime-tokio
   ```

2. `tokio_keep_alive`

   This example uses Tokio keep-alive mode, reuses the same FastCGI connection three times, and prints the stdout payload from each request to `tests/php/index.php`.

   Run it with:

   ```shell
   cargo run --example tokio_keep_alive --features runtime-tokio
   ```

3. `tokio_stream_response`

   This example uses Tokio streaming mode, requests `tests/php/big-response.php`, prints each stdout chunk as it arrives, and then summarizes the total bytes received along with a short preview of the response.

   Run it with:

   ```shell
   cargo run --example tokio_stream_response --features runtime-tokio
   ```

4. `smol_short_connection`

   This example uses Smol short-connection mode, sends a single FastCGI request to `tests/php/index.php`, and prints the full stdout payload returned by php-fpm.

   Run it with:

   ```shell
   cargo run --example smol_short_connection --features runtime-smol
   ```

5. `smol_keep_alive`

   This example uses Smol keep-alive mode, reuses the same FastCGI connection three times, and prints the stdout payload from each request to `tests/php/index.php`.

   Run it with:

   ```shell
   cargo run --example smol_keep_alive --features runtime-smol
   ```

6. `axum_proxy`

   This example starts an Axum server on `127.0.0.1:3000`, converts incoming HTTP requests into FastCGI requests, forwards them to php-fpm, and converts the FastCGI response back into an HTTP response. The route `/` maps to `tests/php/index.php`; other request paths map to PHP files under `tests/php`. Non-existent scripts and obvious path traversal attempts return `404` before contacting php-fpm. The script `tests/php/post.php` intentionally throws an exception, so this example may print FastCGI stderr to stderr while still returning the HTTP response body.

   Run it with:

   ```shell
   cargo run --example axum_proxy --features http,runtime-tokio
   ```

   Example manual checks:

   ```shell
   curl -i http://127.0.0.1:3000/
   curl -i -X POST 'http://127.0.0.1:3000/post.php?g1=1&g2=2' \
     -H 'content-type: application/x-www-form-urlencoded' \
     --data 'p1=3&p2=4'
   curl -i http://127.0.0.1:3000/body-size.php --data-binary @Cargo.toml
   curl -i http://127.0.0.1:3000/not-found.php
   curl --path-as-is -i http://127.0.0.1:3000/../README.md
   ```
