import random
dick = open('./data/dick.txt').read()
dicks = dick.split('\n\n')

def get_random_dick():
  random_dick = ''

  while (len(random_dick) < 100 or len(random_dick) > 800):
    random_dick = random.choice(dicks)
  
  return random_dick
