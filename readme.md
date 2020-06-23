# Future discord bot

## Cloud function testing

To test locally, you'll need [functions-framework-python](https://github.com/GoogleCloudPlatform/functions-framework-python)

and you can just run `pipenv run test`

## Cloud function deploy

To deploy, you'll need the gcloud cli

`brew cask install google-cloud-sdk`

Then you create a project on the gcp website.

You'll need to set up credentials for your gcp account, maybe `gclould init`? I don't remember because I only did it once.

Once that's setup, you just need to `pipenv run deploy`

Google has a [quickstart guide](https://cloud.google.com/functions/docs/quickstart-python), but the paste-your-code-in-the-box method, I think doesn't allow you to include any dependencies. For that you gotta set up the CLI.