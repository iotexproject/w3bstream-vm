# risc0-server

## setup
### install diesel

``` shell
cargo install diesel_cli --no-default-features --features postgres
```

### build release

``` shell
cargo build --release
```

### configure database
modify `.env` file

``` shell
DATABASE_URL=postgres://test_user:test_passwd@127.0.0.1:5432/test?sslmode=disable
```

### migrate database

``` shell
diesel setup

diesel migration generate risc0-server

diesel migration run
```

### run risc0 rpc sever

``` shell
./target/release/risc0server
```