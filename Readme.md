# json_env

`json_env` is [dotenv](https://github.com/motdotla/dotenv), but with JSON.
It loads an environment variables from a JSON file (`.env.json` per default) and starts a subprocess with them. 
Storing configuration in the environment separate from code is based on [The Twelve-Factor](http://12factor.net/config) App methodology.

## How to install

With homebrew:
```shell
$ brew tap brodo/json_env
$ brew install json_env
```

With NPM
```shell
$ npm i -g @brodo/json_env
```

With cargo:
```shell
$ cargo install json_env
```

Or download the binaries for your platform on the [releases page](https://github.com/brodo/json_env/releases/) and
put them in your $PATH.

## How to use

Just run json_env with any program as a parameter: 
```shell
$ json_env my_program
```

Additional command line arguments that are passed to `json_env` are forwarded to the child process:
```shell
$ json_env echo "Test"

Test
```

### Example
.env.json:
```json
{
    "NODE_ENV": "DEV",
    "MY_USER": "Carl",
    "NUM_USERS": 10,
    "nested": {
        "hello": "world",
        "boo": "far"
    }
}
```

Shell:
```shell
$ json_env env

MY_USER=Carl
NODE_ENV=DEV
NUM_USERS=10
nested={"boo":"far","hello":"world"}
[...]
```


### Environment Variable Expansion

You can include existing environment variables in your env file to expand them:

.env.json:
```json
{
  "MY_VAR": "$FOO",
  "MY_OTHER_VAR": "User:$USER"
}
```

Shell:
```shell
$ json_env -e env
FOO=Bar
USER=Carl
MY_VAR=Bar
MY_OTHER_VAR=User:Carl
[...]
```


### JSON Path support

There are some use cases where you already have environment variables defined
in a JSON file but not at the root level. Take this 
[Azure Function local.settings.json file](https://learn.microsoft.com/en-us/azure/azure-functions/functions-develop-local#local-settings-file)
for example:

```json

{
  "IsEncrypted": false,
  "Values": {
    "FUNCTIONS_WORKER_RUNTIME": "<language worker>",
    "AzureWebJobsStorage": "<connection-string>",
    "MyBindingConnection": "<binding-connection-string>",
    "AzureWebJobs.HttpExample.Disabled": "true"
  },
  "Host": {
    "LocalHttpPort": 7071,
    "CORS": "*",
    "CORSCredentials": false
  },
  "ConnectionStrings": {
    "SQLConnectionString": "<sqlclient-connection-string>"
  }
}
```

The `Values` property contains the environment variables we are interested in.
You can use this file to run `app.js` with the environment variables defined in `Values`
by providing the [JSON Path](https://docs.rs/jsonpath-rust/latest/jsonpath_rust/) `$.Values``:

```shell
$ json_env -c local.settings.json -p $.Values node app.js

```


## License

json_env is licensed under the Apache 2.0 license.

