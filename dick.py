import random

dicks = open('./data/dick.txt', encoding='utf8').read().split('\n\n')
dickenses = open('./data/bleak-house.txt', encoding='utf8').read().split('\n\n')
willies = open('./data/willy.txt', encoding='utf8').read().split('\n\n')
summas = open('./data/summa.txt', encoding='utf8').read().split('\n\n')
janes = open('./data/jane.txt', encoding='utf8').read().split('\n\n')
yoda = open('./data/star-wars-quotes.txt', encoding='utf8').read().split('\n\n')


dudes = {
  "dick": dicks,
  "dickens": dickenses,
  "willy": willies,
  "thomas": summas,
  "jane": janes
}

def get_random_quote(name):
  choice = ''
  pool = dudes[name]

  while (len(choice) < 100 or len(choice) > 800):
    choice = random.choice(pool)

  return choice
