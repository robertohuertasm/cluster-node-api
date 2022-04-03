# Cluster Node API

This is simple REST API that emulates the management of nodes of computing clusters.

## Setting up the project

### Rust

The project is written in [Rust](https://www.rust-lang.org/). Follow [these instructions](https://www.rust-lang.org/tools/install) to install it.

Once you have installed [Rust](https://www.rust-lang.org/), please run this command to install [cargo-make](https://github.com/sagiegurari/cargo-make), which is just a task runner that will help us to set up the database and run the API.

```sh
cargo install --force cargo-make
```

I chose to use [Rust](https://www.rust-lang.org/) because it's a language I feel comfortable with. Considering that this was just an exercise, I didn't have into account other kind of considerations like, for instance, the fact that the rest of the team might not be comfortable using/learning it, the complexity of hiring new Rust developers...

### PostgreSQL

I've used [PostgreSQL](https://www.postgresql.org/) as a database to emulate the state of the cluster. In order to ease the development, I've leveraged [Docker](https://www.docker.com/) to run the database but you can use your own instance if you want.

Note that the database connection is already configured in the `.env` file. If you're using your own Postgres instance, you can change the `DATABASE_URL` value in the `.env` file or just set it as an environment variable.

In this case, I chose to use [PostgreSQL](https://www.postgresql.org/) because it's quite flexible to work with but I guess there were many options here that could have also worked well. In the end, given that there were other options I just chose the one I felt comfortable with.


## Starting the API

If you want to go with the happy path, just run this:

```sh
## This will start a docker postges container, create the database and run the SQL migrations to set it up
makers db-init
## This will start the API in the port 8080 and in release mode
makers start
```

Note that you can change the port by setting the `PORT` environment variable. Remember that you can use the `.env` file to set the environment variables, too. We're leveraging the [dotenv crate](https://docs.rs/dotenv/latest/dotenv/).

## Development mode

If you would like to run the API while develping it, you can run this:

```sh
makers dev
```

This will start the API in watch mode, so it will reload on eveyy file change and it will show the logs in colour in your terminal.


## Testing

I tried to cover most of the components of the API but given the time constraints I didn't have time to cover all the cases.

You can run the tests by executing the following command:

```sh
cargo test
```

## Tracing

I normally use to instrument the code I write so I can understand what's going on. The API has been instrumented using the [tracing](https://docs.rs/tracing/0.1.31/tracing/) crate.

In order to enable it, you can set the env var `RUST_LOG` to `cluster_node_api=debug`. You can change the level of log by changing the `debug` to `trace`, `info`, `warn` or `error`. Likewise, if you want to get information about other crates, just remove the `cluster_node_api` from the env var: `RUST_LOG=debug`.

## Architecture

The idea was to provide a clean architecture so we have a clear separation of concerns while keeping loose coupling between the different components. This generally has the side effect of simplifying the testing, too.

The entry point of the API is the `main.rs` file. Here, we set up all the dependencies and start the api.

We have 3 separated layers:

1. Infrastructure: Responsible for the data storage, authorization middleware and api controllers.
2. Application: Responsible for orchestration logic.
3. Domain: Responsible for the high level data objects that we use.

The direction of coupling goes from 1 to 3.

In order to maintain the loose coupling, we're leveraging [traits](https://doc.rust-lang.org/book/ch10-02-traits.html) to define the interface between the different layers.

Take into account, that given that the majority of the API are CRUD operations, the `application` layer is quite minimal. Indeed, I've just created a service for the `commands API` (which I called `operation`) just for the sake of the example, but normally, adding this layer when there's no need of business logic just adds extra complexity.

![architecture](/docs/architecture.png)

## API endpoints

The API has several endpoints:

- /healh: GET. This endpoint is used to check if the API is running.
- /v1/features: GET
- /v1/clusters: GET, POST, PUT and DELETE
- /v1/nodes: GET, POST, PUT and DELETE. The GET endpoint accepts a query param called `name` to filter the nodes by node name or cluster name.
- /v1/operations/poweron: POST
- /v1/operations/poweroff: POST
- /v1/operations/reboot: POST

You can find more details about this endpoints in the files located in the [http folder](/http).

If you use [vscode](https://code.visualstudio.com/),and have the [REST Client extension](https://marketplace.visualstudio.com/items?itemName=humao.rest-client) installed, you can use it to test the API with the previous files.

## Authorization

Note that the only endpoints that are accesible without any kind of authorization are the `/health` and `/v1/features` endpoints.

The rest of endpoints need a simple token. The token is passed as a header with the name `Authorization` and the value is `Bearer im_a_valid_user`.

If this token is not present or is invalid, the API will return a 401 error.

## Open API

Aside from other improvements, I could imagine expanding the documentation of this API by providing [OPEN API](https://github.com/OAI/OpenAPI-Specification/) support.
