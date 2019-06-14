```rust,skt-verify-ok
use manual::*;
use ark_auth::server::route::auth;
use ark_auth::server::route::user;
use ark_auth::server::route::key;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```

```rust,skt-verify-bad-request
use manual::*;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```

```rust,skt-verify-forbidden
use manual::*;
use ark_auth::server::route::auth;

fn main() {{
    let client = reqwest::Client::new();

    {}
}}
```