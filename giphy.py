import time
import giphy_client
from giphy_client.rest import ApiException
import os

# create an instance of the API class
api_instance = giphy_client.DefaultApi()
api_key = os.getenv('GIPHY_TOKEN') # str | Giphy API Key.
rating = 'g' # str | Filters results by specified rating. (optional)
fmt = 'json' # str | Used to indicate the expected response format. Default is Json. (optional) (default to json)

def random_gif(search):
  try: 
    # Search Endpoint
    api_response = api_instance.gifs_random_get(api_key, tag=search, rating=rating, fmt=fmt)
    return api_response.data.image_url
  except ApiException as e:
    return 'http://gph.is/2efdN3V'