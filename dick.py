import random

dicks = open('./data/dick.txt').read().split('\n\n')
dickenses = open('./data/bleak-house.txt').read().split('\n\n')
willies = open('./data/willy.txt').read().split('\n\n')
summas = open('./data/summa.txt').read().split('\n\n')

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

def get_random_thomas():
  random_thomas = ''

  while (len(random_thomas) < 100 or len(random_thomas) > 800):
    random_thomas = random.choice(summas)
  
  return random_thomas
