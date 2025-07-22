# Rust Kafka

This portion of the code handles fetching data from the Open-Meteo Air Quality
API and storing it in a PostgreSQL database. It also makes use of Kafka to
stream data from the endpoint to be ingested in the database. 

A visual flow diagram will be coming soon

### Kafka Producer
The producer is what makes the fetch call to the API and sends it to the Kafka
topic. The producer contains 2 different API calls that can be made: historical
data, and recent data. The historical data is fetched from january 1, 2023 to
the current date at time of run for the endpoint. Within this, we fetch data in
92 day increments as that is what the API is able to handle sending data for. We
then update the dates at the end of the current loop to fetch in the next 92 day
increments. January 1 2023 is used as that is when the most consistent data is
available for the parameters returned by the API. When I was initially
investigating, I was hoping to get data from 2022 and that is what the API
mentions is availble, but this was just returning numerous NA values per
parameter, hence using January 1, 2023. 

The recent endpoint runs every hour, and fetches the previous hour of actual
data from the API, without any forecasts. With the API, even though we have
specified to get the past hour, it also returns forecast values for the next
hour(s). Therefore, we need additional parsing on that data to ensure we only
get the past timestamps and not forecasted ones. It then sends this single
observation to the Kafka producer. The data sent to the producer will always be
a vector of data, regardless of 1 or more observations

You can start with producer code with 
```bash
cargo run -- --broker localhost:9092 producer --mode recent #start the recent
producer
cargo run -- --broker localhost:9092 producer --mode historical #start the
historical producer
```

If running this in docker, you would update `localhost:9092` to `kafka:29092` or
wherever your kafka topic exposes in the container

### Kafka Consumer
The kafka consumer is fairly straightforward: take data from the kafka topic and
ingest it. The caveat here is when it comes to ingesting data into the database.
Since TimescaleDB is what we are using (a flavour of Postgres that is excellent
at handling time series data), we have access to standard Postgres functions
like `UNNEST()`. What this allows us to do is boot insert performance into the
database while writing all of our vector data at once. This saves so much
time and is incredibly more efficient than writing data row-by-row. 

The consumer can be started using
```bash
cargo run -- --broker localhost:9092 consumer
```

Likewise, if running this in a containerized environment, you would update
`localhost:9092` to `kafka:29092`

### Structural Decisions
#### Rust Traits
One thing that I wanted to explore myself with Rust is how to use traits to
improve code flow. Traits give us this logic by allowing us to define traits for
structs to maintain clean code and abstract logic away from parts of the
application. One instance is 

```rust
Impl From<RawAirQuality> for Vec<AirQualityHourly>
```

This is incredibly powerful, as it allows us to take the raw air quality data
and immediately convert it into a vector of air quality structs simply calling
`.into()`. This makes the application logic far cleaner in this case. While it
does make the `AirQuality` struct effectively useless, it gives more power and
clarity in the code. Any change that are made to the original structs can then
be updated in the trait.

Additionally, I abstract the API calls to traits, again to better familiarize
myself with them. In our case, we need to define an API struct that contains the
lat/long of the location to fetch data, as well as a `reqwest::Client`. Then, we
can define and implement the traits for this struct so that when passed to the
producer contain the functionality needed. Something special with this is that
it will be part of an async operation, so we need to add the `Send` and `Static`
types as part of the error return type.

Rust traits are incredibly powerful, and I may look to incorporate more as time
goes on, however it will depend on what changes are made as this project should
maintain clarity as well.

### Deplying in Cloud
If the infrastructure is already set up for us, great! But, when running this, we need to ensure that we include a `config.toml` that contains the lat/long information, alond with the database url. For example

```toml
[location]
latitude = 50.0000
longitude = -50.0000

[database]
db_url = "postgres://your_user:your_pass@your_address:5432/your_db"
```

We need this config information for the application to be able to access the air quality data for your area. Now, this was not developed at first with cloud solutions in mind, so I will try and come up with a more long term solution for this. Regardless, this needs to be in the working directory of the Compute Engine instance

### Additional Notes
Something else I want to include is that this is a first iteration of my
project. I have completed it to the point of initial scoped design, but as time
goes on I may have deisgn or structural changes or I may catch errors with
certain pieces of code as well. As I said in the main README file, I want to
iterate and improve on this project as time goes on and want it to be something
that I can improve as time goes on
