import random

dick = open('./data/dick.txt').read()
dicks = dick.split('\n\n')

dickens = open('./data/bleak-house.txt').read()
dickenses = dickens.split('\n\n')

def get_random_dick():
  random_dick = ''

  while (len(random_dick) < 100 or len(random_dick) > 800):
    random_dick = random.choice(dicks)
  
  return random_dick

def get_random_dickens():
  random_dickens = ''

  while (len(random_dickens) < 100 or len(random_dickens) > 800):
    random_dickens = random.choice(dickenses)
  
  return random_dickens

print(get_random_dickens())