import random

dick = open('./data/dick.txt').read()
dicks = dick.split('\n\n')

dickens = open('./data/bleak-house.txt').read()
dickenses = dickens.split('\n\n')

willy = open('./data/willy.txt').read()
willies = willy.split('\n\n')

def get_random_dick():
  random_dick = ''

  while (len(random_dick) < 100 or len(random_dick) > 800):
    random_dick = random.choice(dicks)
  
  return random_dick

def get_random_willy():
  random_willy = ''

  while (len(random_willy) < 100 or len(random_willy) > 800):
    random_willy = random.choice(willies)
  
  return random_willy
